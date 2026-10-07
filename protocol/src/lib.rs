//! Shared radio protocol between the remote and the blind modules.

#![no_std]

pub mod mac;

pub const PAN_ID: u16 = 0xB11D;
pub const REMOTE_ADDRESS: u16 = 0x0010;
pub const BLIND_ADDRESSES: [u16; 2] = [0x0001, 0x0002];
pub const CHANNEL: u8 = 15;
pub const MAX_RETRIES: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Omhoog,
    Omlaag,
    NaarAangepast,
    Stop,
    KalibratieStart,
    KalibratieStop,
    BeweegOmhoog,
    BeweegOmlaag,
    BeweegStop,
    BovenOpslaan,
    AangepastOpslaan,
    OnderOpslaan,
}

impl Command {
    /// `Stop` only comes from Home Assistant, so it never goes over the radio.
    pub const fn to_byte(self) -> Option<u8> {
        Some(match self {
            Command::Omhoog => 0x01,
            Command::Omlaag => 0x02,
            Command::NaarAangepast => 0x03,
            Command::KalibratieStart => 0x04,
            Command::KalibratieStop => 0x05,
            Command::BeweegOmhoog => 0x06,
            Command::BeweegOmlaag => 0x07,
            Command::BeweegStop => 0x08,
            Command::BovenOpslaan => 0x09,
            Command::AangepastOpslaan => 0x0A,
            Command::OnderOpslaan => 0x0B,
            Command::Stop => return None,
        })
    }

    pub const fn from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            0x01 => Command::Omhoog,
            0x02 => Command::Omlaag,
            0x03 => Command::NaarAangepast,
            0x04 => Command::KalibratieStart,
            0x05 => Command::KalibratieStop,
            0x06 => Command::BeweegOmhoog,
            0x07 => Command::BeweegOmlaag,
            0x08 => Command::BeweegStop,
            0x09 => Command::BovenOpslaan,
            0x0A => Command::AangepastOpslaan,
            0x0B => Command::OnderOpslaan,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FrameError {
    WrongLength(usize),
    UnknownCommand(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Frame {
    pub command: Command,
    pub sequence: u8,
    pub battery: u8,
}

impl Frame {
    pub const LEN: usize = 3;

    pub fn parse(payload: &[u8]) -> Result<Frame, FrameError> {
        let &[command, sequence, battery] = payload else {
            return Err(FrameError::WrongLength(payload.len()));
        };
        let command = Command::from_byte(command).ok_or(FrameError::UnknownCommand(command))?;
        Ok(Frame {
            command,
            sequence,
            battery,
        })
    }

    /// A `Stop` command encodes as 0x00, which `parse` rejects as unknown.
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        [
            self.command.to_byte().unwrap_or(0x00),
            self.sequence,
            self.battery,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_COMMANDS: [Command; 12] = [
        Command::Omhoog,
        Command::Omlaag,
        Command::NaarAangepast,
        Command::Stop,
        Command::KalibratieStart,
        Command::KalibratieStop,
        Command::BeweegOmhoog,
        Command::BeweegOmlaag,
        Command::BeweegStop,
        Command::BovenOpslaan,
        Command::AangepastOpslaan,
        Command::OnderOpslaan,
    ];

    #[test]
    fn command_bytes_match_the_design() {
        assert_eq!(Command::Omhoog.to_byte(), Some(0x01));
        assert_eq!(Command::NaarAangepast.to_byte(), Some(0x03));
        assert_eq!(Command::BeweegStop.to_byte(), Some(0x08));
        assert_eq!(Command::OnderOpslaan.to_byte(), Some(0x0B));
        assert_eq!(Command::Stop.to_byte(), None);
    }

    #[test]
    fn every_radio_command_round_trips() {
        for command in ALL_COMMANDS {
            if let Some(byte) = command.to_byte() {
                assert_eq!(Command::from_byte(byte), Some(command));
            }
        }
    }

    #[test]
    fn frame_round_trips() {
        let frame = Frame {
            command: Command::BovenOpslaan,
            sequence: 200,
            battery: 87,
        };
        assert_eq!(frame.to_bytes(), [0x09, 200, 87]);
        assert_eq!(Frame::parse(&frame.to_bytes()), Ok(frame));
    }

    #[test]
    fn parse_rejects_wrong_length() {
        assert_eq!(Frame::parse(&[1, 2]), Err(FrameError::WrongLength(2)));
        assert_eq!(Frame::parse(&[1, 2, 3, 4]), Err(FrameError::WrongLength(4)));
    }

    #[test]
    fn parse_rejects_unknown_command() {
        assert_eq!(
            Frame::parse(&[0x00, 1, 1]),
            Err(FrameError::UnknownCommand(0))
        );
        assert_eq!(
            Frame::parse(&[0x0C, 1, 1]),
            Err(FrameError::UnknownCommand(0x0C))
        );
    }

    #[test]
    fn stop_frame_is_not_accepted_over_the_radio() {
        let frame = Frame {
            command: Command::Stop,
            sequence: 1,
            battery: 100,
        };
        assert!(Frame::parse(&frame.to_bytes()).is_err());
    }
}
