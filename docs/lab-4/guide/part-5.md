## 调度器

### 处理器抽象

我们已经可以创建和保存线程了，现在，我们再抽象出「处理器」来存放和管理线程池。同时，也需要存放和管理目前正在执行的线程（即中断前执行的线程，因为操作系统在工作时是处于中断、异常或系统调用服务之中）。

{% label %}os/src/process/processor.rs{% endlabel %}
```rust
lazy_static! {
    /// 全局的 [`Processor`]
    pub static ref PROCESSOR: Lock<Processor> = Lock::new(Processor::default());
}

/// 线程调度和管理
#[derive(Default)]
pub struct Processor {
    /// 当前正在执行的线程
    current_thread: Option<Arc<Thread>>,
    /// 线程调度器，记录活跃线程
    scheduler: SchedulerImpl<Arc<Thread>>,
    /// 保存休眠线程
    sleeping_threads: HashSet<Arc<Thread>>,
}
```

注意到这里我们用了一个 `Lock`，它封装了 `spin::Mutex`，而在其基础上进一步关闭了中断。这是因为我们在内核线程中也有可能访问 `PROCESSOR`，但是此时我们不希望它被时钟打断，这样在中断处理中就无法访问 `PROCESSOR` 了，因为它已经被锁住。

### 调度器

调度器的算法有许多种，我们将它提取出一个 trait 作为接口

{% label %}os/src/algorithm/src/scheduler/mod.rs{% endlabel %}
```rust
/// 线程调度器
///
/// 这里 `ThreadType` 就是 `Arc<Thread>`
pub trait Scheduler<ThreadType: Clone + Eq>: Default {
    /// 优先级的类型
    type Priority;
    /// 向线程池中添加一个线程
    fn add_thread(&mut self, thread: ThreadType);
    /// 获取下一个时间段应当执行的线程
    fn get_next(&mut self) -> Option<ThreadType>;
    /// 移除一个线程
    fn remove_thread(&mut self, thread: &ThreadType);
    /// 设置线程的优先级
    fn set_priority(&mut self, thread: ThreadType, priority: Self::Priority);
}
```

具体的算法就不在此展开了，我们可以参照目录 `os/src/algorithm/src/scheduler` 下的一些样例。

### 运行！

最后，让我们补充 `Processor::run` 的实现，让我们运行起第一个线程！

{% label %}os/src/process/processor.rs: impl Processor{% endlabel %}
```rust
/// 第一次开始运行
pub fn run(&mut self) -> ! {
    // interrupt.asm 中的标签
    extern "C" {
        fn __restore(context: usize);
    }
    // 从 current_thread 中取出 Context
    if self.current_thread.is_none() {
        panic!("no thread to run, shutting down");
    }
    let context = self.current_thread().prepare();
    // 从此将没有回头
    unsafe {
        __restore(context as usize);
    }
    unreachable!()
}
```

修改 `main.rs`，我们就可以跑起来多线程了。

{% label %}os/src/main.rs{% endlabel %}
```rust
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    interrupt::init();

    {
        // 新建一个带有内核映射的进程。需要执行的代码就在内核中
        let process = Process::new_kernel().unwrap();

        for message in 0..8 {
            let thread = Thread::new(
                process.clone(),            // 使用同一个进程
                sample_process as usize,    // 入口函数
                Some(&[message]),           // 参数
            ).unwrap();
            PROCESSOR.get().add_thread(thread);
        }
    }

    PROCESSOR.get().run();
}

fn sample_process(message: usize) {
    for i in 0..1000000 {
        if i % 200000 == 0 {
            println!("thread {}", message);
        }
    }
}

```

运行一下，我们会得到类似的输出：

{% label %}运行输出{% endlabel %}
```
thread 7
thread 6
thread 5
...
thread 7
...
thread 2
thread 1
thread 0
...
```
