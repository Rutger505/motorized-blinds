//! Sending commands to the blinds over IEEE 802.15.4.

use core::future::Future;

use protocol::{Frame, MAX_RETRIES, REMOTE_ADDRESS, mac};

use crate::knoppen::Message;

/// VDDH of the nRF52840 works down to 2.5 V, a fresh CR2032 gives about 3 V.
const EMPTY_MILLIVOLTS: u32 = 2500;
const FULL_MILLIVOLTS: u32 = 3000;

pub fn battery_percent(millivolts: u32) -> u8 {
    let above_empty = millivolts.clamp(EMPTY_MILLIVOLTS, FULL_MILLIVOLTS) - EMPTY_MILLIVOLTS;
    (above_empty * 100 / (FULL_MILLIVOLTS - EMPTY_MILLIVOLTS)) as u8
}

pub trait Link {
    /// Transmits one MAC frame, and returns whether its ack came back.
    fn transmit(&mut self, psdu: &[u8], sequence: u8) -> impl Future<Output = bool>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum SendError {
    NoAck,
}

pub struct Sender<L> {
    radio: L,
    sequence: u8,
}

impl<L: Link> Sender<L> {
    /// A random `first_sequence` keeps a blind from dropping the first
    /// command after the remote restarts as a resend.
    pub fn new(radio: L, first_sequence: u8) -> Self {
        Self {
            radio,
            sequence: first_sequence,
        }
    }

    /// Resends the same frame up to `MAX_RETRIES` times while no ack comes.
    pub async fn send(&mut self, message: Message, battery: u8) -> Result<(), SendError> {
        let sequence = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);

        let frame = Frame {
            command: message.command,
            sequence,
            battery,
        };
        let psdu = mac::data_frame(sequence, message.blind, REMOTE_ADDRESS, frame.to_bytes());

        for _ in 0..=MAX_RETRIES {
            if self.radio.transmit(&psdu, sequence).await {
                return Ok(());
            }
        }
        Err(SendError::NoAck)
    }
}

#[cfg(target_os = "none")]
pub use hardware::{Battery, Hfxo, Led, NrfLink};

#[cfg(target_os = "none")]
mod hardware {
    use embassy_nrf::gpio::Output;
    use embassy_nrf::pac;
    use embassy_nrf::radio::ieee802154::{Packet, Radio};
    use embassy_nrf::saadc::Saadc;
    use embassy_time::{Duration, Timer, with_timeout};
    use protocol::{CHANNEL, mac};

    use super::{Link, battery_percent};

    /// macAckWaitDuration is 864 µs; the rest is margin for waking up.
    const ACK_TIMEOUT: Duration = Duration::from_millis(2);
    const RETRY_DELAY: Duration = Duration::from_millis(5);

    pub struct NrfLink<'d> {
        radio: Radio<'d>,
        packet: Packet,
        ack: Packet,
    }

    impl<'d> NrfLink<'d> {
        pub fn new(mut radio: Radio<'d>) -> Self {
            radio.set_channel(CHANNEL);
            radio.set_transmission_power(8);
            Self {
                radio,
                packet: Packet::new(),
                ack: Packet::new(),
            }
        }

        async fn wait_for_ack(&mut self, sequence: u8) {
            loop {
                if self.radio.receive(&mut self.ack).await.is_ok()
                    && mac::is_ack_for(&self.ack, sequence)
                {
                    return;
                }
            }
        }
    }

    impl Link for NrfLink<'_> {
        async fn transmit(&mut self, psdu: &[u8], sequence: u8) -> bool {
            self.packet.copy_from_slice(psdu);
            let acked = match self.radio.try_send(&mut self.packet).await {
                Ok(()) => with_timeout(ACK_TIMEOUT, self.wait_for_ack(sequence))
                    .await
                    .is_ok(),
                Err(error) => {
                    defmt::debug!("Not sent: {}", error);
                    false
                }
            };
            if !acked {
                Timer::after(RETRY_DELAY).await;
            }
            acked
        }
    }

    /// The radio needs the external 32 MHz crystal, which draws far too much
    /// current for a coin cell to keep running between button presses.
    pub struct Hfxo;

    impl Hfxo {
        pub fn start() -> Self {
            let clock = pac::CLOCK;
            clock.events_hfclkstarted().write_value(0);
            clock.tasks_hfclkstart().write_value(1);
            while clock.events_hfclkstarted().read() == 0 {}
            Self
        }
    }

    impl Drop for Hfxo {
        fn drop(&mut self) {
            pac::CLOCK.tasks_hfclkstop().write_value(1);
        }
    }

    pub struct Battery<'d> {
        saadc: Saadc<'d, 1>,
    }

    impl<'d> Battery<'d> {
        /// `saadc` must sample VDDH/5 with the default gain of 1/6 and the
        /// internal 0.6 V reference, so a 12 bit reading spans 0 to 3.6 V.
        pub async fn new(saadc: Saadc<'d, 1>) -> Self {
            saadc.calibrate().await;
            Self { saadc }
        }

        pub async fn percent(&mut self) -> u8 {
            let mut sample = [0];
            self.saadc.sample(&mut sample).await;
            let divided_millivolts = u32::from(sample[0].max(0) as u16) * 3600 / 4096;
            battery_percent(divided_millivolts * 5)
        }
    }

    pub struct Led<'d> {
        pin: Output<'d>,
    }

    impl<'d> Led<'d> {
        pub fn new(pin: Output<'d>) -> Self {
            Self { pin }
        }

        pub async fn blink(&mut self) {
            for _ in 0..3 {
                self.pin.set_high();
                Timer::after_millis(150).await;
                self.pin.set_low();
                Timer::after_millis(150).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;
    use embassy_futures::block_on;
    use protocol::{BLIND_ADDRESSES, Command};

    /// Acks from the given attempt on, counting from 1.
    struct FakeLink {
        acked_from_attempt: Option<usize>,
        sent: Vec<(Vec<u8>, u8)>,
    }

    impl Link for &mut FakeLink {
        async fn transmit(&mut self, psdu: &[u8], sequence: u8) -> bool {
            self.sent.push((psdu.to_vec(), sequence));
            self.acked_from_attempt
                .is_some_and(|attempt| self.sent.len() >= attempt)
        }
    }

    fn link(acked_from_attempt: Option<usize>) -> FakeLink {
        FakeLink {
            acked_from_attempt,
            sent: Vec::new(),
        }
    }

    const MESSAGE: Message = Message {
        blind: BLIND_ADDRESSES[1],
        command: Command::BovenOpslaan,
    };

    #[test]
    fn battery_percent_maps_the_coin_cell_range() {
        assert_eq!(battery_percent(3200), 100);
        assert_eq!(battery_percent(3000), 100);
        assert_eq!(battery_percent(2750), 50);
        assert_eq!(battery_percent(2500), 0);
        assert_eq!(battery_percent(1800), 0);
    }

    #[test]
    fn sends_the_frame_to_the_blind() {
        let mut link = link(Some(1));
        let mut sender = Sender::new(&mut link, 7);
        assert_eq!(block_on(sender.send(MESSAGE, 64)), Ok(()));

        let expected = mac::data_frame(
            7,
            BLIND_ADDRESSES[1],
            REMOTE_ADDRESS,
            [Command::BovenOpslaan.to_byte().unwrap(), 7, 64],
        );
        assert_eq!(link.sent, [(expected.to_vec(), 7)]);
    }

    #[test]
    fn retries_with_the_same_sequence_until_acked() {
        let mut link = link(Some(3));
        let mut sender = Sender::new(&mut link, 200);
        assert_eq!(block_on(sender.send(MESSAGE, 100)), Ok(()));

        assert_eq!(link.sent.len(), 3);
        assert!(
            link.sent
                .iter()
                .all(|(psdu, sequence)| { *sequence == 200 && *psdu == link.sent[0].0 })
        );
    }

    #[test]
    fn gives_up_after_three_retries() {
        let mut link = link(None);
        let mut sender = Sender::new(&mut link, 0);
        assert_eq!(block_on(sender.send(MESSAGE, 100)), Err(SendError::NoAck));
        assert_eq!(link.sent.len(), 1 + usize::from(MAX_RETRIES));
    }

    #[test]
    fn every_message_gets_the_next_sequence() {
        let mut link = link(Some(1));
        let mut sender = Sender::new(&mut link, 255);
        for _ in 0..3 {
            block_on(sender.send(MESSAGE, 100)).unwrap();
        }
        let sequences: Vec<_> = link.sent.iter().map(|(_, sequence)| *sequence).collect();
        assert_eq!(sequences, [255, 0, 1]);
    }
}
