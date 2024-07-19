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

}
