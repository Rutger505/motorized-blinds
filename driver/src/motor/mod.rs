//! The blind's state machine and its stepper driver.

pub mod tmc2209;

use core::future::Future;

use embedded_storage::nor_flash::NorFlash;
use protocol::Command;

use crate::mqtt::{CoverState, Status};
use crate::opslag::{Positions, Storage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum Direction {
    Omhoog,
    Omlaag,
}

pub trait Stepper {
    fn enable(&mut self);
    fn disable(&mut self);
    fn set_direction(&mut self, direction: Direction);
    /// Gives one step pulse, then waits until the next step may follow.
    fn step(&mut self) -> impl Future<Output = ()>;
}

pub trait Endstop {
    fn is_pressed(&mut self) -> bool;
}

#[cfg(target_os = "none")]
impl Endstop for esp_hal::gpio::Input<'_> {
    /// The switch pulls the pin to GND, against the internal pull-up.
    fn is_pressed(&mut self) -> bool {
        self.is_low()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum State {
    Stil,
    Omhoog,
    Omlaag,
    NaarAangepast,
    KalibratieStil,
    KalibratieOmhoog,
    KalibratieOmlaag,
}

pub struct Blind<D, E, F> {
    driver: D,
    endstop: E,
    storage: Storage<F>,
    state: State,
    positions: Positions,
}

impl<D: Stepper, E: Endstop, F: NorFlash> Blind<D, E, F> {
    pub fn new(mut driver: D, endstop: E, mut storage: Storage<F>) -> Self {
        driver.disable();
        let positions = storage.load();
        Self {
            driver,
            endstop,
            storage,
            state: State::Stil,
            positions,
        }
    }

    pub fn handle(&mut self, command: Command) -> Result<(), F::Error> {
        use State::*;

        match (self.state, command) {
            (Stil, Command::Omhoog) => self.start(Omhoog),
            (Stil, Command::Omlaag) => self.start(Omlaag),
            (Stil, Command::NaarAangepast) => self.start(NaarAangepast),
            (Stil, Command::KalibratieStart) => self.state = KalibratieStil,
            (
                Omhoog | Omlaag | NaarAangepast,
                Command::Omhoog | Command::Omlaag | Command::Stop,
            ) => {
                return self.stop(Stil);
            }

            (KalibratieStil, Command::BeweegOmhoog) => self.start(KalibratieOmhoog),
            (KalibratieStil, Command::BeweegOmlaag) => self.start(KalibratieOmlaag),
            (KalibratieOmhoog | KalibratieOmlaag, Command::BeweegStop | Command::Stop) => {
                return self.stop(KalibratieStil);
            }
            (KalibratieStil, Command::BovenOpslaan) => {
                self.positions.set_top();
                return self.save();
            }
            (KalibratieStil, Command::AangepastOpslaan) => {
                self.positions.custom = self.positions.current;
                return self.save();
            }
            (KalibratieStil, Command::OnderOpslaan) => {
                self.positions.max = self.positions.current;
                return self.save();
            }
            (KalibratieStil, Command::KalibratieStop) => self.state = Stil,
            (KalibratieOmhoog | KalibratieOmlaag, Command::KalibratieStop) => {
                return self.stop(Stil);
            }

            _ => {}
        }
        Ok(())
    }

    pub fn is_moving(&self) -> bool {
        self.direction().is_some()
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn positions(&self) -> Positions {
        self.positions
    }

    /// Moves one step towards the target, or stops once it is reached.
    pub async fn step(&mut self) -> Result<(), F::Error> {
        let Some(direction) = self.direction() else {
            return Ok(());
        };

        if direction == Direction::Omhoog && self.endstop.is_pressed() {
            return if self.is_calibrating() {
                self.stop(State::KalibratieStil)
            } else {
                self.positions.current = 0;
                self.stop(State::Stil)
            };
        }

        if self.target_reached(direction) {
            return self.stop(State::Stil);
        }

        self.positions.current += match direction {
            Direction::Omhoog => -1,
            Direction::Omlaag => 1,
        };
        self.driver.step().await;
        Ok(())
    }

    pub fn status(&self) -> Status {
        let Positions { current, max, .. } = self.positions;

        let state = match self.direction() {
            Some(Direction::Omhoog) => CoverState::Opening,
            Some(Direction::Omlaag) => CoverState::Closing,
            None if current <= 0 => CoverState::Open,
            None if max > 0 && current >= max => CoverState::Closed,
            None => CoverState::Stopped,
        };

        let closed_percent = if max > 0 {
            (i64::from(current) * 100 / i64::from(max)).clamp(0, 100)
        } else if current > 0 {
            100
        } else {
            0
        };

        Status {
            state,
            position: (100 - closed_percent) as u8,
        }
    }

    fn start(&mut self, state: State) {
        self.state = state;
        if let Some(direction) = self.direction() {
            self.driver.set_direction(direction);
            self.driver.enable();
        }
    }

    fn stop(&mut self, state: State) -> Result<(), F::Error> {
        self.driver.disable();
        self.state = state;
        self.save()
    }

    fn save(&mut self) -> Result<(), F::Error> {
        self.storage.save(&self.positions)
    }

    fn is_calibrating(&self) -> bool {
        matches!(
            self.state,
            State::KalibratieStil | State::KalibratieOmhoog | State::KalibratieOmlaag
        )
    }

    fn target(&self) -> Option<i32> {
        match self.state {
            State::Omhoog => Some(0),
            State::Omlaag => Some(self.positions.max),
            State::NaarAangepast => Some(self.positions.custom),
            _ => None,
        }
    }

    fn direction(&self) -> Option<Direction> {
        match self.state {
            State::Omhoog | State::KalibratieOmhoog => Some(Direction::Omhoog),
            State::Omlaag | State::KalibratieOmlaag => Some(Direction::Omlaag),
            State::NaarAangepast if self.positions.custom < self.positions.current => {
                Some(Direction::Omhoog)
            }
            State::NaarAangepast => Some(Direction::Omlaag),
            State::Stil | State::KalibratieStil => None,
        }
    }

    fn target_reached(&self, direction: Direction) -> bool {
        match (self.target(), direction) {
            (Some(target), Direction::Omhoog) => self.positions.current <= target,
            (Some(target), Direction::Omlaag) => self.positions.current >= target,
            (None, _) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::RamFlash;
    use core::cell::Cell;
    use embassy_futures::block_on;

    #[derive(Default)]
    struct FakeStepper {
        enabled: Cell<bool>,
        direction: Cell<Option<Direction>>,
        steps: Cell<u32>,
    }

    impl Stepper for &FakeStepper {
        fn enable(&mut self) {
            self.enabled.set(true);
        }

        fn disable(&mut self) {
            self.enabled.set(false);
        }

        fn set_direction(&mut self, direction: Direction) {
            self.direction.set(Some(direction));
        }

        async fn step(&mut self) {
            assert!(self.enabled.get(), "stepped while the driver is off");
            self.steps.set(self.steps.get() + 1);
        }
    }

    /// Pressed once the stepper has made this many steps.
    struct FakeEndstop<'a> {
        pressed_after: Option<u32>,
        stepper: &'a FakeStepper,
    }

    impl Endstop for FakeEndstop<'_> {
        fn is_pressed(&mut self) -> bool {
            self.pressed_after
                .is_some_and(|steps| self.stepper.steps.get() >= steps)
        }
    }

    type TestBlind<'a> = Blind<&'a FakeStepper, FakeEndstop<'a>, &'a mut RamFlash>;

    const CALIBRATED: Positions = Positions {
        current: 0,
        max: 400,
        custom: 150,
    };

    struct Fixture {
        stepper: FakeStepper,
        flash: RamFlash,
    }

    impl Fixture {
        fn with(positions: Positions) -> Self {
            let mut flash = RamFlash::new();
            let mut storage = Storage::new(&mut flash);
            storage.load();
            storage.save(&positions).unwrap();
            Self {
                stepper: FakeStepper::default(),
                flash,
            }
        }

        fn blind(&mut self) -> TestBlind<'_> {
            self.blind_with_endstop(None)
        }

        fn blind_with_endstop(&mut self, pressed_after: Option<u32>) -> TestBlind<'_> {
            let endstop = FakeEndstop {
                pressed_after,
                stepper: &self.stepper,
            };
            Blind::new(&self.stepper, endstop, Storage::new(&mut self.flash))
        }

        fn saved(&mut self) -> Positions {
            Storage::new(&mut self.flash).load()
        }
    }

    fn run(blind: &mut TestBlind) {
        for _ in 0..10_000 {
            if !blind.is_moving() {
                return;
            }
            block_on(blind.step()).unwrap();
        }
        panic!("the blind never stopped");
    }

    fn step_times(blind: &mut TestBlind, times: u32) {
        for _ in 0..times {
            block_on(blind.step()).unwrap();
        }
    }

    #[test]
    fn starts_still_with_the_saved_positions() {
        let mut fixture = Fixture::with(CALIBRATED);
        let blind = fixture.blind();
        assert_eq!(blind.state(), State::Stil);
        assert_eq!(blind.positions(), CALIBRATED);
        assert!(!blind.is_moving());
    }

    #[test]
    fn omlaag_moves_to_max_and_saves() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind();

        blind.handle(Command::Omlaag).unwrap();
        assert!(blind.is_moving());
        run(&mut blind);

        assert_eq!(blind.state(), State::Stil);
        assert_eq!(blind.positions().current, 400);
        assert_eq!(fixture.stepper.direction.get(), Some(Direction::Omlaag));
        assert!(!fixture.stepper.enabled.get());
        assert_eq!(fixture.stepper.steps.get(), 400);
        assert_eq!(fixture.saved().current, 400);
    }

    #[test]
    fn omhoog_moves_back_to_step_zero() {
        let mut fixture = Fixture::with(Positions {
            current: 400,
            ..CALIBRATED
        });
        let mut blind = fixture.blind();

        blind.handle(Command::Omhoog).unwrap();
        run(&mut blind);

        assert_eq!(blind.positions().current, 0);
        assert_eq!(fixture.stepper.direction.get(), Some(Direction::Omhoog));
        assert_eq!(fixture.stepper.steps.get(), 400);
    }

    #[test]
    fn naar_aangepast_moves_down_or_up_to_custom() {
        for (start, direction) in [(0, Direction::Omlaag), (400, Direction::Omhoog)] {
            let mut fixture = Fixture::with(Positions {
                current: start,
                ..CALIBRATED
            });
            let mut blind = fixture.blind();
            blind.handle(Command::NaarAangepast).unwrap();
            run(&mut blind);
            assert_eq!(blind.positions().current, 150);
            assert_eq!(fixture.stepper.direction.get(), Some(direction));
        }
    }

    #[test]
    fn omhoog_omlaag_or_stop_while_moving_stops_and_saves() {
        for command in [Command::Omhoog, Command::Omlaag, Command::Stop] {
            let mut fixture = Fixture::with(CALIBRATED);
            let mut blind = fixture.blind();
            blind.handle(Command::Omlaag).unwrap();
            step_times(&mut blind, 50);

            blind.handle(command).unwrap();

            assert_eq!(blind.state(), State::Stil);
            assert!(!blind.is_moving());
            assert!(!fixture.stepper.enabled.get());
            assert_eq!(fixture.saved().current, 50);
        }
    }

    #[test]
    fn naar_aangepast_while_moving_is_ignored() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind();
        blind.handle(Command::Omlaag).unwrap();
        blind.handle(Command::NaarAangepast).unwrap();
        assert_eq!(blind.state(), State::Omlaag);
    }

    #[test]
    fn endstop_resets_the_position_to_zero() {
        let mut fixture = Fixture::with(Positions {
            current: 400,
            ..CALIBRATED
        });
        let mut blind = fixture.blind_with_endstop(Some(300));

        blind.handle(Command::Omhoog).unwrap();
        run(&mut blind);

        assert_eq!(blind.state(), State::Stil);
        assert_eq!(blind.positions().current, 0);
        assert_eq!(fixture.stepper.steps.get(), 300);
        assert_eq!(fixture.saved().current, 0);
    }

    #[test]
    fn endstop_is_ignored_when_moving_down() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind_with_endstop(Some(0));

        blind.handle(Command::Omlaag).unwrap();
        run(&mut blind);

        assert_eq!(blind.positions().current, 400);
    }

    #[test]
    fn calibration_moves_freely_past_the_limits() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind();

        blind.handle(Command::KalibratieStart).unwrap();
        assert_eq!(blind.state(), State::KalibratieStil);
        blind.handle(Command::BeweegOmhoog).unwrap();
        step_times(&mut blind, 100);
        assert!(blind.is_moving());
        blind.handle(Command::BeweegStop).unwrap();

        assert_eq!(blind.state(), State::KalibratieStil);
        assert_eq!(blind.positions().current, -100);
    }

    #[test]
    fn calibration_saves_top_custom_and_bottom() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind();
        blind.handle(Command::KalibratieStart).unwrap();

        blind.handle(Command::BeweegOmhoog).unwrap();
        step_times(&mut blind, 20);
        blind.handle(Command::BeweegStop).unwrap();
        blind.handle(Command::BovenOpslaan).unwrap();
        assert_eq!(
            blind.positions(),
            Positions {
                current: 0,
                max: 420,
                custom: 170,
            }
        );

        blind.handle(Command::BeweegOmlaag).unwrap();
        step_times(&mut blind, 600);
        blind.handle(Command::BeweegStop).unwrap();
        blind.handle(Command::OnderOpslaan).unwrap();

        blind.handle(Command::BeweegOmhoog).unwrap();
        step_times(&mut blind, 200);
        blind.handle(Command::BeweegStop).unwrap();
        blind.handle(Command::AangepastOpslaan).unwrap();

        blind.handle(Command::KalibratieStop).unwrap();
        assert_eq!(blind.state(), State::Stil);
        let expected = Positions {
            current: 400,
            max: 600,
            custom: 400,
        };
        assert_eq!(blind.positions(), expected);
        assert_eq!(fixture.saved(), expected);
    }

    #[test]
    fn kalibratie_stop_while_moving_stops_the_motor() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind();
        blind.handle(Command::KalibratieStart).unwrap();
        blind.handle(Command::BeweegOmlaag).unwrap();
        blind.handle(Command::KalibratieStop).unwrap();

        assert_eq!(blind.state(), State::Stil);
        assert!(!fixture.stepper.enabled.get());
    }

    #[test]
    fn endstop_stops_calibration_without_resetting() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind_with_endstop(Some(0));
        blind.handle(Command::KalibratieStart).unwrap();
        blind.handle(Command::BeweegOmhoog).unwrap();
        step_times(&mut blind, 1);

        assert_eq!(blind.state(), State::KalibratieStil);
        assert_eq!(blind.positions(), CALIBRATED);
    }

    #[test]
    fn commands_for_another_mode_are_ignored() {
        let mut fixture = Fixture::with(CALIBRATED);
        let mut blind = fixture.blind();

        for command in [
            Command::BeweegOmhoog,
            Command::BovenOpslaan,
            Command::OnderOpslaan,
            Command::KalibratieStop,
        ] {
            blind.handle(command).unwrap();
            assert_eq!(blind.state(), State::Stil);
        }

        blind.handle(Command::KalibratieStart).unwrap();
        for command in [Command::Omhoog, Command::Omlaag, Command::NaarAangepast] {
            blind.handle(command).unwrap();
            assert_eq!(blind.state(), State::KalibratieStil);
        }
        assert_eq!(blind.positions(), CALIBRATED);
    }

    #[test]
    fn status_reports_state_and_open_percentage() {
        let cases = [
            (0, CoverState::Open, 100),
            (400, CoverState::Closed, 0),
            (100, CoverState::Stopped, 75),
            (-20, CoverState::Open, 100),
            (500, CoverState::Closed, 0),
        ];
        for (current, state, position) in cases {
            let mut fixture = Fixture::with(Positions {
                current,
                ..CALIBRATED
            });
            assert_eq!(fixture.blind().status(), Status { state, position });
        }
    }

    #[test]
    fn status_while_moving() {
        let mut fixture = Fixture::with(Positions {
            current: 200,
            ..CALIBRATED
        });
        let mut blind = fixture.blind();
        blind.handle(Command::Omlaag).unwrap();
        assert_eq!(blind.status().state, CoverState::Closing);
        blind.handle(Command::Stop).unwrap();
        blind.handle(Command::NaarAangepast).unwrap();
        assert_eq!(
            blind.status(),
            Status {
                state: CoverState::Opening,
                position: 50
            }
        );
    }

    #[test]
    fn uncalibrated_blind_does_not_move_down() {
        let mut fixture = Fixture::with(Positions::default());
        let mut blind = fixture.blind();
        blind.handle(Command::Omlaag).unwrap();
        run(&mut blind);
        assert_eq!(blind.status().state, CoverState::Open);
        assert_eq!(fixture.stepper.steps.get(), 0);
    }
}
