//! Turns button presses and combinations into commands for a blind.
//!
//! Commands that involve the calibration button are only sent once every
//! button of the column is released, because only then is it known whether
//! calibration was pressed alone or together with another button.

use heapless::Vec;
use protocol::{BLIND_ADDRESSES, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum Button {
    Omhoog,
    Omlaag,
    Aangepast,
    Kalibratie,
}

/// Calibration goes first, so pressing it together with another button
/// never starts a movement.
const PRESS_ORDER: [Button; 4] = [
    Button::Kalibratie,
    Button::Omhoog,
    Button::Omlaag,
    Button::Aangepast,
];

/// Which buttons of a column are down, indexed by `Button as usize`.
pub type ColumnState = [bool; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub struct Message {
    pub blind: u16,
    pub command: Command,
}

#[derive(Debug, Default, Clone, Copy)]
struct Column {
    calibrating: bool,
    held: ColumnState,
    /// Every button pressed since the column was last fully released.
    gesture: ColumnState,
    moving: Option<Button>,
}

impl Column {
    fn update(&mut self, now: ColumnState, emit: &mut impl FnMut(Command)) {
        for button in PRESS_ORDER {
            let index = button as usize;
            match (self.held[index], now[index]) {
                (false, true) => {
                    self.held[index] = true;
                    self.gesture[index] = true;
                    self.pressed(button, emit);
                }
                (true, false) => {
                    self.held[index] = false;
                    self.released(button, emit);
                }
                _ => {}
            }
        }

        if self.held == ColumnState::default() && self.gesture != ColumnState::default() {
            self.finish_gesture(emit);
            self.gesture = ColumnState::default();
        }
    }

    fn pressed(&mut self, button: Button, emit: &mut impl FnMut(Command)) {
        let with_calibration = self.gesture[Button::Kalibratie as usize];
        match button {
            Button::Kalibratie => {
                if self.moving.take().is_some() {
                    emit(Command::BeweegStop);
                }
            }
            _ if with_calibration => {}
            Button::Omhoog | Button::Omlaag if self.calibrating => {
                if self.moving.is_none() {
                    self.moving = Some(button);
                    emit(match button {
                        Button::Omhoog => Command::BeweegOmhoog,
                        _ => Command::BeweegOmlaag,
                    });
                }
            }
            Button::Omhoog => emit(Command::Omhoog),
            Button::Omlaag => emit(Command::Omlaag),
            Button::Aangepast if !self.calibrating => emit(Command::NaarAangepast),
            Button::Aangepast => {}
        }
    }

    fn released(&mut self, button: Button, emit: &mut impl FnMut(Command)) {
        if self.moving == Some(button) {
            self.moving = None;
            emit(Command::BeweegStop);
        }
    }

    fn finish_gesture(&mut self, emit: &mut impl FnMut(Command)) {
        let [omhoog, omlaag, aangepast, kalibratie] = self.gesture;
        if !kalibratie {
            return;
        }

        match (self.calibrating, omhoog, omlaag, aangepast) {
            (false, false, false, false) => {
                self.calibrating = true;
                emit(Command::KalibratieStart);
            }
            (true, false, false, false) => {
                self.calibrating = false;
                emit(Command::KalibratieStop);
            }
            (true, true, false, false) => emit(Command::BovenOpslaan),
            (true, false, true, false) => emit(Command::OnderOpslaan),
            (true, false, false, true) => emit(Command::AangepastOpslaan),
            _ => {}
        }
    }
}

/// The button logic of both columns, fed with debounced button states.
#[derive(Debug, Default)]
pub struct KeypadLogic {
    columns: [Column; 2],
}

impl KeypadLogic {
    pub fn update(&mut self, pressed: [ColumnState; 2]) -> Vec<Message, 8> {
        let mut messages = Vec::new();
        for ((column, now), blind) in self.columns.iter_mut().zip(pressed).zip(BLIND_ADDRESSES) {
            column.update(now, &mut |command| {
                // At most 2 commands per button per update, so 8 always fits.
                messages.push(Message { blind, command }).ok();
            });
        }
        messages
    }

    pub fn is_calibrating(&self, column: usize) -> bool {
        self.columns[column].calibrating
    }
}

#[cfg(target_os = "none")]
pub use hardware::Keypad;

#[cfg(target_os = "none")]
mod hardware {
    use embassy_futures::select::select_array;
    use embassy_nrf::gpio::Input;
    use embassy_time::{Duration, Timer};
    use heapless::Deque;

    use super::{ColumnState, KeypadLogic, Message};

    const DEBOUNCE: Duration = Duration::from_millis(20);
    const POLL_INTERVAL: Duration = Duration::from_millis(10);

    pub struct Keypad<'d> {
        columns: [[Input<'d>; 4]; 2],
        logic: KeypadLogic,
        pending: Deque<Message, 8>,
    }

    impl<'d> Keypad<'d> {
        /// Each column is ordered up, down, custom, calibration.
        pub fn new(columns: [[Input<'d>; 4]; 2]) -> Self {
            Self {
                columns,
                logic: KeypadLogic::default(),
                pending: Deque::new(),
            }
        }

        pub fn is_calibrating(&self, column: usize) -> bool {
            self.logic.is_calibrating(column)
        }

        /// While no button is down this waits on pin interrupts, so the
        /// chip sleeps. While a button is held, the pins are polled.
        pub async fn next_message(&mut self) -> Message {
            loop {
                if let Some(message) = self.pending.pop_front() {
                    return message;
                }

                if self.any_pressed() {
                    Timer::after(POLL_INTERVAL).await;
                } else {
                    let [first, second] = &mut self.columns;
                    let [a, b, c, d] = first;
                    let [e, f, g, h] = second;
                    select_array([
                        a.wait_for_low(),
                        b.wait_for_low(),
                        c.wait_for_low(),
                        d.wait_for_low(),
                        e.wait_for_low(),
                        f.wait_for_low(),
                        g.wait_for_low(),
                        h.wait_for_low(),
                    ])
                    .await;
                    Timer::after(DEBOUNCE).await;
                }

                for message in self.logic.update(self.read()) {
                    self.pending.push_back(message).ok();
                }
            }
        }

        fn read(&self) -> [ColumnState; 2] {
            self.columns
                .each_ref()
                .map(|column| column.each_ref().map(|input| input.is_low()))
        }

        fn any_pressed(&self) -> bool {
            self.read().iter().flatten().any(|&pressed| pressed)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;

    const RELEASED: ColumnState = [false; 4];
    const OMHOOG: ColumnState = [true, false, false, false];
    const OMLAAG: ColumnState = [false, true, false, false];
    const AANGEPAST: ColumnState = [false, false, true, false];
    const KALIBRATIE: ColumnState = [false, false, false, true];

    fn with(a: ColumnState, b: ColumnState) -> ColumnState {
        core::array::from_fn(|i| a[i] || b[i])
    }

    /// Feeds button states of the first column, and returns the commands.
    fn press(logic: &mut KeypadLogic, states: &[ColumnState]) -> Vec<Command> {
        states
            .iter()
            .flat_map(|&state| logic.update([state, RELEASED]))
            .map(|message| {
                assert_eq!(message.blind, BLIND_ADDRESSES[0]);
                message.command
            })
            .collect()
    }

    fn calibrating() -> KeypadLogic {
        let mut logic = KeypadLogic::default();
        press(&mut logic, &[KALIBRATIE, RELEASED]);
        logic
    }

    #[test]
    fn short_presses_send_on_press() {
        let mut logic = KeypadLogic::default();
        assert_eq!(press(&mut logic, &[OMHOOG]), [Command::Omhoog]);
        assert_eq!(press(&mut logic, &[OMHOOG, RELEASED]), []);
        assert_eq!(press(&mut logic, &[OMLAAG, RELEASED]), [Command::Omlaag]);
        assert_eq!(
            press(&mut logic, &[AANGEPAST, RELEASED]),
            [Command::NaarAangepast]
        );
    }

    #[test]
    fn calibration_toggles_on_release() {
        let mut logic = KeypadLogic::default();
        assert_eq!(press(&mut logic, &[KALIBRATIE]), []);
        assert_eq!(press(&mut logic, &[RELEASED]), [Command::KalibratieStart]);
        assert!(logic.is_calibrating(0));
        assert!(!logic.is_calibrating(1));

        assert_eq!(
            press(&mut logic, &[KALIBRATIE, RELEASED]),
            [Command::KalibratieStop]
        );
        assert!(!logic.is_calibrating(0));
    }

    #[test]
    fn holding_up_or_down_moves_while_calibrating() {
        let mut logic = calibrating();
        assert_eq!(
            press(&mut logic, &[OMHOOG, OMHOOG, RELEASED]),
            [Command::BeweegOmhoog, Command::BeweegStop]
        );
        assert_eq!(
            press(&mut logic, &[OMLAAG, RELEASED]),
            [Command::BeweegOmlaag, Command::BeweegStop]
        );
        assert!(logic.is_calibrating(0));
    }

    #[test]
    fn a_second_direction_does_not_interrupt_a_movement() {
        let mut logic = calibrating();
        assert_eq!(
            press(
                &mut logic,
                &[OMHOOG, with(OMHOOG, OMLAAG), OMLAAG, RELEASED]
            ),
            [Command::BeweegOmhoog, Command::BeweegStop]
        );
    }

    #[test]
    fn calibration_combinations_save_positions() {
        let cases = [
            (OMHOOG, Command::BovenOpslaan),
            (OMLAAG, Command::OnderOpslaan),
            (AANGEPAST, Command::AangepastOpslaan),
        ];
        for (button, command) in cases {
            let mut logic = calibrating();
            let both = with(KALIBRATIE, button);
            assert_eq!(press(&mut logic, &[KALIBRATIE, both, button]), []);
            assert_eq!(press(&mut logic, &[RELEASED]), [command]);
            assert!(logic.is_calibrating(0), "saving keeps calibration on");
        }
    }

    #[test]
    fn pressing_both_at_once_is_a_combination() {
        let mut logic = calibrating();
        assert_eq!(
            press(&mut logic, &[with(KALIBRATIE, OMHOOG), RELEASED]),
            [Command::BovenOpslaan]
        );
    }

    #[test]
    fn calibration_during_a_movement_stops_it_first() {
        let mut logic = calibrating();
        assert_eq!(
            press(
                &mut logic,
                &[OMLAAG, with(OMLAAG, KALIBRATIE), KALIBRATIE, RELEASED]
            ),
            [
                Command::BeweegOmlaag,
                Command::BeweegStop,
                Command::OnderOpslaan
            ]
        );
    }

    #[test]
    fn ambiguous_combinations_send_nothing() {
        let mut logic = calibrating();
        let three = with(KALIBRATIE, with(OMHOOG, OMLAAG));
        assert_eq!(press(&mut logic, &[three, RELEASED]), []);

        let mut logic = KeypadLogic::default();
        assert_eq!(press(&mut logic, &[with(KALIBRATIE, OMHOOG), RELEASED]), []);
        assert!(!logic.is_calibrating(0));
    }

    #[test]
    fn custom_alone_does_nothing_while_calibrating() {
        let mut logic = calibrating();
        assert_eq!(press(&mut logic, &[AANGEPAST, RELEASED]), []);
    }

    #[test]
    fn columns_are_independent() {
        let mut logic = KeypadLogic::default();
        let messages = logic.update([OMHOOG, KALIBRATIE]);
        assert_eq!(
            messages.as_slice(),
            [Message {
                blind: BLIND_ADDRESSES[0],
                command: Command::Omhoog
            }]
        );

        let messages = logic.update([RELEASED, RELEASED]);
        assert_eq!(
            messages.as_slice(),
            [Message {
                blind: BLIND_ADDRESSES[1],
                command: Command::KalibratieStart
            }]
        );
        assert!(!logic.is_calibrating(0));
        assert!(logic.is_calibrating(1));
    }
}
