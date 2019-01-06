SECTIONS {
  . = 0x4000000; /* bootloader start; leave ~64MiB free */

  /* start of the binary */
  _start = .;

  .text : {
      KEEP(*(.text.init)) /* from init.S */
      *(.text .text.* .gnu.linkonce.t*)
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

  /* end of the binary */
  _end = ALIGN(8);

  /* number of bytes in BSS section and complete binary */
  __bss_length = (__bss_end - __bss_start);
  __binary_length = (_end - _start);

  /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
