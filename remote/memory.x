/* Pro Micro nRF52840 (nice!nano v2 clone) with the Adafruit UF2 bootloader.
   The MBR and SoftDevice S140 v6.1.1 slot occupy the flash below 0x26000,
   and the bootloader starts at 0xF4000. The MBR reserves the first 8 bytes of RAM. */
MEMORY
{
  FLASH : ORIGIN = 0x00026000, LENGTH = 0xF4000 - 0x26000
  RAM   : ORIGIN = 0x20000008, LENGTH = 256K - 8
}
