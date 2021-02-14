## 调度器

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

修改 `main.rs`，我们就可以跑起来多线程了。

{% label %}os/src/main.rs{% endlabel %}
```rust
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    interrupt::init();

    {
        let mut processor = PROCESSOR.lock();
        // 创建一个内核进程
        let kernel_process = Process::new_kernel().unwrap();
        // 为这个进程创建多个线程，并设置入口均为 sample_process，而参数不同
        for i in 1..9usize {
            processor.add_thread(create_kernel_thread(
                kernel_process.clone(),
                sample_process as usize,
                Some(&[i]),
            ));
        }
    }

    extern "C" {
        fn __restore(context: usize);
    }
    // 获取第一个线程的 Context
    let context = PROCESSOR.lock().prepare_next_thread();
    // 启动第一个线程
    unsafe { __restore(context as usize) };
    unreachable!()
}

fn sample_process(id: usize) {
    println!("hello from kernel thread {}", id);
}
```

运行一下，我们会得到类似的输出：

{% label %}运行输出{% endlabel %}
```
hello from kernel thread 7
thread 7 exit
hello from kernel thread 6
thread 6 exit
hello from kernel thread 5
thread 5 exit
hello from kernel thread 4
thread 4 exit
hello from kernel thread 3
thread 3 exit
hello from kernel thread 2
thread 2 exit
hello from kernel thread 1
thread 1 exit
hello from kernel thread 8
thread 8 exit
src/process/processor.rs:87: 'all threads terminated, shutting down'
```
