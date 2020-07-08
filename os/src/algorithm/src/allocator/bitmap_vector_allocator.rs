//! 提供向量分配器的简单实现 [`BitmapVectorAllocator`]

use super::VectorAllocator;
use bit_field::BitArray;
use core::cmp::min;

/// Bitmap 中的位数（4K）
const BITMAP_SIZE: usize = 4096;

/// 向量分配器的简单实现，每字节用一位表示
pub struct BitmapVectorAllocator {
    /// 容量，单位为 bitmap 中可以使用的位数，即待分配空间的字节数
    capacity: usize,
    /// 每一位 0 表示空闲
    bitmap: [u8; BITMAP_SIZE / 8],
}

impl VectorAllocator for BitmapVectorAllocator {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: min(BITMAP_SIZE, capacity),
            bitmap: [0u8; BITMAP_SIZE / 8],
        }
    }
    fn alloc(&mut self, size: usize, align: usize) -> Option<usize> {
        for start in (0..self.capacity - size).step_by(align) {
            if (start..start + size).all(|i| !self.bitmap.get_bit(i)) {
                (start..start + size).for_each(|i| self.bitmap.set_bit(i, true));
                return Some(start);
            }
        }
        None
    }
    fn dealloc(&mut self, start: usize, size: usize, _align: usize) {
        assert!(self.bitmap.get_bit(start));
        (start..start + size).for_each(|i| self.bitmap.set_bit(i, false));
    }
}
