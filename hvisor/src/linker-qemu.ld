OUTPUT_ARCH(riscv)
ENTRY(arch_entry)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;


    /DISCARD/ : {
        *(.eh_frame)
    }
    . = ALIGN(4K);
	__core_end = .;
    . = . + 0x2000000;
    gdtb = .;
    .dtb1 : {
        *(.dtb1)
    }
    . = ALIGN(4K);
    .dtb2 : {
        *(.dtb2)
    }
    . = ALIGN(4K);
    . = 0x84000000;
    .img2 : {
        *(.img2)
    }
    . = 0x90000000;
    .img1 : {
        *(.img1)
    }
    
}