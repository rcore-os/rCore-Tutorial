//! 页面置换算法

use crate::memory::{frame::FrameTracker, *};
use alloc::collections::VecDeque;
use hashbrown::HashMap;

/// 页面置换算法
pub trait Swapper {
    /// 新建带有一个分配数量上限的置换器
    fn new(quota: usize) -> Self;

    /// 是否已达到上限
    fn full(&self) -> bool;

    /// 取出一组映射
    fn pop(&mut self) -> Option<(VirtualPageNumber, FrameTracker)>;

    /// 添加一组映射（不会在以达到分配上限时调用）
    fn push(&mut self, vpn: VirtualPageNumber, frame: FrameTracker);

    /// 找到某个页号对应的页面
    fn find(&mut self, vpn: VirtualPageNumber) -> Option<&mut FrameTracker>;

    /// 只保留符合某种条件的条目（用于移除一段虚拟地址）
    fn retain(&mut self, predicate: impl Fn(&VirtualPageNumber) -> bool);
}

pub type SwapperImpl = FIFOSwapper;

/// 页面置换算法基础实现：FIFO
pub struct FIFOSwapper {
    /// 记录所有映射
    entries: HashMap<VirtualPageNumber, FrameTracker>,
    /// 记录映射添加的顺序
    queue: VecDeque<VirtualPageNumber>,
    /// 映射数量上限
    quota: usize,
}

impl Swapper for FIFOSwapper {
    fn new(quota: usize) -> Self {
        Self {
            entries: HashMap::new(),
            queue: VecDeque::new(),
            quota,
        }
    }
    fn full(&self) -> bool {
        self.entries.len() == self.quota
    }
    fn pop(&mut self) -> Option<(VirtualPageNumber, FrameTracker)> {
        self.queue
            .pop_front()
            .map(|vpn| (vpn, self.entries.remove(&vpn).unwrap()))
    }
    fn push(&mut self, vpn: VirtualPageNumber, frame: FrameTracker) {
        self.queue.push_back(vpn);
        self.entries.insert(vpn, frame);
    }
    fn find(&mut self, vpn: VirtualPageNumber) -> Option<&mut FrameTracker> {
        self.entries.get_mut(&vpn)
    }
    fn retain(&mut self, predicate: impl Fn(&VirtualPageNumber) -> bool) {
        self.queue.retain(|vpn| predicate(vpn));
        self.entries.retain(|vpn, _| predicate(vpn));
    }
}
