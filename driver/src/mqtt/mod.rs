//! Home Assistant integration over MQTT.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum CoverState {
    Open,
    Closed,
    Opening,
    Closing,
    Stopped,
}

impl CoverState {
    pub const fn as_str(self) -> &'static str {
        match self {
            CoverState::Open => "open",
            CoverState::Closed => "closed",
            CoverState::Opening => "opening",
            CoverState::Closing => "closing",
            CoverState::Stopped => "stopped",
        }
    }
}

/// `position` follows the Home Assistant convention: 100 is fully open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub struct Status {
    pub state: CoverState,
    pub position: u8,
}
