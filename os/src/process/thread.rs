//! 线程 [`Thread`]

use super::*;
use crate::fs::*;
use core::hash::{Hash, Hasher};

/// 线程 ID 使用 `isize`，可以用负数表示错误
pub type ThreadID = isize;

static mut THREAD_COUNTER: ThreadID = 0;

/// 线程的信息
pub struct Thread {
    /// 线程 ID
    pub id: ThreadID,
    /// 线程的栈
    pub stack: Range<VirtualAddress>,
    /// 所属的进程
    pub process: Arc<RwLock<Process>>,
    /// 用 `Mutex` 包装一些可变的变量
    pub inner: Mutex<ThreadInner>,
}

/// 线程中需要可变的部分
pub struct ThreadInner {
    /// 线程执行上下文
    ///
    /// 当且仅当线程被暂停执行时，`context` 为 `Some`
    pub context: Option<Context>,
    /// 是否进入休眠
    pub sleeping: bool,
    /// 是否已经结束
    pub dead: bool,
    /// 打开的文件
    pub descriptors: Vec<Arc<dyn INode>>,
}

impl Thread {
    /// 准备执行一个线程
    ///
    /// 激活对应进程的页表，并返回其 Context
    pub fn prepare(&self) -> *mut Context {
        // 激活页表
        self.process.write().memory_set.activate();
        // 取出 Context
        let parked_frame = self.inner().context.take().unwrap();
        // 将 Context 放至内核栈顶
        unsafe { KERNEL_STACK.push_context(parked_frame) }
    }

    /// 发生时钟中断后暂停线程，保存状态
    pub fn park(&self, context: Context) {
        // 检查目前线程内的 context 应当为 None
        assert!(self.inner().context.is_none());
        // 将 Context 保存到线程中
        self.inner().context.replace(context);
    }

    /// 创建一个线程
    pub fn new(
        process: Arc<RwLock<Process>>,
        entry_point: usize,
        arguments: Option<&[usize]>,
    ) -> MemoryResult<Arc<Thread>> {
        // 让所属进程分配并映射一段空间，作为线程的栈
        let stack = process
            .write()
            .alloc_page_range(STACK_SIZE, Flags::READABLE | Flags::WRITABLE)?;

        // 构建线程的 Context
        let context = Context::new(
            stack.end.into(),
            entry_point,
            arguments,
            process.read().is_user,
        );

        // 打包成线程
        let thread = Arc::new(Thread {
            id: unsafe {
                THREAD_COUNTER += 1;
                THREAD_COUNTER
            },
            stack,
            process,
            inner: Mutex::new(ThreadInner {
                context: Some(context),
                sleeping: false,
                dead: false,
                descriptors: vec![STDIN.clone(), STDOUT.clone()],
            }),
        });

        Ok(thread)
    }

    pub fn inner(&self) -> spin::MutexGuard<ThreadInner> {
        self.inner.lock()
    }
}

/// 通过线程 ID 来判等
impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// 通过线程 ID 来判等
///
/// 在 Rust 中，[`PartialEq`] trait 不要求任意对象 `a` 满足 `a == a`。
/// 将类型标注为 [`Eq`]，会沿用 `PartialEq` 中定义的 `eq()` 方法，
/// 同时声明对于任意对象 `a` 满足 `a == a`。
impl Eq for Thread {}

/// 通过线程 ID 来哈希
impl Hash for Thread {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_isize(self.id);
    }
}

/// 打印线程除了父进程以外的信息
impl core::fmt::Debug for Thread {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter
            .debug_struct("Thread")
            .field("thread_id", &self.id)
            .field("stack", &self.stack)
            .field("context", &self.inner().context)
            .finish()
    }
}
