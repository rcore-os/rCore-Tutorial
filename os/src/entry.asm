# 操作系统启动时所需的指令以及字段
#
# 我们在 linker.ld 中将程序入口设置为了 _start，因此在这里我们将填充这个标签
# 它将会执行一些必要操作，然后跳转至我们用 rust 编写的入口函数
#
# 关于 RISC-V 下的汇编语言，可以参考 https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md
# %hi 表示取 [12,32) 位，%lo 表示取 [0,12) 位

    .section .text.entry
    .globl _start
# 目前 _start 的功能：将预留的栈空间写入 $sp，然后跳转至 rust_main
_start:
    # 通过线性映射关系计算 boot_page_table 的物理页号
    lui t0, %hi(boot_page_table)
    li t1, 0xffffffff00000000
    sub t0, t0, t1
    srli t0, t0, 12
    # 8 << 60 是 satp 中使用 Sv39 模式的记号
    li t1, (8 << 60)
    or t0, t0, t1
    # 写入 satp 并更新 TLB
    csrw satp, t0
    sfence.vma

    # 加载栈的虚拟地址
    lui sp, %hi(boot_stack_top)
    addi sp, sp, %lo(boot_stack_top)
    # 跳转至 rust_main
    # 这里同时伴随 hart 和 dtb_pa 两个指针的传入（是 OpenSBI 帮我们完成的）
    lui t0, %hi(rust_main)
    addi t0, t0, %lo(rust_main)
    jr t0

    # 回忆：bss 段是 ELF 文件中只记录长度，而全部初始化为 0 的一段内存空间
    # 这里声明字段 .bss.stack 作为操作系统启动时的栈
    .section .bss.stack
    .global boot_stack
boot_stack:
    # 16K 启动栈大小
    .space 4096 * 16
    .global boot_stack_top
boot_stack_top:
    # 栈结尾

    # 初始内核映射所用的页表
    .section .data
    .align 12
    .global boot_page_table
boot_page_table:
    # .8byte表示长度为8个字节的整数
    .8byte 0
    .8byte 0
    # 第 2 项：0x8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    .8byte (0x80000 << 10) | 0xcf
    .zero 505 * 8
    # 第 508 项（外设用）：0xffff_ffff_0000_0000 -> 0x0000_0000，0xcf 表示 VRWXAD 均为 1
    .8byte (0x00000 << 10) | 0xcf
    .8byte 0
    # 第 510 项：0xffff_ffff_8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    .8byte (0x80000 << 10) | 0xcf
    .8byte 0
