## 实验一：中断

### 实验之前

- 阅读实验指导零和一，最好一步步跟着实现一遍。
- checkout 到仓库中的 `lab-1` 分支，实验题将以此展开。

> 我们的实验题会提供一个基础的代码框架，以便于进行实验。如果你选择参考教程，自己编写操作系统，这个代码框架也可以用来进行对照。

### 实验题

<!--
reveal tag 作为 anchor 必须有缩进，否则会打破前后的序号关系。其中的文字不能缩进，否则 click-reveal 会把格式搞乱
reveal 还会把行间距吃掉，所以手动加 <br>
-->

1.  简述：在 `rust_main` 函数中，执行 `ebreak` 命令后至函数结束前，`sp` 寄存器的值是怎样变化的？

    {% reveal %}
> - `sp` 首先减去一个 `Context` 的大小（入栈），然后原 `sp` 的值被保存到这个入栈的 `Context` 中。
>
> - 执行 `handle_interrupt` 的过程中，随着局部变量的使用，编译器可能会自动加入一些出入栈操作。但无论如何，`handle_interrupt` 前后 `sp` 的值是一样的。
>
> - 从 `handle_interrupt` 返回后，执行 `__restore`，在最后将保存的原 `sp` 值恢复。
    {% endreveal %}

    <br>
2.  回答：如果去掉 `rust_main` 后的 `panic` 会发生什么，为什么？

    {% reveal %}
> `rust_main` 返回后，程序并没有停止。`rust_main` 是在 `entry.asm` 中通过 `jal` 指令调用的，因此其执行完后会回到 `entry.asm` 中。但是，`entry.asm` 并没有在后面写任何指令，这意味着程序将接着向后执行内存中的任何指令。
>
> 我们可以通过 `rust-objdump -d -S os/target/riscv64imac-unknown-none-elf/debug/os | less` 来查看汇编代码，其中就能看到：`_start` 只有短短三条指令，而后面则放着许多 Rust 库中的函数。这些指令可能导致程序进入循环，或崩溃退出。
    {% endreveal %}

    <br>
3.  实验
    1.  如果程序访问不存在的地址，会得到 `Exception::LoadFault`。模仿捕获 `ebreak` 和时钟中断的方法，捕获 `LoadFault`（之后 `panic` 即可）。

        {% reveal %}
> 直接在 `match` 中添加一个 arm 即可。例如 `Trap::Exception(Exception::LoadFault) => panic!()`
        {% endreveal %}

        <br>
    2.  在处理异常的过程中，如果程序想要非法访问的地址是 `0x0`，则打印 `SUCCESS!`。

        {% reveal %}
> 如果程序因无效访问内存造成异常，这个访问的地址会被存放在 `stval` 中，而它已经被我们作为参数传入 `handle_interrupt` 了，因此直接判断即可
        {% endreveal %}

        <br>
    3.  添加或修改少量代码，使得运行时触发这个异常，并且打印出 `SUCCESS!`。
        - 要求：不允许添加或修改任何 unsafe 代码

        <br>

        {% reveal %}
> - 解法 1：在 `interrupt/handler.rs` 的 `breakpoint` 函数中，将 `context.sepc += 2` 修改为 `context.sepc = 0`（则 `sret` 时程序会跳转到 `0x0`）
> - 解法 2：去除 `rust_main` 中的 `panic` 语句，并在 `entry.asm` 的 `jal rust_main` 之后，添加一行读取 `0x0` 地址的指令（例如 `jr x0` 或 `ld x1, (x0)`）

        {% endreveal %}
