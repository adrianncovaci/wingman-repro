use crate::{ButtonCommand, DigitalOutput, IoCustomError, IoLevel, SwitchCommand, Wingman2HardwareStatus};

pub(crate) trait DigitalOutputImpl {
    fn read(&self, hardware_status: &Wingman2HardwareStatus) -> Option<IoLevel>;

    fn write(&self, hardware_status: &mut Wingman2HardwareStatus, value: Result<IoLevel, IoCustomError>);

    fn toggle(&self, hardware_status: &mut Wingman2HardwareStatus);
}

impl DigitalOutputImpl for DigitalOutput {
    fn read(&self, hardware_status: &Wingman2HardwareStatus) -> Option<IoLevel> {
        hardware_status.get_digital_output(self)
    }

    fn write(&self, hardware_status: &mut Wingman2HardwareStatus, value: Result<IoLevel, IoCustomError>) {
        match value {
            Ok(value) => hardware_status.set_digital_output(self, value),
            Err(_) => hardware_status.set_digital_output(
                self,
                if self.value_on_error {
                    IoLevel::High
                } else {
                    IoLevel::Low
                },
            ), // Todo: report error.
        }
    }

    fn toggle(&self, hardware_status: &mut Wingman2HardwareStatus) {
        if let Some(level) = self.read(hardware_status).map(|level| level.toggle()) {
            self.write(hardware_status, Ok(level));
        }
    }
}

pub(crate) trait ApplyCommand<C> {
    fn apply(&self, hardware_status: &mut Wingman2HardwareStatus, command: C);
}

impl ApplyCommand<SwitchCommand> for DigitalOutput {
    fn apply(&self, hardware_status: &mut Wingman2HardwareStatus, command: SwitchCommand) {
        match command {
            SwitchCommand::On => self.write(hardware_status, Ok(IoLevel::High)),
            SwitchCommand::Off => self.write(hardware_status, Ok(IoLevel::Low)),
            SwitchCommand::Toggle => self.toggle(hardware_status),
        }
    }
}

impl ApplyCommand<ButtonCommand> for DigitalOutput {
    fn apply(&self, hardware_status: &mut Wingman2HardwareStatus, command: ButtonCommand) {
        match command {
            ButtonCommand::On => self.write(hardware_status, Ok(IoLevel::High)),
            ButtonCommand::Off => self.write(hardware_status, Ok(IoLevel::Low)),
        }
    }
}
