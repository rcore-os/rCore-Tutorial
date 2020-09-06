TARGET      := riscv64imac-unknown-none-elf
MODE        := debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/os
BIN_FILE    := target/$(TARGET)/$(MODE)/kernel.bin

BOARD		?= qemu
SBI			?= rustsbi
BOOTLOADER	:= ../bootloader/$(SBI)-$(BOARD).bin

# Run K210
K210-SERIALPORT	= /dev/ttyUSB0
K210-BURNER		= ../tools/kflash.py	

# Disassembly
DISASM		?= -D

USER_DIR    := ../user
USER_BUILD  := $(USER_DIR)/build
IMG_FILE    := $(USER_BUILD)/disk.img

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

.PHONY: doc kernel build clean qemu run env

# 默认 build 为输出二进制文件
build: $(BIN_FILE)

# 通过 Rust 文件中的注释生成文档
doc:
	@cargo doc --document-private-items

# 编译 kernel
kernel:
	@cp src/board/$(BOARD)/linker.ld src/linker.ld
	@cargo build --features "board_$(BOARD)"

# 生成 kernel 的二进制文件
$(BIN_FILE): kernel
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $@


# 查看反汇编结果
asm:
	@$(OBJDUMP) $(DISASM) $(KERNEL_FILE) | less

asm-save:
	@$(OBJDUMP) $(DISASM) $(KERNEL_FILE) > asm

# 清理编译出的文件
clean:
	@cargo clean

# 运行 QEMU
run-inner: build
ifeq ($(BOARD),qemu)
	@qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios $(BOOTLOADER) \
    		-device loader,file=$(BIN_FILE),addr=0x80200000 \
    		-drive file=$(IMG_FILE),format=qcow2,id=sfs \
    		-device virtio-blk-device,drive=sfs
else
	@cp $(BOOTLOADER) $(BOOTLOADER).copy
	@dd if=$(BIN_FILE) of=$(BOOTLOADER).copy bs=128K seek=1
	@mv $(BOOTLOADER).copy $(BIN_FILE)
	@sudo chmod 777 $(K210-SERIALPORT)
	python3 $(K210-BURNER) -p $(K210-SERIALPORT) -b 1500000 -t $(BIN_FILE)
endif

# 一键运行
run: build run-inner

# 一键 gdb
debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios default -device loader,file=$(BIN_FILE),addr=0x80200000 \
		-drive file=$(IMG_FILE),format=qcow2,id=sfs -device virtio-blk-device,drive=sfs -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_FILE)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d
