## 线程的结束

### 现有问题

当内核线程终止时，会发生什么？如果我们按照实验指导中的实现，应该会观察到：内核线程在运行完成后触发了 `Exception::InstructionPageFault` 而终止，其中访问的的地址 `stval = 0`。

这是因为内核线程在执行完 `entry_point` 所指向的函数后会返回到 `ra` 指向的地址，而我们没有为其赋初值（初值为 0）。此时，程序就会尝试跳转到 `0x0` 地址，而显然它是不存在的。

### 解决办法

很自然的，我们希望能够让内核线程在结束时触发一个友善的中断（而不是一个看上去像是错误的缺页异常），然后被操作系统释放。我们可能会想到系统调用，但很可惜我们无法使用它，因为系统调用的本质是一个环境调用 `ecall`，而在内核线程（内核态）中进行的环境调用是用来与 M 态通信的。我们之前实现的 SBI 调用就是使用的 S 态 `ecall`。

因此，我们设计一个折衷的解决办法：内核线程将自己标记为“已结束”，同时触发一个普通的异常 `ebreak`。此时操作系统观察到线程的标记，便将其终止。

{% label %}os/src/main.rs{% endlabel %}
```rust
/// 内核线程需要调用这个函数来退出
fn kernel_thread_exit() {
    // 当前线程标记为结束
    PROCESSOR.get().current_thread().as_ref().inner().dead = true;
    // 制造一个中断来交给操作系统处理
    unsafe { llvm_asm!("ebreak" :::: "volatile") };
}
```

然后，我们将这个函数作为内核线程的 `ra`，使得它执行的函数完成后便执行 `kernel_thread_exit()`

{% label %}os/src/main.rs{% endlabel %}
```rust
// 设置线程的返回地址为 kernel_thread_exit
thread.as_ref().inner().context.as_mut().unwrap().set_ra(kernel_thread_exit as usize);
```
