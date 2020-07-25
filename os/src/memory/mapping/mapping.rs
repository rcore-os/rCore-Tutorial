//! Rv39 页表的构建 [`Mapping`]
//!
//! 许多方法返回 [`Result`]，如果出现错误会返回 `Err(message)`。设计目标是，此时如果终止线程，则不会产生后续问题。
//! 但是如果错误是由操作系统代码逻辑产生的，则会直接 panic。

use crate::fs::SwapTracker;
use crate::memory::{
    address::*,
    config::PAGE_SIZE,
    frame::FRAME_ALLOCATOR,
    mapping::{
        Flags, MapType, PageTable, PageTableEntry, PageTableTracker, Segment, Swapper, SwapperImpl,
    },
    MemoryResult,
};
use alloc::{vec, vec::Vec};
use core::{cmp::min, ptr::slice_from_raw_parts_mut};
use hashbrown::HashMap;

/// 某个线程的内存映射关系
pub struct Mapping {
    /// 保存所有使用到的页表
    page_tables: Vec<PageTableTracker>,
    /// 根页表的物理页号
    root_ppn: PhysicalPageNumber,
    /// 所有分配的物理页面映射信息
    mapped_pairs: SwapperImpl,
    /// 被换出的页面存储在虚拟内存文件中的 Tracker
    swapped_pages: HashMap<VirtualPageNumber, SwapTracker>,
}

impl Mapping {
    /// 将当前的映射加载到 `satp` 寄存器并记录
    pub fn activate(&self) {
        // satp 低 27 位为页号，高 4 位为模式，8 表示 Sv39
        let new_satp = self.root_ppn.0 | (8 << 60);
        unsafe {
            // 将 new_satp 的值写到 satp 寄存器
            llvm_asm!("csrw satp, $0" :: "r"(new_satp) :: "volatile");
            // 刷新 TLB
            llvm_asm!("sfence.vma" :::: "volatile");
        }
    }

    /// 创建一个有根节点的映射
    pub fn new(frame_quota: usize) -> MemoryResult<Mapping> {
        let root_table = PageTableTracker::new(FRAME_ALLOCATOR.lock().alloc()?);
        let root_ppn = root_table.page_number();
        Ok(Mapping {
            page_tables: vec![root_table],
            root_ppn,
            mapped_pairs: SwapperImpl::new(frame_quota),
            swapped_pages: HashMap::new(),
        })
    }

    /// 加入一段映射，可能会相应地分配物理页面
    ///
    /// 未被分配物理页面的虚拟页号暂时不会写入页表当中，它们会在发生 PageFault 后再建立页表项。
    pub fn map(&mut self, segment: &Segment, init_data: Option<&[u8]>) -> MemoryResult<()> {
        match segment.map_type {
            // 线性映射，直接对虚拟地址进行转换
            MapType::Linear => {
                for vpn in segment.page_range().iter() {
                    self.map_one(vpn, Some(vpn.into()), segment.flags)?;
                }
                // 拷贝数据
                if let Some(data) = init_data {
                    unsafe {
                        (&mut *slice_from_raw_parts_mut(segment.range.start.deref(), data.len()))
                            .copy_from_slice(data);
                    }
                }
            }
            // 需要分配帧进行映射
            MapType::Framed => {
                for vpn in segment.page_range().iter() {
                    // 如果有初始化数据，找到相应的数据
                    let page_data = if init_data.is_none() || init_data.unwrap().is_empty() {
                        [0u8; PAGE_SIZE]
                    } else {
                        // 这里必须进行一些调整，因为传入的数据可能并非按照整页对齐

                        // 传入的初始化数据
                        let init_data = init_data.unwrap();
                        // 整理后将要返回的一整个页面的数据
                        let mut page_data = [0u8; PAGE_SIZE];

                        // 拷贝时必须考虑区间与整页不对齐的情况
                        //    start（仅第一页时非零）
                        //      |        stop（仅最后一页时非零）
                        // 0    |---data---|          4096
                        // |------------page------------|
                        let page_address = VirtualAddress::from(vpn);
                        let start = if segment.range.start > page_address {
                            segment.range.start - page_address
                        } else {
                            0
                        };
                        let stop = min(PAGE_SIZE, segment.range.end - page_address);
                        // 计算来源和目标区间并进行拷贝
                        let dst_slice = &mut page_data[start..stop];
                        let src_slice = &init_data[(page_address + start - segment.range.start)
                            ..(page_address + stop - segment.range.start)];
                        dst_slice.copy_from_slice(src_slice);

                        page_data
                    };

                    // 建立映射，先检查是否有配额来分配新的物理页面
                    if !self.mapped_pairs.full() {
                        // 有配额，分配新的物理页面
                        let mut frame = FRAME_ALLOCATOR.lock().alloc()?;
                        // 更新页表
                        let entry = self.map_one(vpn, Some(frame.page_number()), segment.flags)?;
                        // 写入数据
                        (*frame).copy_from_slice(&page_data);
                        // 保存
                        self.mapped_pairs.push(vpn, frame, entry);
                    } else {
                        // 配额用完，改为在置换文件中分配一个页面
                        let swap_page = SwapTracker::new()?;
                        // 更新页表
                        self.map_one(vpn, None, segment.flags)?;
                        // 写入数据
                        swap_page.write(&page_data);
                        // 保存
                        self.swapped_pages.insert(vpn, swap_page);
                    }
                }
            }
        }
        Ok(())
    }

    /// 移除一段映射
    pub fn unmap(&mut self, segment: &Segment) {
        for vpn in segment.page_range().iter() {
            let entry = self.find_entry(vpn).unwrap();
            assert!(!entry.is_empty());
            // 从页表中清除项
            entry.clear();
        }
        self.mapped_pairs
            .retain(|vpn| !segment.page_range().contains(*vpn));
        self.swapped_pages
            .retain(|vpn, _| !segment.page_range().contains(*vpn));
    }

    /// 找到给定虚拟页号的三级页表项
    ///
    /// 如果找不到对应的页表项，则会相应创建页表
    pub fn find_entry(&mut self, vpn: VirtualPageNumber) -> MemoryResult<&mut PageTableEntry> {
        // 从根页表开始向下查询
        // 这里不用 self.page_tables[0] 避免后面产生 borrow-check 冲突（我太菜了）
        let root_table: &mut PageTable = PhysicalAddress::from(self.root_ppn).deref_kernel();
        let mut entry = &mut root_table.entries[vpn.levels()[0]];
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                // 如果页表不存在，则需要分配一个新的页表
                let new_table = PageTableTracker::new(FRAME_ALLOCATOR.lock().alloc()?);
                let new_ppn = new_table.page_number();
                // 将新页表的页号写入当前的页表项
                *entry = PageTableEntry::new(Some(new_ppn), Flags::VALID);
                // 保存页表
                self.page_tables.push(new_table);
            }
            // 进入下一级页表（使用偏移量来访问物理地址）
            entry = &mut entry.get_next_table().entries[*vpn_slice];
        }
        // 此时 entry 位于第三级页表
        Ok(entry)
    }

    /// 查找虚拟地址对应的物理地址
    pub fn lookup(va: VirtualAddress) -> Option<PhysicalAddress> {
        let mut current_ppn;
        unsafe {
            llvm_asm!("csrr $0, satp" : "=r"(current_ppn) ::: "volatile");
            current_ppn ^= 8 << 60;
        }

        let root_table: &PageTable =
            PhysicalAddress::from(PhysicalPageNumber(current_ppn)).deref_kernel();
        let vpn = VirtualPageNumber::floor(va);
        let mut entry = &root_table.entries[vpn.levels()[0]];
        // 为了支持大页的查找，我们用 length 表示查找到的物理页需要加多少位的偏移
        let mut length = 12 + 2 * 9;
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                return None;
            }
            if entry.has_next_level() {
                length -= 9;
                entry = &mut entry.get_next_table().entries[*vpn_slice];
            } else {
                break;
            }
        }
        let base = PhysicalAddress::from(entry.page_number()).0;
        let offset = va.0 & ((1 << length) - 1);
        Some(PhysicalAddress(base + offset))
    }

    /// 为给定的虚拟 / 物理页号建立映射关系
    fn map_one(
        &mut self,
        vpn: VirtualPageNumber,
        ppn: Option<PhysicalPageNumber>,
        flags: Flags,
    ) -> MemoryResult<*mut PageTableEntry> {
        // 定位到页表项
        let entry = self.find_entry(vpn)?;
        assert!(entry.is_empty(), "virtual address is already mapped");
        // 页表项为空，则写入内容
        *entry = PageTableEntry::new(ppn, flags);
        Ok(entry)
    }

    /// 移除一个虚拟地址的映射
    fn invalidate_one(&mut self, vpn: VirtualPageNumber) -> MemoryResult<()> {
        let entry = self.find_entry(vpn).expect("swapping out unmapped address");
        assert!(entry.is_valid());
        entry.update_page_number(None);
        Ok(())
    }

    /// 为指定的虚拟页号设置新的物理页面映射，要求虚拟页号在页表中但无现有映射
    fn remap_one(
        &mut self,
        vpn: VirtualPageNumber,
        ppn: PhysicalPageNumber,
    ) -> MemoryResult<*mut PageTableEntry> {
        let entry = self.find_entry(vpn)?;
        assert!(!entry.is_empty() && !entry.is_valid());
        // 设置页表项的物理页号和 Valid 标志
        entry.update_page_number(Some(ppn));
        Ok(entry)
    }

    /// 处理缺页异常
    pub fn handle_page_fault(&mut self, stval: usize) -> MemoryResult<()> {
        // 发生异常的访问页面
        let vpn = VirtualPageNumber::floor(stval.into());
        // 该页面如果在进程的内存中，则应该在 Swap 中，将其取出
        let swap_tracker: SwapTracker = self
            .swapped_pages
            .remove(&vpn)
            .ok_or("stval page is not mapped")?;
        // 读取该页面的数据
        let page_data = swap_tracker.read();

        if self.mapped_pairs.full() {
            // 取出一个映射
            let (popped_vpn, mut popped_frame) = self.mapped_pairs.pop().unwrap();
            // 交换数据
            swap_tracker.write(&*popped_frame);
            (*popped_frame).copy_from_slice(&page_data);
            // 修改页表映射
            self.invalidate_one(popped_vpn)?;
            let entry = self.remap_one(vpn, popped_frame.page_number())?;
            // 更新记录
            self.mapped_pairs.push(vpn, popped_frame, entry);
            self.swapped_pages.insert(popped_vpn, swap_tracker);
        } else {
            // 添加新的映射
            let mut frame = FRAME_ALLOCATOR.lock().alloc()?;
            // 复制数据
            (*frame).copy_from_slice(&page_data);
            // 更新映射
            let entry = self.remap_one(vpn, frame.page_number())?;
            // 更新记录
            self.mapped_pairs.push(vpn, frame, entry);
        }
        Ok(())
    }
}
