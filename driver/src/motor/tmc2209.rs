//! TMC2209 stepper driver: configured over its single wire UART, driven with
//! STEP, DIR and EN.

const SYNC: u8 = 0x05;
const WRITE: u8 = 0x80;
const MASTER_ADDRESS: u8 = 0xFF;
/// MS1 and MS2 are tied to GND.
pub const NODE_ADDRESS: u8 = 0;

pub const GCONF: u8 = 0x00;
pub const IFCNT: u8 = 0x02;
pub const IHOLD_IRUN: u8 = 0x10;
pub const CHOPCONF: u8 = 0x6C;

/// The sense resistors on the common TMC2209 modules.
pub const SENSE_RESISTOR_MILLIOHM: u32 = 110;

const GCONF_PDN_DISABLE: u32 = 1 << 6;
const GCONF_MSTEP_REG_SELECT: u32 = 1 << 7;
const GCONF_MULTISTEP_FILT: u32 = 1 << 8;
const CHOPCONF_RESET: u32 = 0x1000_0053;
const CHOPCONF_VSENSE: u32 = 1 << 17;
const CHOPCONF_MRES_SHIFT: u32 = 24;
const CHOPCONF_MRES_MASK: u32 = 0xF << CHOPCONF_MRES_SHIFT;

/// TMC's CRC8 from the datasheet: polynomial 0x07, bytes fed LSB first.
pub fn crc(bytes: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &byte in bytes {
        let mut byte = byte;
        for _ in 0..8 {
            crc = if (crc >> 7) ^ (byte & 1) != 0 {
                (crc << 1) ^ 0x07
            } else {
                crc << 1
            };
            byte >>= 1;
        }
    }
    crc
}

pub fn write_datagram(register: u8, value: u32) -> [u8; 8] {
    let [b0, b1, b2, b3] = value.to_be_bytes();
    let mut datagram = [SYNC, NODE_ADDRESS, register | WRITE, b0, b1, b2, b3, 0];
    datagram[7] = crc(&datagram[..7]);
    datagram
}

pub fn read_request(register: u8) -> [u8; 4] {
    let mut request = [SYNC, NODE_ADDRESS, register, 0];
    request[3] = crc(&request[..3]);
    request
}

pub fn parse_reply(reply: &[u8; 8], register: u8) -> Option<u32> {
    let valid = reply[0] == SYNC
        && reply[1] == MASTER_ADDRESS
        && reply[2] == register
        && reply[7] == crc(&reply[..7]);
    valid.then(|| u32::from_be_bytes([reply[3], reply[4], reply[5], reply[6]]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub gconf: u32,
    pub chopconf: u32,
    pub ihold_irun: u32,
}

/// Register values for stealthChop at the given RMS current, with the
/// microstep resolution set over UART. `None` if `microsteps` isn't a power of
/// two from 1 to 256.
pub fn settings(current_ma: u16, microsteps: u16) -> Option<Settings> {
    let mres = microstep_resolution(microsteps)?;
    let (current_scale, vsense) = current_scale(current_ma, SENSE_RESISTOR_MILLIOHM);

    let mut chopconf = CHOPCONF_RESET & !CHOPCONF_MRES_MASK;
    chopconf |= u32::from(mres) << CHOPCONF_MRES_SHIFT;
    if vsense {
        chopconf |= CHOPCONF_VSENSE;
    }

    let irun = u32::from(current_scale);
    let ihold = irun / 2;
    let ihold_delay = 1;

    Some(Settings {
        gconf: GCONF_PDN_DISABLE | GCONF_MSTEP_REG_SELECT | GCONF_MULTISTEP_FILT,
        chopconf,
        ihold_irun: ihold | irun << 8 | ihold_delay << 16,
    })
}

fn microstep_resolution(microsteps: u16) -> Option<u8> {
    (microsteps.is_power_of_two() && microsteps <= 256).then(|| 8 - microsteps.ilog2() as u8)
}

/// The current scale (CS) for an RMS current, from the datasheet:
/// `I_rms = (CS + 1) / 32 * V_fs / (R_sense + 20 mΩ) / √2`.
/// The more sensitive range (vsense) is used when it gives a finer CS.
fn current_scale(current_ma: u16, sense_milliohm: u32) -> (u8, bool) {
    let scale = |full_scale_mv: u64| {
        let numerator = 32 * 1414 * u64::from(current_ma) * u64::from(sense_milliohm + 20);
        let cs_plus_one = numerator / (full_scale_mv * 1_000_000);
        cs_plus_one.saturating_sub(1).min(31) as u8
    };

    match scale(325) {
        cs if cs < 16 => (scale(180), true),
        cs => (cs, false),
    }
}

#[cfg(target_os = "none")]
pub use hardware::{Error, Tmc2209};

#[cfg(target_os = "none")]
mod hardware {
    use embassy_time::{Duration, Timer, with_timeout};
    use esp_hal::Async;
    use esp_hal::delay::Delay;
    use esp_hal::gpio::Output;
    use esp_hal::uart::Uart;

    use super::*;
    use crate::motor::{Direction, Stepper};

    const REPLY_TIMEOUT: Duration = Duration::from_millis(20);
    /// The TMC2209 waits 8 bit times before replying, plus margin at 115200 baud.
    const REPLY_DELAY: Duration = Duration::from_millis(1);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
    pub enum Error {
        Uart,
        Timeout,
        BadReply,
        InvalidMicrosteps,
        /// The interface counter did not count every write, so a write was lost.
        WriteNotAccepted,
    }

    pub struct Tmc2209<'d> {
        uart: Uart<'d, Async>,
        step: Output<'d>,
        dir: Output<'d>,
        enable: Output<'d>,
        step_interval: Duration,
    }

    impl<'d> Tmc2209<'d> {
        /// EN is active low: `enable` must start high so the motor is off.
        pub fn new(
            uart: Uart<'d, Async>,
            step: Output<'d>,
            dir: Output<'d>,
            enable: Output<'d>,
            step_interval: Duration,
        ) -> Self {
            Self {
                uart,
                step,
                dir,
                enable,
                step_interval,
            }
        }

        pub async fn configure(&mut self, current_ma: u16, microsteps: u16) -> Result<(), Error> {
            let settings = settings(current_ma, microsteps).ok_or(Error::InvalidMicrosteps)?;

            let count_before = self.read(IFCNT).await?;
            self.write(GCONF, settings.gconf).await?;
            self.write(CHOPCONF, settings.chopconf).await?;
            self.write(IHOLD_IRUN, settings.ihold_irun).await?;
            let count_after = self.read(IFCNT).await?;

            if count_after.wrapping_sub(count_before) & 0xFF != 3 {
                return Err(Error::WriteNotAccepted);
            }
            Ok(())
        }

        async fn write(&mut self, register: u8, value: u32) -> Result<(), Error> {
            let datagram = write_datagram(register, value);
            self.transfer(&datagram, &mut [0; 8]).await?;
            Ok(())
        }

        async fn read(&mut self, register: u8) -> Result<u32, Error> {
            let mut received = [0; 12];
            self.transfer(&read_request(register), &mut received)
                .await?;
            let reply: &[u8; 8] = received[4..].try_into().unwrap();
            parse_reply(reply, register).ok_or(Error::BadReply)
        }

        /// TX and RX share one wire, so every sent byte is also received
        /// back before the reply.
        async fn transfer(&mut self, request: &[u8], received: &mut [u8]) -> Result<(), Error> {
            let mut stale = [0; 16];
            while self
                .uart
                .read_buffered(&mut stale)
                .map_err(|_| Error::Uart)?
                > 0
            {}

            self.uart
                .write_async(request)
                .await
                .map_err(|_| Error::Uart)?;
            self.uart.flush_async().await.map_err(|_| Error::Uart)?;

            with_timeout(REPLY_TIMEOUT, self.uart.read_exact_async(received))
                .await
                .map_err(|_| Error::Timeout)?
                .map_err(|_| Error::Uart)?;
            Timer::after(REPLY_DELAY).await;
            Ok(())
        }
    }

    impl Stepper for Tmc2209<'_> {
        fn enable(&mut self) {
            self.enable.set_low();
        }

        fn disable(&mut self) {
            self.enable.set_high();
        }

        fn set_direction(&mut self, direction: Direction) {
            match direction {
                Direction::Omhoog => self.dir.set_low(),
                Direction::Omlaag => self.dir.set_high(),
            }
        }

        /// The pulse is given before the first await, so cancelling this
        /// future never loses a step that was already counted.
        async fn step(&mut self) {
            self.step.set_high();
            Delay::new().delay_micros(2);
            self.step.set_low();
            Timer::after(self.step_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_known_read_requests() {
        assert_eq!(read_request(GCONF), [0x05, 0x00, 0x00, 0x48]);
        assert_eq!(read_request(0x06), [0x05, 0x00, 0x06, 0x6F]);
    }

    #[test]
    fn write_datagram_sets_the_write_bit_and_big_endian_data() {
        let datagram = write_datagram(CHOPCONF, 0x1400_0053);
        assert_eq!(&datagram[..7], &[0x05, 0x00, 0xEC, 0x14, 0x00, 0x00, 0x53]);
        assert_eq!(datagram[7], crc(&datagram[..7]));
    }

    #[test]
    fn parses_a_valid_reply() {
        let mut reply = [0x05, 0xFF, IFCNT, 0x00, 0x00, 0x00, 0x07, 0x00];
        reply[7] = crc(&reply[..7]);
        assert_eq!(parse_reply(&reply, IFCNT), Some(7));
    }

    #[test]
    fn rejects_a_corrupt_or_unexpected_reply() {
        let mut reply = [0x05, 0xFF, IFCNT, 0x00, 0x00, 0x00, 0x07, 0x00];
        reply[7] = crc(&reply[..7]);
        assert_eq!(parse_reply(&reply, GCONF), None);

        let mut corrupt = reply;
        corrupt[6] ^= 1;
        assert_eq!(parse_reply(&corrupt, IFCNT), None);
    }

    #[test]
    fn microstep_resolution_follows_the_mres_table() {
        assert_eq!(microstep_resolution(256), Some(0));
        assert_eq!(microstep_resolution(16), Some(4));
        assert_eq!(microstep_resolution(1), Some(8));
        assert_eq!(microstep_resolution(12), None);
        assert_eq!(microstep_resolution(512), None);
        assert_eq!(microstep_resolution(0), None);
    }

    #[test]
    fn current_scale_matches_the_datasheet_formula() {
        assert_eq!(current_scale(800, 110), (25, true));
        assert_eq!(current_scale(1500, 110), (26, false));
        assert_eq!(current_scale(3000, 110), (31, false));
        assert_eq!(current_scale(0, 110), (0, true));
    }

    #[test]
    fn settings_enable_uart_control_and_stealthchop() {
        let settings = settings(800, 16).unwrap();
        assert_eq!(settings.gconf, 0x1C0);
        assert_eq!(settings.chopconf, 0x1402_0053);
        assert_eq!(settings.ihold_irun, 0x0001_190C);
        assert_eq!(super::settings(800, 3), None);
    }
}
