# 使用 GDB 对 rCore 进行 debug*

> *：使用 GDB 调试可以方便观察 rCore 运行过程，但不属于教程要求

GDB 需要支持 riscv64 架构才能够对 rCore 进行 debug。

- 运行 `gdb --configuration` 来查看本地的 GDB 支持什么架构，其中 `--target` 参数指定了它可以 debug 的架构
- 如果 `gdb` 不支持，可以按照下面的步骤来安装 `riscv64-unknown-elf-gdb`

## 安装 `riscv64-unknown-elf-gdb`

0.  安装依赖（针对 Linux，macOS 可以遇到错误再去搜索）
    - python 并非必须
    - 在 `Ubuntu 20.04` 等系统中，`python` 和 `python-dev` 需要替换成 `python2` 和 `python2-dev`
    ```bash
    sudo apt-get install libncurses5-dev python python-dev texinfo libreadline-dev
    ```

1.  前往[清华镜像](https://mirrors.tuna.tsinghua.edu.cn/gnu/gdb/?C=M&O=D)下载最新的 GDB 源代码

2.  解压源代码，并定位到目录

3.  执行以下命令
    - `--prefix` 是安装的路径，按照以上指令会安装到 `/usr/local/bin/` 下
    - `--with-python` 是 `python2` 的地址，它和 `--enable-tui` 都是为了支持后续安装一个可视化插件，并非必须
    ```bash
    mkdir build
    cd build
    ../configure --prefix=/usr/local --with-python=/usr/bin/python --target=riscv64-unknown-elf --enable-tui=yes
    ```

4.  编译安装

    ```bash
    # Linux
    make -j$(nproc)
    # macOS
    make -j$(sysctl -n hw.ncpu)

    sudo make install
    ```

5.  （可选）安装 [`gdb-dashboard`](https://github.com/cyrus-and/gdb-dashboard/) 插件，优化 debug 体验
    ```bash
    wget -P ~ https://git.io/.gdbinit
    ```

## 使用 GDB 对 rCore 进行 debug

在 `os/Makefile` 中，包含了 `debug` 方法，可以执行 `make debug` 来在 `tmux` 中开启调试。

手动：

- 将 QEMU 的运行参数加上 `-s -S`，它将在 1234 端口等待调试器接入
- 运行 `riscv64-unknown-elf-gdb`
- 在 GDB 中执行 `file target/riscv64imac-unknown-none-elf/debug/os` 来加载未被 `strip` 过的内核文件中的各种符号
- 在 GDB 中执行 `target remote localhost:1234` 来连接 QEMU，开始调试

## GDB 简单使用方法

### 控制流

- `b <函数名>` 在函数进入时设置断点，例如 `b rust_main` 或 `b os::memory::heap::init`
- `cont` 继续执行
- `n` 执行下一行代码，不进入函数
- `ni` 执行下一条指令（跳转指令则执行至返回）
- `s` 执行下一行代码，进入函数
- `si` 执行下一条指令，包括跳转指令

### 查看状态

- 如果没有安装 `gdb-dashboard`，可以通过 `layout` 指令来呈现寄存器等信息，具体查看 `help layout`
- 使用 `x/<格式> <地址>` 来查看内存，例如 `x/8i 0x80200000` 表示查看 `0x80200000` 起始的 8 条指令。具体格式查看 `help x`

## 注意

调试虚实地址转换时，GDB 完全通过读取文件来判断函数地址，因此可能会遇到一些问题，需要手动设置地址来调试
