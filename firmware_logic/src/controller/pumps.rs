use crate::{
    data::user_commands::{HardwareCommand, UserCommands},
    io::digital_output_impl::DigitalOutputImpl,
    ControllerLogic, Wingman2HardwareStatus,
};

#[derive(Default, Clone)]
pub struct Pumps {}

impl ControllerLogic for Pumps {
    fn initialize(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
        // Start with all pumps activated
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