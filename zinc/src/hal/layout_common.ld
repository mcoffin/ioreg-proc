/* TODO(farcaller): layout_common isn't actually common. k20 keeps own copy */
__aeabi_unwind_cpp_pr0 = abort;
__aeabi_unwind_cpp_pr1 = abort;
__aeabi_unwind_cpp_pr2 = abort;
__exidx_start          = abort;
__exidx_end            = abort;

__aeabi_memclr4 = __aeabi_memclr;

PROVIDE(isr_systick = isr_hardfault);

SECTIONS
{
    .vector : ALIGN(4)
    {
        FILL(0xff)

        KEEP(*(.isr_vector))
        KEEP(*(.isr_vector_nvic))
    } > vectors

    .text : ALIGN(4)
    {
        FILL(0xff)
	*(.text*);
        *(.rodata .rodata.*)
    } > rom

    .data : ALIGN(4)
    {
        _data = .;

        *(SORT_BY_ALIGNMENT(.data*))
        . = ALIGN(4);

        _edata = .;
    } > ram AT>rom = 0xff

    .bss : ALIGN(4)
    {
        _bss = .;

        *(.bss*)
        *(COMMON)
        . = ALIGN(4);

        _ebss = .;

        . += 4;

        __STACK_LIMIT = .;

        . += 4;

        _eglobals = .;
    } > ram

    /DISCARD/ :
    {
        *(.glue_7*)  /* arm-thumb interworking */
        *(.v4_bx)  /* ARMv4 interworking fixup for missing BX */
        *(.vfp11_veneer)  /* VFP11 bugfixes s.a. http://sourceware.org/ml/binutils/2006-12/msg00196.html */
        *(.iplt .igot.plt)  /* STT_GNU_IFUNC symbols */
        *(.rel.*)  /* dynamic relocations */
        *(.ARM.exidx*) /* index entries for section unwinding */
        *(.ARM.extab*) /* exception unwinding information */
        *(.debug_gdb_scripts)
    }
}
