# Building
TARGET := riscv64gc-unknown-none-elf
MODE := debug
KERNEL_ELF := target/$(TARGET)/$(MODE)/hvisor
KERNEL_BIN := $(KERNEL_ELF).bin
LOG:=TRACE
export LOG
# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64


build: $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Platform: $(BOARD)
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build $(MODE_ARG)
	@rm src/linker.ld

clean:
	@cargo clean

disa:
	@$(OBJDUMP) --disassemble $(KERNEL_ELF) > hvisor.S


run: run-inner

QEMU_ARGS := -machine virt 
QEMU_ARGS +=-nographic 
QEMU_ARGS +=-cpu rv64 
QEMU_ARGS +=-m 4G 
QEMU_ARGS +=-bios $(BOOTLOADER)
#QEMU_ARGS +=-kernel ../guests/Image-62 
#QEMU_ARGS +=-drive file=../guests/rootfs-busybox.img,format=raw,id=hd0 
#QEMU_ARGS +=-device virtio-blk-device,drive=hd0 
#QEMU_ARGS +=-append "root=/dev/vda rw console=ttyS0"
#QEMU_ARGS +=-kernel $(KERNEL_BIN)
QEMU_ARGS +=		 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) 
#QEMU_ARGS +=		 -device loader,file=../guests/os_ch5_802.bin,addr=0x80400000 
			 
#QEMU_ARGS +=		 -device virtio-serial-port -chardev pty,id=serial3 -device virtconsole,chardev=serial3 \

run-inner: build
	@qemu-system-riscv64 $(QEMU_ARGS)

debug: build
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

monitor:
	gdb-multiarch \
	-ex 'target remote:1234' \
	-ex 'b *$(KERNEL_ENTRY_PA)' \
	-ex 'file $(KERNEL_ELF)' 
dtb:
	@dtc -I dts -O dtb -o ../guests/linux.dtb ../guests/linux.dts

.PHONY: build env kernel clean disa run-inner debug monitor dtb
