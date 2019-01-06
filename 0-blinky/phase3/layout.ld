MEMORY {
  SRAM (rwx) : ORIGIN = 0, LENGTH = 32M
}

SECTIONS {
  .text : {
    KEEP (*(.traps))
    . = 0x8000;   /* Space for command line.  */
    *(.text .text.* .gnu.linkonce.t*)
  }

  .rodata : {
    *(.rodata .rodata.* .gnu.linkonce.r*)
  }

  .data : {
    *(.data .data.* .gnu.linkonce.d*)
  }

  .bss (NOLOAD) : {
    __bss_start = ALIGN(0x10);
    *(.bss .bss.*)
    *(COMMON)

    __bss_end = ALIGN(0x10);

    . = ALIGN(0x10);
    . += 0x1000;
    __cpu0_stack_end = .;

    _end = .;
  }

   __bss_dwords = (__bss_end - __bss_start) >> 3;

  /**
   * DWARF debug sections.
   *
   * Symbols in the DWARF debugging sections are relative to the beginning of
   * the section so we begin them at 0.
  **/

  /* DWARF 1.1 and DWARF 2 */
  .debug_aranges  0 : { *(.debug_aranges) }
  .debug_pubnames 0 : { *(.debug_pubnames) }

  /* DWARF 2 */
  .debug_info     0 : { *(.debug_info .gnu.linkonce.wi.*) }
  .debug_abbrev   0 : { *(.debug_abbrev) }
  .debug_line     0 : { *(.debug_line) }
  .debug_frame    0 : { *(.debug_frame) }
  .debug_str      0 : { *(.debug_str) }
  .debug_loc      0 : { *(.debug_loc) }
  .debug_macinfo  0 : { *(.debug_macinfo) }

  /* DWARF 3 */
  .debug_pubtypes 0 : { *(.debug_pubtypes) }
  .debug_ranges   0 : { *(.debug_ranges) }

  .ARM.attributes 0 : { KEEP (*(.ARM.attributes)) KEEP (*(.gnu.attributes)) }
  .note.gnu.arm.ident 0 : { KEEP (*(.note.gnu.arm.ident)) }

  .ARM.extab : { *(.ARM.extab* .gnu.linkonce.armextab.*) }
  PROVIDE_HIDDEN (__exidx_start = .);
  .ARM.exidx : { *(.ARM.exidx* .gnu.linkonce.armexidx.*) }
  PROVIDE_HIDDEN (__exidx_end = .);

  /DISCARD/ : { *(.note.GNU-stack) *(.gnu_debuglink) *(.gnu.lto_*) }
}
