MEMORY
{
  FLASH (rx)            : ORIGIN = 0x00000000, LENGTH = 0x00400000    /* 4MB  ZBTSRAM1 */
  RAM (rwx)             : ORIGIN = 0x20000000, LENGTH = 0x00400000    /* 4MB  ZBTSRAM2 & ZBTSRAM3 */
}

/* INCLUDE "demo/common.x" */
