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

# QEMU_ARGS := -machine virt,dumpdtb=qemu.dtb
QEMU_ARGS := -machine virt
# QEMU_ARGS := -machine virt,aclint=on
QEMU_ARGS +=-nographic 
QEMU_ARGS +=-cpu rv64 
QEMU_ARGS +=-m 3G 
#rember to change the consts MAX_CPU_NUM and dts
QEMU_ARGS +=-smp 4
QEMU_ARGS +=-bios default
# QEMU_ARGS +=-bios $(BOOTLOADER)
# QEMU_ARGS +=-kernel ../guests/Image-62 
# QEMU_ARGS +=-kernel ../guests/bao-linux.bin
QEMU_ARGS +=-kernel $(KERNEL_BIN)
# QEMU_ARGS +=-kernel ../guests/os_ch5_802.bin
# QEMU_ARGS +=-drive file=../guests/rootfs-busybox.qcow2,format=qcow2,id=hd0 
QEMU_ARGS +=-drive file=../guests/rootfs-buildroot.qcow2,format=qcow2,id=hd0 
QEMU_ARGS +=-device virtio-blk-device,drive=hd0 
# QEMU_ARGS +=-append "root=/dev/vda rw earlycon console=ttyS0"
# QEMU_ARGS +=-append "root=/dev/vda rw earlycon console=hvc0"

# QEMU_ARGS +=		 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) 
# QEMU_ARGS +=		 -device loader,file=../guests/Image-62,addr=$(KERNEL_ENTRY_PA) 
#QEMU_ARGS +=		 -device loader,file=../guests/os_ch5_802.bin,addr=0x80400000 			 
QEMU_ARGS +=		 -device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3
QEMU_ARGS +=-drive file=../guests/rootfs-busybox.qcow2,format=qcow2,id=hd1 
QEMU_ARGS +=-device virtio-blk-device,drive=hd1 


# QEMU_ARGS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
# QEMU_ARGS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
# QEMU_ARGS	+=-drive file=../guests/rCore-Tutorial-v3/fs.img,if=none,format=raw,id=x0
# QEMU_ARGS	+=-device virtio-blk-device,drive=x0
# QEMU_ARGS	+=-device virtio-gpu-device
# QEMU_ARGS	+=-device virtio-keyboard-device
# QEMU_ARGS	+=-device virtio-mouse-device
# QEMU_ARGS 	+=-device virtio-net-device,netdev=net0
# QEMU_ARGS	+=-netdev user,id=net0,hostfwd=udp::6200-:2000

run-inner: build
	@qemu-system-riscv64 $(QEMU_ARGS) -S

debug: build
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

monitor:
	gdb-multiarch \
	-ex 'target remote:1234' \
	-ex 'b *$(KERNEL_ENTRY_PA)' \
	-ex 'file $(KERNEL_ELF)' 
dtb:
	@dtc -I dts -O dtb -o ../guests/linux.dtb ../guests/linux.dts
	@dtc -I dts -O dtb -o ../guests/linux2.dtb ../guests/linux2.dts
	@dtc -I dts -O dtb -o ../guests/linux3.dtb ../guests/linux3.dts
	@dtc -I dts -O dtb -o ../guests/linux4.dtb ../guests/linux4.dts
	@dtc -I dts -O dtb -o ../guests/qemu.dtb ../guests/qemu.dts
	@dtc -I dts -O dtb -o ../guests/os_ch5.dtb ../guests/os_ch5.dts
	

.PHONY: build env kernel clean disa run-inner debug monitor dtb
