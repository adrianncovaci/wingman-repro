use crate::data::user_commands::{HardwareCommand, UserCommands};
use crate::io::digital_output_impl::{ApplyCommand, DigitalOutputImpl};
use crate::{BoardMap, ControllerLogic, SwitchCommand, UserCommand};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::Wingman2HardwareStatus;
use crate::DigitalOutput;

#[derive(Clone, Copy, EnumIter, PartialEq, Eq)]
enum Light {
    Navigation,
    Anchor,
    Courtesy,
    Underwater,
    Ambient,
}

#[allow(clippy::single_match)]
impl Light {
    fn command_extractor(&self, user_command: &UserCommand) -> Option<SwitchCommand> {
        None
    }

    const fn digital_output(&self, board_map: &BoardMap) -> Option<DigitalOutput> {
        None
    }

    fn force(&self, hardware_status: &mut Wingman2HardwareStatus, switch_command: &SwitchCommand) {
    }

    fn on_apply_hook(
        &self,
        hardware_status: &mut Wingman2HardwareStatus,
        _switch_command: &SwitchCommand,
    ) {
    }
}

#[allow(clippy::single_match)]
impl ControllerLogic for Light {
    fn initialize(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
    }

    fn apply_user_commands(
        &mut self,
        user_commands: &mut UserCommands,
        hardware_status: &mut Wingman2HardwareStatus,
    ) {
    }

    fn update(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
    }
}

#[derive(Default, Clone)]
pub struct Lights {}

impl ControllerLogic for Lights {
    fn initialize(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
    }

    fn apply_user_commands(
        &mut self,
        user_commands: &mut UserCommands,
        hardware_status: &mut Wingman2HardwareStatus,
    ) {
    }

    fn update(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
    }
}