MEMORY
{
  /* STM32F103C8 имеет 64K FLASH и 20K RAM */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}

/* Корректно расположить секции в памяти */
SECTIONS
{
  /* ... */
}