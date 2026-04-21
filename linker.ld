ENTRY(_start);

SECTIONS
{
  . = 0x40200000;

  . = ALIGN(4K);
  .text : {
    KEEP(*(.text.boot))
    *(.text .text.*)
  }

  . = ALIGN(4K);
  .rodata : {
    *(.rodata .rodata.*)
  }

  . = ALIGN(4K);
  .data : {
    *(.data .data.*)
  }

  . = ALIGN(16);
  __bss_start = .;
  .bss (NOLOAD) : {
    *(.bss .bss.*)
    *(COMMON)
  }
  . = ALIGN(16);
  __bss_end = .;

  . = ALIGN(16);
  __boot_stack_bottom = .;
  .stack (NOLOAD) : {
    . = . + 64K;
  }
  . = ALIGN(16);
  __boot_stack_top = .;

  /DISCARD/ : {
    *(.comment*)
    *(.eh_frame*)
    *(.note*)
    *(.got*)
  }
}
