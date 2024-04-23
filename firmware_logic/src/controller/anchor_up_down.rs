use crate::data::user_commands::UserCommands;
use crate::io::digital_output_impl::DigitalOutputImpl;
use crate::{ControllerLogic, Wingman2HardwareStatus};

#[derive(Default, Clone)]
pub struct AnchorUpDown {}

impl ControllerLogic for AnchorUpDown {
    fn initialize(&mut self, _: &mut Wingman2HardwareStatus) {
        // NoOp
    }

    fn apply_user_commands(
        &mut self,
        user_commands: &mut UserCommands,
        hardware_status: &mut Wingman2HardwareStatus,
    ) {
    }

    fn update(&mut self, _: &mut Wingman2HardwareStatus) {
        // NoOp
    }
}
