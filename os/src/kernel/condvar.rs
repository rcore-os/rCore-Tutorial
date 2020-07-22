//! 条件变量

use super::*;
use alloc::collections::VecDeque;

#[derive(Default)]
pub struct Condvar {
    /// 所有等待此条件变量的线程
    watchers: Mutex<VecDeque<Arc<Thread>>>,
}

impl Condvar {
    /// 令当前线程休眠，等待此条件变量
    pub fn wait(&self) {
        self.watchers
            .lock()
            .push_back(PROCESSOR.lock().current_thread());
        PROCESSOR.lock().sleep_current_thread();
    }

    /// 唤起一个等待此条件变量的线程
    pub fn notify_one(&self) {
        if let Some(thread) = self.watchers.lock().pop_front() {
            PROCESSOR.lock().wake_thread(thread);
        }
    }
}
