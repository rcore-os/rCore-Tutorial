## 中断栈

### 为什么 / 怎么做

在实现中断栈之前，让我们先检查一下需求和我们的解决办法。

- **不是每个线程都需要一个独立的中断栈**，因为中断栈只会在中断时使用，而中断结束后就不再使用。在只有一个 CPU 的情况下，不会有两个线程同时出现中断，**所以我们只需要实现一个共用的中断栈就可以了**。
- **每个线程都需要能够在中断时第一时间找到中断栈的地址**。这时，所有通用寄存器的值都无法预知，也无法从某个变量来加载地址。为此，**我们将中断栈的地址存放到内核态使用的特权寄存器 `sscratch` 中**。这个寄存器只能在内核态访问，这样在中断发生时，就可以安全地找到中断栈了。

因此，我们的做法就是：

- 预留一段空间作为中断栈
- 运行线程时，在 `sscratch` 寄存器中保存中断栈指针  
- 如果线程遇到中断，则从将 `Context` 压入 `sscratch` 指向的栈中（`Context` 的地址为 `sscratch - size_of::<Context>()`），同时用新的栈地址来替换 `sp`（此时 `sp` 也会被复制到 `a0` 作为 `handle_interrupt` 的参数）  
- 从中断中返回时（`__restore` 时），`a0` 应指向**被压在中断栈中的 `Context`**。此时出栈 `Context` 并且将栈顶保存到 `sscratch` 中

### 实现

#### 为中断栈预留空间

我们直接使用一个 `static mut` 来指定一段空间作为栈。

{% label %}os/src/process/interrupt_stack.rs{% endlabel %}
```rust
/// 中断栈
#[repr(align(16))]
#[repr(C)]
pub struct InterruptStack([u8; INTERRUPT_STACK_SIZE]);

/// 公用的中断栈
pub static mut INTERRUPT_STACK: InterruptStack = InterruptStack([0; STACK_SIZE]);
```

在我们创建线程时，需要使用的操作就是在中断栈顶压入一个初始状态 `Context`：

{% label %}os/src/process/interrupt_stack.rs{% endlabel %}
```rust
impl InterruptStack {
    /// 在栈顶加入 Context 并且返回新的栈顶指针
    pub fn push_context(&mut self, context: Context) -> *mut Context {
        // 栈顶
        let stack_top = &self.0 as *const _ as usize + size_of::<Self>();
        // Context 的位置
        let push_address = (stack_top - size_of::<Context>()) as *mut Context;
        unsafe {
            *push_address = context;
        }
        push_address
    }
}
```

#### 修改 `interrupt.asm`

在这个汇编代码中，我们需要加入对 `sscratch` 的判断和使用。

{% label %}os/src/interrput/interrupt.asm{% endlabel %}
```asm
__interrupt:
    # 因为线程当前的栈不一定可用，必须切换到中断栈来保存 Context 并进行中断流程
    # 因此，我们使用 sscratch 寄存器保存中断栈地址
    # 思考：sscratch 的值最初是在什么地方写入的？

    # 交换 sp 和 sscratch（切换到中断栈）
    csrrw   sp, sscratch, sp
    # 在中断栈开辟 Context 的空间
    addi    sp, sp, -36*8

    # 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    # 将本来的栈地址 sp（即 x2）保存
    csrr    x1, sscratch
    SAVE    x1, 2

    # ...
```

以及事后的恢复：

{% label %}os/src/interrupt/interrupt.asm{% endlabel %}
```asm
# 离开中断
# 此时中断栈顶被推入了一个 Context，而 a0 指向它
# 接下来从 Context 中恢复所有寄存器，并将 Context 出栈（用 sscratch 记录中断栈地址）
# 最后跳转至恢复的 sepc 的位置
__restore:
    # 从 a0 中读取 sp
    # 思考：a0 是在哪里被赋值的？（有两种情况）
    mv      sp, a0
    # 恢复 CSR
    LOAD    t0, 32
    LOAD    t1, 33
    csrw    sstatus, t0
    csrw    sepc, t1
    # 将中断栈地址写入 sscratch
    addi    t0, sp, 36*8
    csrw    sscratch, t0

    # 恢复通用寄存器
    # ...
```

### 小结

为了能够鲁棒地处理用户线程产生的异常，我们为线程准备好一个中断栈，发生中断时会切换到这里继续处理。

#### 思考

在栈的切换过程中，会不会导致一些栈空间没有被释放，或者被错误释放的情况？
