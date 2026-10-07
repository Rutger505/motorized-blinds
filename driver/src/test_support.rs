//! Fakes shared by the unit tests.

extern crate std;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

pub const SECTOR: usize = 4096;

#[derive(Debug)]
pub struct FlashError;

impl NorFlashError for FlashError {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

/// Behaves like NOR flash: writing can only clear bits, erasing sets them.
pub struct RamFlash {
    pub data: std::vec::Vec<u8>,
    pub erases: usize,
}

impl RamFlash {
    pub fn new() -> Self {
        Self {
            data: std::vec![0xFF; SECTOR * 2],
            erases: 0,
        }
    }
}

impl ErrorType for &mut RamFlash {
    type Error = FlashError;
}

impl ReadNorFlash for &mut RamFlash {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), FlashError> {
        let offset = offset as usize;
        bytes.copy_from_slice(&self.data[offset..offset + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }
}

impl NorFlash for &mut RamFlash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = SECTOR;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), FlashError> {
        self.data[from as usize..to as usize].fill(0xFF);
        self.erases += 1;
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), FlashError> {
        let offset = offset as usize;
        for (cell, byte) in self.data[offset..].iter_mut().zip(bytes) {
            *cell &= byte;
        }
        Ok(())
    }
}
