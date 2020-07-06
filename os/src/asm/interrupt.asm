# 宏：将寄存器存到栈上
.macro SAVE reg, offset
    sd  \reg, \offset*8(sp)
.endm

# 宏：将寄存器从栈中取出
.macro LOAD reg, offset
    ld  \reg, \offset*8(sp)
.endm

    .section .text
    .globl __interrupt
# 进入中断
# 保存 Context 并且进入 rust 中的中断处理函数 interrupt::handler::handle_interrupt()
__interrupt:
    # 因为线程当前的栈不一定可用，必须切换到内核栈来保存 Context 并进行中断流程
    # 因此，我们使用 sscratch 寄存器保存内核栈地址
    # 思考：sscratch 的值最初是在什么地方写入的？

    # 交换 sp 和 sscratch（切换到内核栈）
    csrrw   sp, sscratch, sp
    # 在内核栈开辟 Context 的空间
    addi    sp, sp, -36*8
    
    # 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    # 将本来的栈地址 sp（即 x2）保存
    csrr    x1, sscratch
    SAVE    x1, 2
    SAVE    x3, 3
    SAVE    x4, 4
    SAVE    x5, 5
    SAVE    x6, 6
    SAVE    x7, 7
    SAVE    x8, 8
    SAVE    x9, 9
    SAVE    x10, 10
    SAVE    x11, 11
    SAVE    x12, 12
    SAVE    x13, 13
    SAVE    x14, 14
    SAVE    x15, 15
    SAVE    x16, 16
    SAVE    x17, 17
    SAVE    x18, 18
    SAVE    x19, 19
    SAVE    x20, 20
    SAVE    x21, 21
    SAVE    x22, 22
    SAVE    x23, 23
    SAVE    x24, 24
    SAVE    x25, 25
    SAVE    x26, 26
    SAVE    x27, 27
    SAVE    x28, 28
    SAVE    x29, 29
    SAVE    x30, 30
    SAVE    x31, 31

    # 取出 CSR 并保存
    csrr    t0, sstatus
    csrr    t1, sepc
    SAVE    t0, 32
    SAVE    t1, 33
    # 调用 handle_interrupt，传入参数
    # context: &mut Context
    mv      a0, sp
    # scause: Scause
    csrr    a1, scause
    # stval: usize
    csrr    a2, stval
    jal handle_interrupt

    .globl __restore
# 离开中断
# 此时内核栈顶被推入了一个 Context，而 a0 指向它
# 接下来从 Context 中恢复所有寄存器，并将 Context 出栈（用 sscratch 记录内核栈地址）
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
    # 将内核栈地址写入 sscratch
    addi    t0, sp, 36*8
    csrw    sscratch, t0

    # 恢复通用寄存器
    LOAD    x1, 1
    LOAD    x3, 3
    LOAD    x4, 4
    LOAD    x5, 5
    LOAD    x6, 6
    LOAD    x7, 7
    LOAD    x8, 8
    LOAD    x9, 9
    LOAD    x10, 10
    LOAD    x11, 11
    LOAD    x12, 12
    LOAD    x13, 13
    LOAD    x14, 14
    LOAD    x15, 15
    LOAD    x16, 16
    LOAD    x17, 17
    LOAD    x18, 18
    LOAD    x19, 19
    LOAD    x20, 20
    LOAD    x21, 21
    LOAD    x22, 22
    LOAD    x23, 23
    LOAD    x24, 24
    LOAD    x25, 25
    LOAD    x26, 26
    LOAD    x27, 27
    LOAD    x28, 28
    LOAD    x29, 29
    LOAD    x30, 30
    LOAD    x31, 31

    # 恢复 sp（又名 x2）这里最后恢复是为了上面可以正常使用 LOAD 宏
    LOAD    x2, 2
    sret