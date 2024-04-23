use crate::{DigitalInput, IoLevel, Wingman2HardwareStatus};


pub(crate) trait DigitalInputImpl {
    fn read(&self, hardware_status: &Wingman2HardwareStatus) -> Option<IoLevel>;
}

impl DigitalInputImpl for DigitalInput {
    fn read(&self, hardware_status: &Wingman2HardwareStatus) -> Option<IoLevel> {
        hardware_status.get_digital_input(self)
    }
}
