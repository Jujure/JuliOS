ENTRY(_start)

SECTIONS {
    . = 1M;

    .boot :
    {
        /* ensure that the multiboot header is at the beginning */
        *(.multiboot_header)
    }

    .stack :
    {
        *(.stack)
    }

    .bss :
    {
        *(.bss)
    }

    .text :
    {
        *(.text)
    }
}
