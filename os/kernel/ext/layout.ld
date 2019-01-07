SECTIONS {
  . = 0x80000; /* Raspbery Pi 3 Aarch64 (kernel8.img) load address */

  /* start of the binary */
  _start = .;

  .text : {
    PROVIDE(__text_start = .);
    KEEP(*(.text.init)) /* from init.S */
    *(.text .text.* .gnu.linkonce.t*)
    PROVIDE(__text_end = .);
  }

  .rodata : {
    *(.rodata .rodata.* .gnu.linkonce.r*)
  }

  .data : {
    *(.data .data.* .gnu.linkonce.d*)
  }

  .bss (NOLOAD) : {
    . = ALIGN(32);
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    . = ALIGN(8);
    __bss_end = .;
  }

  .debug_info : {
    . = ALIGN(32);
    PROVIDE(__debug_info_start = .);
    KEEP(*(.debug_info .debug_info.*))
    PROVIDE(__debug_info_end = .);
    BYTE(0)
  }

  .debug_abbrev : {
    . = ALIGN(32);
    PROVIDE(__debug_abbrev_start = .);
    KEEP(*(.debug_abbrev .debug_abbrev.*))
    PROVIDE(__debug_abbrev_end = .);
    BYTE(0)
  }

  .debug_str : {
    . = ALIGN(32);
    PROVIDE(__debug_str_start = .);
    KEEP(*(.debug_str .debug_str.*))
    PROVIDE(__debug_str_end = .);
    BYTE(0)
  }

  /* end of the binary */
  _end = ALIGN(8);

  /* number of bytes in BSS section and complete binary */
  __bss_length = (__bss_end - __bss_start);
  __binary_length = (_end - _start);

  /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
