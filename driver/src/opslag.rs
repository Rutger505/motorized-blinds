//! Positions of the blind, kept in flash across power cycles (FE8).
//!
//! Every save appends a record instead of rewriting one, so a sector is only
//! erased once it is full. Two sectors are used in turn, so the newest record
//! survives a power loss during an erase.

use embedded_storage::nor_flash::NorFlash;

const SECTORS: u32 = 2;
const RECORD_LEN: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub struct Positions {
    pub current: i32,
    pub max: i32,
    pub custom: i32,
}

impl Positions {
    /// Makes the current position step 0, and shifts `max` and `custom` along
    /// so they keep pointing at the same height.
    pub fn set_top(&mut self) {
        let shift = self.current;
        self.current = 0;
        self.max -= shift;
        self.custom -= shift;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Record {
    counter: u32,
    positions: Positions,
}

impl Record {
    fn encode(&self) -> [u8; RECORD_LEN] {
        let mut bytes = [0; RECORD_LEN];
        bytes[0..4].copy_from_slice(&self.counter.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.positions.current.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.positions.max.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.positions.custom.to_le_bytes());
        let checksum = checksum(&bytes[..16]);
        bytes[16..20].copy_from_slice(&checksum.to_le_bytes());
        bytes
    }

    fn decode(bytes: &[u8; RECORD_LEN]) -> Option<Record> {
        let word = |at: usize| [bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]];
        if u32::from_le_bytes(word(16)) != checksum(&bytes[..16]) {
            return None;
        }
        Some(Record {
            counter: u32::from_le_bytes(word(0)),
            positions: Positions {
                current: i32::from_le_bytes(word(4)),
                max: i32::from_le_bytes(word(8)),
                custom: i32::from_le_bytes(word(12)),
            },
        })
    }
}

/// FNV-1a, which never yields the all ones of erased flash for erased data.
fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811C_9DC5, |hash, &byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

pub struct Storage<F> {
    flash: F,
    next_slot: u32,
    next_counter: u32,
}

impl<F: NorFlash> Storage<F> {
    const SLOTS_PER_SECTOR: u32 = F::ERASE_SIZE as u32 / RECORD_LEN as u32;
    const SLOTS: u32 = Self::SLOTS_PER_SECTOR * SECTORS;

    /// `flash` must hold at least two erase sectors, starting at address 0.
    pub fn new(flash: F) -> Self {
        assert!(flash.capacity() >= F::ERASE_SIZE * SECTORS as usize);
        assert!(RECORD_LEN.is_multiple_of(F::WRITE_SIZE));
        Self {
            flash,
            next_slot: 0,
            next_counter: 0,
        }
    }

    /// Returns the newest saved positions, or all zeros when nothing was saved.
    pub fn load(&mut self) -> Positions {
        let newest = (0..Self::SLOTS)
            .filter_map(|slot| Some((slot, self.read(slot)?)))
            .max_by_key(|(_, record)| record.counter);

        match newest {
            Some((slot, record)) => {
                self.next_slot = (slot + 1) % Self::SLOTS;
                self.next_counter = record.counter.wrapping_add(1);
                record.positions
            }
            None => Positions::default(),
        }
    }

    pub fn save(&mut self, positions: &Positions) -> Result<(), F::Error> {
        let record = Record {
            counter: self.next_counter,
            positions: *positions,
        };

        let slot = self.claim_erased_slot()?;
        self.flash.write(Self::address(slot), &record.encode())?;

        self.next_slot = (slot + 1) % Self::SLOTS;
        self.next_counter = self.next_counter.wrapping_add(1);
        Ok(())
    }

    /// Slots left half written by a power loss can't be programmed again
    /// until their sector is erased, so they are skipped.
    fn claim_erased_slot(&mut self) -> Result<u32, F::Error> {
        let mut slot = self.next_slot;
        loop {
            if slot.is_multiple_of(Self::SLOTS_PER_SECTOR) {
                let start = slot / Self::SLOTS_PER_SECTOR * F::ERASE_SIZE as u32;
                self.flash.erase(start, start + F::ERASE_SIZE as u32)?;
                return Ok(slot);
            }
            if self.is_erased(slot)? {
                return Ok(slot);
            }
            slot = (slot + 1) % Self::SLOTS;
        }
    }

    fn is_erased(&mut self, slot: u32) -> Result<bool, F::Error> {
        let mut bytes = [0; RECORD_LEN];
        self.flash.read(Self::address(slot), &mut bytes)?;
        Ok(bytes.iter().all(|&byte| byte == 0xFF))
    }

    fn read(&mut self, slot: u32) -> Option<Record> {
        let mut bytes = [0; RECORD_LEN];
        self.flash.read(Self::address(slot), &mut bytes).ok()?;
        Record::decode(&bytes)
    }

    fn address(slot: u32) -> u32 {
        let sector = slot / Self::SLOTS_PER_SECTOR;
        let index = slot % Self::SLOTS_PER_SECTOR;
        sector * F::ERASE_SIZE as u32 + index * RECORD_LEN as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{RamFlash, SECTOR};

    fn positions(current: i32) -> Positions {
        Positions {
            current,
            max: 5000,
            custom: 2500,
        }
    }

    #[test]
    fn set_top_shifts_max_and_custom_along() {
        let mut positions = Positions {
            current: -300,
            max: 4000,
            custom: 1000,
        };
        positions.set_top();
        assert_eq!(
            positions,
            Positions {
                current: 0,
                max: 4300,
                custom: 1300,
            }
        );
    }

    #[test]
    fn erased_flash_loads_zeros() {
        let mut flash = RamFlash::new();
        assert_eq!(Storage::new(&mut flash).load(), Positions::default());
    }

    #[test]
    fn erased_record_has_no_valid_checksum() {
        assert_eq!(Record::decode(&[0xFF; RECORD_LEN]), None);
    }

    #[test]
    fn loads_the_last_saved_positions_after_a_restart() {
        let mut flash = RamFlash::new();
        let mut storage = Storage::new(&mut flash);
        storage.load();
        for current in 0..10 {
            storage.save(&positions(current)).unwrap();
        }

        assert_eq!(Storage::new(&mut flash).load(), positions(9));
    }

    #[test]
    fn wraps_around_both_sectors() {
        let mut flash = RamFlash::new();
        let slots = Storage::<&mut RamFlash>::SLOTS as i32;
        let mut storage = Storage::new(&mut flash);
        storage.load();
        for current in 0..slots * 3 + 7 {
            storage.save(&positions(current)).unwrap();
            if current % 97 == 0 {
                storage = Storage::new(storage.flash);
                assert_eq!(storage.load(), positions(current));
            }
        }

        assert_eq!(Storage::new(&mut flash).load(), positions(slots * 3 + 6));
        assert_eq!(flash.erases, 7);
    }

    #[test]
    fn skips_a_half_written_slot() {
        let mut flash = RamFlash::new();
        let mut storage = Storage::new(&mut flash);
        storage.load();
        storage.save(&positions(1)).unwrap();
        storage.flash.data[RECORD_LEN..RECORD_LEN + 6].fill(0x00);

        let mut storage = Storage::new(&mut flash);
        assert_eq!(storage.load(), positions(1));
        storage.save(&positions(2)).unwrap();

        assert_eq!(Storage::new(&mut flash).load(), positions(2));
    }

    #[test]
    fn keeps_the_old_sector_until_the_new_one_has_a_record() {
        let mut flash = RamFlash::new();
        let slots_per_sector = Storage::<&mut RamFlash>::SLOTS_PER_SECTOR as i32;
        let mut storage = Storage::new(&mut flash);
        storage.load();
        for current in 0..slots_per_sector {
            storage.save(&positions(current)).unwrap();
        }
        storage.flash.data[SECTOR..].fill(0xFF);

        assert_eq!(
            Storage::new(&mut flash).load(),
            positions(slots_per_sector - 1)
        );
    }
}
