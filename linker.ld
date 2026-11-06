ENTRY(_start)

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    }

    . = ALIGN(0x1000);

    .rodata : {
        *(.rodata .rodata.*)
        *(.eh_frame_hdr)
        *(.eh_frame)
    }

    . = ALIGN(0x1000);

    .got : {
        *(.got .got.*)
    }
    
    . = ALIGN(0x1000);

    .limine_requests : {
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    }

    . = ALIGN(0x1000);

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
    }
}
