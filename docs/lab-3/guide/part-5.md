## 页面置换*

> **[info] 注意**
> 本小节涉及内容繁杂，实现也可能有考虑不周之处，具体的代码仅供有兴趣的同学阅读。
>
> 由于需要操作“页面置换文件”，页面置换的具体实现会用到文件系统（实验五）的框架。不过，实验五会提供非常抽象的接口，我们暂时不需要完全了解后面实验的实现方法。
>
> 如果你选择对照我们的实验指导，自行实现一个操作系统，你可以阅读本小节但暂时跳过这部分的实现。它不会影响任何后续实验的功能。

### 原理

虚拟内存的一大优势是可以用有限的物理内存空间虚拟出近乎无限的虚拟内存空间，其原理就是只将一部分虚拟内存所对应的数据存放在物理内存中，而剩余的则存放在磁盘（外设）中。当一个线程操作到那些不在物理内存中的虚拟地址时，就会产生**缺页异常（Page Fault）**。此时操作系统会介入，交换一部分物理内存和磁盘中的数据，使得需要访问的内存数据被放入物理内存之中。

在页表中，页表项的 Valid 位就表示对应的页面是否在物理内存中。因此，操作系统还必须更新页表，并刷新缓存。

### 置换算法

我们可以很自然地认为，操作系统需要将那些“经常被使用”的内存空间放在物理内存中，而“不那么经常被使用”的放在外部存储中。但是，计算机不可能预言哪些地址会在以后被访问。我们只能通过一些置换算法，根据前一段时间的内存使用情况，来估计未来哪些地址会被使用，从而将这部分数据保留在物理内存中。

朴素地考虑，我们可以假设如果程序刚刚访问了一部分内存，那么他就很有可能重复地访问它。因此，每次出现缺页时，就将物理内存中最后访问时间最靠前的页面替换出去。这就是**LRU (Least Recently Used) 算法**。但这种算法需要维护一个优先队列，而且在每一次访问内存时都要更新。很显然这是不现实的，它带来的开销太大。

目前存在着大量的置换算法，我们可以在[维基百科](https://en.wikipedia.org/wiki/Page_replacement_algorithm)上初步了解。

### 我们的实现

首先，我们要在磁盘中建立一个页面置换文件，来保存所有换出的页面。为了简化实现，我们直接在镜像中打包一个全是 0 的文件 `SWAP_FILE` 进去。

{% label %}user/Makefile{% endlabel %}
```makefile
# 编译、打包、格式转换、预留空间
build: dependency
	@cargo build
	@echo Targets: $(patsubst $(SRC_DIR)/%.rs, %, $(SRC_FILES))
	@rm -rf $(OUT_DIR)
	@mkdir -p $(OUT_DIR)
	@cp $(BIN_FILES) $(OUT_DIR)
-->	@dd if=/dev/zero of=$(OUT_DIR)/SWAP_FILE bs=1M count=16
	@rcore-fs-fuse --fs sfs $(IMG_FILE) $(OUT_DIR) zip
	@qemu-img convert -f raw $(IMG_FILE) -O qcow2 $(QCOW_FILE)
	@qemu-img resize $(QCOW_FILE) +1G
```

我们希望每个进程的 `Mapping` 都能够像管理物理页面一样管理这些置换页面（在销毁时也能够释放它们），因此我们实现了一个类似于 `FrameTracker` 的 `SwapTracker`。它的具体实现会用到一些文件系统的操作，如果感兴趣可以参考源码。但简概括之：**`SwapTracker` 记录了一个被置换出物理内存的页面，并提供一些便捷的操作接口**。

{% label %}os/src/fs/swap.rs{% endlabel %}
```rust
/// 类似于 [`FrameTracker`]，相当于 `Box<置换文件中的一个页面>`
///
/// 内部保存该置换页面在文件中保存的 index
///
/// [`FrameTracker`]: crate::memory::frame::FrameTracker
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SwapTracker(pub(super) usize);

impl SwapTracker {
    /// 从置换文件分配一个页面空间
    pub fn new() -> MemoryResult<Self> {
        ...
    }

    /// 读取页面数据
    pub fn read(&self) -> [u8; PAGE_SIZE] {
        ...
    }

    /// 写入页面数据
    pub fn write(&self, data: &[u8; PAGE_SIZE]) {
        ...
    }
}

impl Drop for SwapTracker {
    fn drop(&mut self) {
        ...
    }
}
```

然后，我们定义了一个置换算法的接口，并且实现了一个非常简单的置换算法，具体算法就不呈现了。

{% label %}os/src/memory/mapping/swapper.rs{% endlabel %}
```rust
/// 管理一个线程所映射的页面的置换操作
pub trait Swapper {
    /// 新建带有一个分配数量上限的置换器
    fn new(quota: usize) -> Self;

    /// 是否已达到上限
    fn full(&self) -> bool;

    /// 取出一组映射
    fn pop(&mut self) -> Option<(VirtualPageNumber, FrameTracker)>;

    /// 添加一组映射（不会在以达到分配上限时调用）
    fn push(&mut self, vpn: VirtualPageNumber, frame: FrameTracker);

    /// 只保留符合某种条件的条目（用于移除一段虚拟地址）
    fn retain(&mut self, predicate: impl Fn(&VirtualPageNumber) -> bool);
}
```

这里，`Swapper` 就替代了 `Mapping` 中的 `mapped_pairs: Vec<(VirtualPageNumber, FrameTracker)>` 的作用。因此，我们替换 `Mapping` 中的成员：

{% label %}os/src/memory/mapping/mapping.rs{% endlabel %}
```rust
/// 某个进程的内存映射关系
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
```

最后，让我们实现内存置换：遇到缺页异常，找到需要访问的页号、需要访问的页面数据，并置换出一个物理内存中的页号、页面数据，将二者进行交换

{% label %}os/src/memory/mapping/mapping.rs{% endlabel %}
```rust
impl Mapping {
    /// 处理缺页异常
    pub fn handle_page_fault(&mut self, stval: usize) -> MemoryResult<()> {
        let vpn = VirtualPageNumber::floor(stval.into());
        let swap_tracker = self
            .swapped_pages
            .remove(&vpn)
            .ok_or("stval page is not mapped")?;
        let page_data = swap_tracker.read();

        if self.mapped_pairs.full() {
            // 取出一个映射
            let (popped_vpn, mut popped_frame) = self.mapped_pairs.pop().unwrap();
            // print!("{:x?} -> {:x?}", popped_vpn, vpn);
            // 交换数据
            swap_tracker.write(&*popped_frame);
            (*popped_frame).copy_from_slice(&page_data);
            // 修改页表映射
            self.invalidate_one(popped_vpn)?;
            self.remap_one(vpn, popped_frame.page_number())?;
            // 更新记录
            self.mapped_pairs.push(vpn, popped_frame);
            self.swapped_pages.insert(popped_vpn, swap_tracker);
        } else {
            // 如果当前还没有达到配额，则可以继续分配物理页面。这种情况目前还不会出现
            // 添加新的映射
            let mut frame = FRAME_ALLOCATOR.lock().alloc()?;
            // 复制数据
            (*frame).copy_from_slice(&page_data);
            // 更新映射
            self.remap_one(vpn, frame.page_number())?;
            // 更新记录
            self.mapped_pairs.push(vpn, frame);
        }
        Ok(())
    }
}
```

然后，令缺页异常调用上面的函数，就完成了页面置换的实现

{% label %}os/src/interrupt/handler.rs{% endlabel %}
```rust
/// 处理缺页异常
///
/// todo: 理论上这里需要判断访问类型，并与页表中的标志位进行比对
fn page_fault(context: &mut Context, stval: usize) -> Result<*mut Context, String> {
    println!("page_fault");
    let current_thread = PROCESSOR.get().current_thread();
    let memory_set = &mut current_thread.process.write().memory_set;
    memory_set.mapping.handle_page_fault(stval)?;
    memory_set.activate();
    Ok(context)
}
```