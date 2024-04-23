use self::{anchor_up_down::AnchorUpDown, lights::Lights, pumps::Pumps};
use crate::{data::user_commands::UserCommands, ControllerLogic, Wingman2HardwareStatus};

mod anchor_up_down;
mod engine_ignition;
mod lights;
mod pumps;

#[derive(Default, Clone)]
pub struct ButtonsAndSwitches {
    anchor_up_down: AnchorUpDown,
    lights: Lights,
    pumps: Pumps,
}

impl ControllerLogic for ButtonsAndSwitches {
    fn initialize(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
    }

    fn apply_user_commands(
        &mut self,
        user_commands: &mut UserCommands, // TOCHECK: Testing, probably should be immutable
        hardware_status: &mut Wingman2HardwareStatus,
    ) {
    }

    fn update(&mut self, _hardware_status: &mut Wingman2HardwareStatus) {
    }
}
