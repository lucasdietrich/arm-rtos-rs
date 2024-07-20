MEMORY
{
  FLASH (rx)            : ORIGIN = 0x00000000, LENGTH = 0x00400000    /* 4MB  ZBTSRAM1 */
  RAM (rwx)             : ORIGIN = 0x20000000, LENGTH = 0x00400000    /* 4MB  ZBTSRAM2 & ZBTSRAM3 */
}

/* Instruct the linker to keep this symbol in the final binary */
ENTRY(_reset_handler)

EXTERN(RESET_VECTOR);

PROVIDE(_stack_top = ORIGIN(RAM) + LENGTH(RAM));

SECTIONS
{
  .isr_vector ORIGIN(FLASH) :
  {
    /* reset vector */
    KEEP(*(.vector_table));
    
     . = ALIGN(4);
  } > FLASH

  .text :
  {
    *(.text .text.*);
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  .bss :
  {
    . = ALIGN(4);
    _sbss = .;
    *(.bss .bss.*);
    _ebss = .;
  } > RAM

  .data :
  {
    PROVIDE(_sidata = LOADADDR(.data)); /* Address in RAM of the data section */

    . = ALIGN(4);
    _sdata = .;
    *(.data .data.*);
    _edata = .;
  } > RAM AT > FLASH /* Reside in FLASH, loaded to RAM */

  /DISCARD/ :
  {
    /* We don't do stack unwinding, so discard the exception handling sections */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);

    /* TODO required ? */
    *(.eh_frame*);
  }
}
