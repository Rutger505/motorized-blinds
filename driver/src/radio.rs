//! Receiving the remote's frames over IEEE 802.15.4.

use protocol::Frame;

/// The remote resends a frame with the same sequence number when the ack
/// was lost, so that frame must not run twice.
pub fn is_new(last_sequence: &mut Option<u8>, frame: &Frame) -> bool {
    if *last_sequence == Some(frame.sequence) {
        return false;
    }
    *last_sequence = Some(frame.sequence);
    true
}

#[cfg(all(target_os = "none", feature = "radio"))]
pub use hardware::Receiver;

#[cfg(all(target_os = "none", feature = "radio"))]
mod hardware {
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    use esp_radio::ieee802154::{Config, Ieee802154};
    use protocol::{CHANNEL, Frame, PAN_ID};

    static FRAME_RECEIVED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

    fn on_frame_received() {
        FRAME_RECEIVED.signal(());
    }

    pub struct Receiver<'a> {
        radio: Ieee802154<'a>,
        last_sequence: Option<u8>,
    }

    impl<'a> Receiver<'a> {
        /// The radio hardware filters on PAN ID and address, and sends the
        /// acks by itself.
        pub fn new(mut radio: Ieee802154<'a>, address: u16) -> Self {
            radio.set_config(Config {
                auto_ack_rx: true,
                rx_when_idle: true,
                channel: CHANNEL,
                pan_id: Some(PAN_ID),
                short_addr: Some(address),
                ..Config::default()
            });
            radio.set_rx_available_callback_fn(on_frame_received);
            radio.start_receive();
            Self {
                radio,
                last_sequence: None,
            }
        }

        pub async fn receive(&mut self) -> Frame {
            loop {
                let Some(received) = self.radio.received() else {
                    FRAME_RECEIVED.wait().await;
                    continue;
                };

                let frame = match received {
                    Ok(received) => Frame::parse(&received.frame.payload),
                    Err(error) => {
                        defmt::warn!("Undecodable 802.15.4 frame: {}", error);
                        continue;
                    }
                };
                match frame {
                    Ok(frame) if super::is_new(&mut self.last_sequence, &frame) => return frame,
                    Ok(frame) => defmt::debug!("Ignoring resent frame {}", frame.sequence),
                    Err(error) => defmt::warn!("Invalid frame from the remote: {}", error),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::Command;

    fn frame(sequence: u8) -> Frame {
        Frame {
            command: Command::Omhoog,
            sequence,
            battery: 80,
        }
    }

    #[test]
    fn a_resent_frame_is_dropped() {
        let mut last_sequence = None;
        assert!(is_new(&mut last_sequence, &frame(5)));
        assert!(!is_new(&mut last_sequence, &frame(5)));
        assert!(!is_new(&mut last_sequence, &frame(5)));
        assert!(is_new(&mut last_sequence, &frame(6)));
    }

    #[test]
    fn the_sequence_may_wrap_around() {
        let mut last_sequence = Some(255);
        assert!(is_new(&mut last_sequence, &frame(0)));
    }
}
