MEMORY
{
	FLASH (rx)    : ORIGIN = 0x00080000, LENGTH = 0x00080000 /* Flash, 512K */
	sram0 (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00010000 /* sram0, 64K */
	sram1 (rwx) : ORIGIN = 0x20080000, LENGTH = 0x00008000 /* sram1, 32K */
	RAM (rwx)   : ORIGIN = 0x20070000, LENGTH = 0x00018000 /* sram, 96K */
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
