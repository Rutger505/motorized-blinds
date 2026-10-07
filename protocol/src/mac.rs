//! Minimal IEEE 802.15.4 MAC framing: data frames between short addresses on
//! one PAN, and their acknowledgements. The radio hardware adds and checks the
//! FCS, so it is not part of these buffers.

use crate::{Frame, PAN_ID};

const FRAME_TYPE_MASK: u16 = 0b111;
const FRAME_TYPE_DATA: u16 = 0b001;
const FRAME_TYPE_ACK: u16 = 0b010;
const ACK_REQUEST: u16 = 1 << 5;
const PAN_ID_COMPRESSION: u16 = 1 << 6;
const DESTINATION_SHORT: u16 = 0b10 << 10;
const SOURCE_SHORT: u16 = 0b10 << 14;

const DATA_FRAME_CONTROL: u16 =
    FRAME_TYPE_DATA | ACK_REQUEST | PAN_ID_COMPRESSION | DESTINATION_SHORT | SOURCE_SHORT;

pub const DATA_HEADER_LEN: usize = 9;
pub const DATA_FRAME_LEN: usize = DATA_HEADER_LEN + Frame::LEN;

pub fn data_frame(
    sequence: u8,
    destination: u16,
    source: u16,
    payload: [u8; Frame::LEN],
) -> [u8; DATA_FRAME_LEN] {
    let mut buffer = [0; DATA_FRAME_LEN];
    buffer[0..2].copy_from_slice(&DATA_FRAME_CONTROL.to_le_bytes());
    buffer[2] = sequence;
    buffer[3..5].copy_from_slice(&PAN_ID.to_le_bytes());
    buffer[5..7].copy_from_slice(&destination.to_le_bytes());
    buffer[7..9].copy_from_slice(&source.to_le_bytes());
    buffer[DATA_HEADER_LEN..].copy_from_slice(&payload);
    buffer
}

pub fn is_ack_for(psdu: &[u8], sequence: u8) -> bool {
    let [low, high, ack_sequence, ..] = *psdu else {
        return false;
    };
    let frame_control = u16::from_le_bytes([low, high]);
    frame_control & FRAME_TYPE_MASK == FRAME_TYPE_ACK && ack_sequence == sequence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BLIND_ADDRESSES, Command, REMOTE_ADDRESS};
    use byte::TryRead;
    use ieee802154::mac::{
        Address, FooterMode, Frame as MacFrame, FrameType, FrameVersion, PanId, ShortAddress,
    };

    fn payload() -> [u8; Frame::LEN] {
        Frame {
            command: Command::Omlaag,
            sequence: 42,
            battery: 90,
        }
        .to_bytes()
    }

    #[test]
    fn data_frame_has_the_expected_bytes() {
        let bytes = data_frame(42, BLIND_ADDRESSES[1], REMOTE_ADDRESS, payload());
        assert_eq!(
            bytes,
            [
                0x61, 0x88, 42, 0x1D, 0xB1, 0x02, 0x00, 0x10, 0x00, 0x02, 42, 90
            ]
        );
    }

    /// esp-radio decodes received frames with the `ieee802154` crate, so the
    /// blind modules only understand the remote if that crate accepts them.
    #[test]
    fn data_frame_decodes_like_esp_radio_does() {
        let bytes = data_frame(7, BLIND_ADDRESSES[0], REMOTE_ADDRESS, payload());
        let (frame, _) = MacFrame::try_read(&bytes[..], FooterMode::None).unwrap();

        assert_eq!(frame.header.frame_type, FrameType::Data);
        assert_eq!(frame.header.version, FrameVersion::Ieee802154_2003);
        assert!(frame.header.ack_request);
        assert_eq!(frame.header.seq, 7);
        assert_eq!(
            frame.header.destination,
            Some(Address::Short(
                PanId(PAN_ID),
                ShortAddress(BLIND_ADDRESSES[0])
            ))
        );
        assert_eq!(
            frame.header.source,
            Some(Address::Short(PanId(PAN_ID), ShortAddress(REMOTE_ADDRESS)))
        );
        assert_eq!(Frame::parse(frame.payload), Frame::parse(&payload()));
    }

    #[test]
    fn recognises_an_ack_for_the_sent_sequence() {
        assert!(is_ack_for(&[0x02, 0x00, 42], 42));
        assert!(is_ack_for(&[0x12, 0x00, 42], 42), "frame pending bit set");
    }

    #[test]
    fn rejects_other_acks_and_frames() {
        assert!(!is_ack_for(&[0x02, 0x00, 41], 42));
        assert!(!is_ack_for(&[0x61, 0x88, 42], 42));
        assert!(!is_ack_for(&[0x02, 0x00], 42));
    }
}
