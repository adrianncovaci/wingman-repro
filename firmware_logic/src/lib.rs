#![no_std]

use data::user_commands::UserCommands;

pub mod data;
use serde_big_array::BigArray;
use uom::si::f32::ElectricPotential;
use uom::si::electric_potential::volt;
use uom::si::f32::Frequency;
use uom::si::frequency::cycle_per_minute;
pub trait ControllerLogic {
    fn initialize(&mut self, hardware_status: &mut Wingman2HardwareStatus);

    fn apply_user_commands(
        &mut self,
        user_commands: &mut UserCommands,
        hardware_status: &mut Wingman2HardwareStatus,
    );

    fn update(&mut self, hardware_status: &mut Wingman2HardwareStatus);

    fn update_reporting(&self, firmware_reporting: &mut FirmwareReporting) {
        // NoOp
    }
}

#[derive(Default, Clone)]
pub struct FirmwareLogic {
}

impl ControllerLogic for FirmwareLogic {
    fn initialize(&mut self, hardware_status: &mut Wingman2HardwareStatus) {
    }

    fn apply_user_commands(
        &mut self,
        user_commands: &mut UserCommands,
        hardware_status: &mut Wingman2HardwareStatus,
    ) {
    }

    fn update(&mut self, _hardware_status: &mut Wingman2HardwareStatus) {
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wingman2HardwareStatus {
    pub step: u64,
    pub now: Timestamp,
    pub configuration: Configuration,
    #[serde(with = "BigArray")]
    pub digital_inputs: [HardwareDigital; 48],
    #[serde(with = "BigArray")]
    pub digital_outputs: [HardwareDigital; 48],
    #[serde(with = "BigArray")]
    pub analog_inputs: [HardwareAnalog; 48],
    #[serde(with = "BigArray")]
    pub analog_outputs: [HardwareAnalog; 36],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wingman3HardwareStatus {
    pub step: u64,
    pub now: Timestamp,
    pub configuration: Configuration,
    #[serde(with = "BigArray")]
    pub digital_inputs: [HardwareDigital; 48],
    #[serde(with = "BigArray")]
    pub digital_outputs: [HardwareDigital; 48],
    #[serde(with = "BigArray")]
    pub analog_inputs: [HardwareAnalog; 48],
    #[serde(with = "BigArray")]
    pub analog_outputs: [HardwareAnalog; 36],
}
impl Wingman3HardwareStatus {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            step: 0,
            now: Timestamp::default(),
            configuration,
            digital_inputs: core::array::from_fn(|_| HardwareDigital::default()),
            digital_outputs: core::array::from_fn(|_| HardwareDigital::default()),
            analog_inputs: core::array::from_fn(|_| HardwareAnalog {
                voltage: ElectricPotential::new::<volt>(0.0),
            }),
            analog_outputs: core::array::from_fn(|_| HardwareAnalog {
                voltage: ElectricPotential::new::<volt>(0.0),
            }),
        }
    }
}

impl Default for Wingman2HardwareStatus {
    fn default() -> Self {
        Self {
            step: 0,
            now: Timestamp::default(),
            configuration: Configuration::Unconfigured,
            digital_inputs: core::array::from_fn(|_| Default::default()),
            digital_outputs: core::array::from_fn(|_| Default::default()),
            analog_inputs: core::array::from_fn(|_| Default::default()),
            analog_outputs: core::array::from_fn(|_| Default::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HardwareAnalog {
    pub voltage: ElectricPotential,
}

impl Wingman2HardwareStatus {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            step: 0,
            now: Timestamp::default(),
            configuration,
            digital_inputs: core::array::from_fn(|_| HardwareDigital::default()),
            digital_outputs: core::array::from_fn(|_| HardwareDigital::default()),
            analog_inputs: core::array::from_fn(|_| HardwareAnalog {
                voltage: ElectricPotential::new::<volt>(0.0),
            }),
            analog_outputs: core::array::from_fn(|_| HardwareAnalog {
                voltage: ElectricPotential::new::<volt>(0.0),
            }),
        }
    }
}

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use core::{cmp::Ordering, fmt::Debug, ops::Add};

pub type Result<T, SPI, HAL> = core::result::Result<T, IoError<SPI, HAL>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IoError<SPI: Debug, HAL: Debug> {
    /// SN65HVS881 Parity Bit Incorrect
    DiParity,
    /// SN65HVS881 Not Present
    NotPresent,
    /// Illegal DI Number
    IllegalInput,
    /// Illegal DO Number
    IllegalOutput,
    /// Unsupported Output Level
    UnsupportedLevel,
    /// SPI error.
    Spi(SPI),
    /// Error from HAL crate.
    Hal(HAL),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum IoBank {
    Bank1,
    Bank2,
    Bank3,
    Bank4,
    Bank5,
    Bank6,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoType {
    DigitalIn,
    DigitalOut,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoTemp {
    Normal,
    Warm,
    Hot,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoSupply {
    Volts24,
    Volts12,
    Powered,
    Unpowered,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoState {
    Normal,
    OpenOrFault,
}

impl IoLevel {
    pub fn toggle(self) -> Self {
        match self {
            IoLevel::High => IoLevel::Low,
            IoLevel::Low => IoLevel::High,
            IoLevel::HiZ => IoLevel::HiZ,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoLevel {
    High,
    Low,
    HiZ,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HardwareDigital {
    pub state: Option<IoState>,
    pub level: Option<IoLevel>,
    pub supply: Option<IoSupply>,
    pub temp: Option<IoTemp>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct UserCommand {
    pub command: Command,
    pub timestamp: Option<Timestamp>,
}

/// A SwitchCommand just needs to be sent once.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum SwitchCommand {
    On,
    Off,
    Toggle,
}

/// A ButtonCommands has to be sent continuously, otherwise the button is deemed to be Off.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum ButtonCommand {
    On,
    Off,
}

impl UserCommand {
    const EVICTION_DELAY: Timestamp = Timestamp::new(500);

    pub fn new(command: Command, timestamp: Option<Timestamp>) -> Self {
        Self { command, timestamp }
    }

    pub fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamp = Some(timestamp);
    }

    pub fn is_expired(&self, now: Timestamp) -> bool {
        self.timestamp
            .map_or(true, |timestamp| timestamp + Self::EVICTION_DELAY <= now)
    }

    pub fn is_valid(&self, now: Timestamp) -> bool {
        !self.is_expired(now)
    }
}

impl PartialOrd for UserCommand {
    // A UserCommand is greater than another if it has the same command and is more recent.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.command != other.command {
            None
        } else {
            match (self.timestamp, other.timestamp) {
                (Some(_), None) => Some(Ordering::Greater),
                (Some(timestamp), Some(other_timestamp)) => Some(timestamp.cmp(&other_timestamp)),
                (None, Some(_)) => Some(Ordering::Less),
                (None, None) => None,
            }
        }
    }
}

/// Used to number devices. Starts at 1. Front to back, left to right, as in aeronautics.
pub type DevicePosition = usize;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum DeviceIdentifier {
    All,
    Device(DevicePosition),
}

impl From<usize> for DeviceIdentifier {
    fn from(identifier: usize) -> Self {
        DeviceIdentifier::Device(DevicePosition::from(identifier))
    }
}

impl DeviceIdentifier {
    pub fn from_index(index: usize) -> Self {
        DeviceIdentifier::Device(DevicePosition::from(index + 1))
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Command {
    AmbientLight(SwitchCommand),
    AnchorDown(SwitchCommand),
    AnchorLight(SwitchCommand),
    AnchorUp(SwitchCommand),
    BilgePump(DeviceIdentifier, SwitchCommand),
    BlackWaterPump(ButtonCommand),
    CourtesyLight(SwitchCommand),
    EngineBatteryOn(DeviceIdentifier),
    EngineIgnition(DeviceIdentifier, SwitchCommand),
    EngineRoomLight(DeviceIdentifier, SwitchCommand),
    EngineStart(DeviceIdentifier, ButtonCommand),
    FoilDeploy(DeviceIdentifier, ButtonCommand),
    FoilDown(DeviceIdentifier, ButtonCommand),
    FoilIn(DeviceIdentifier, ButtonCommand),
    FoilOut(DeviceIdentifier, ButtonCommand),
    FoilRetract(DeviceIdentifier, ButtonCommand),
    FoilUp(DeviceIdentifier, ButtonCommand),
    HydraulicDcPumpOn(DeviceIdentifier, ButtonCommand),
    NavigationLight(SwitchCommand),
    RudderDeploy(DeviceIdentifier, ButtonCommand),
    RudderDown(DeviceIdentifier, ButtonCommand),
    RudderPark(DeviceIdentifier, ButtonCommand),
    RudderRetract(DeviceIdentifier, ButtonCommand),
    RudderTiltIn(DeviceIdentifier, ButtonCommand),
    RudderTiltOut(DeviceIdentifier, ButtonCommand),
    RudderUp(DeviceIdentifier, ButtonCommand),
    UnderwaterLight(SwitchCommand),
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp {
    ms: u64,
}

impl Timestamp {
    pub const fn new(millis: u64) -> Self {
        Self { ms: millis }
    }
}

impl Add<Timestamp> for Timestamp {
    type Output = Timestamp;

    fn add(self, other: Timestamp) -> Timestamp {
        Timestamp::new(self.ms + other.ms)
    }
}

impl<const NOM: u32, const DENOM: u32> From<fugit::Instant<u64, NOM, DENOM>> for Timestamp {
    fn from(instant: fugit::Instant<u64, NOM, DENOM>) -> Self {
        Self {
            ms: instant.duration_since_epoch().to_millis(),
        }
    }
}

impl<const NOM: u32, const DENOM: u32> Into<fugit::Instant<u64, NOM, DENOM>> for Timestamp {
    fn into(self) -> fugit::Instant<u64, NOM, DENOM> {
        fugit::Instant::<u64, NOM, DENOM>::from_ticks(0)
            + fugit::Duration::<u64, 1, 1_000>::from_ticks(self.ms).convert()
    }
}

impl<const NOM: u32, const DENOM: u32> From<fugit::Duration<u64, NOM, DENOM>> for Timestamp {
    fn from(duration: fugit::Duration<u64, NOM, DENOM>) -> Self {
        Self {
            ms: duration.to_millis(),
        }
    }
}

impl<const NOM: u32, const DENOM: u32> Into<fugit::Duration<u64, NOM, DENOM>> for Timestamp {
    fn into(self) -> fugit::Duration<u64, NOM, DENOM> {
        fugit::Duration::<u64, 1, 1_000>::from_ticks(self.ms).convert()
    }
}

pub struct BoardMap {
    pub digital_input_count: usize,
    pub digital_output_count: usize,
    pub analog_input_count: usize,
    pub analog_output_count: usize,
    pub pulse_wave_modulation_count: usize,
    // Digital Outputs
    pub anchor_up: Option<DigitalOutput>,
    pub anchor_down: Option<DigitalOutput>,
    pub anchor_light: Option<DigitalOutput>,
    pub navigation_light: Option<DigitalOutput>,
    pub courtesy_light: Option<DigitalOutput>,
    pub ambient_light: Option<DigitalOutput>,
    pub underwater_light: Option<DigitalOutput>,
    pub bilge_pumps: [Option<DigitalOutput>; 3],
    pub black_water_pump: Option<DigitalOutput>,
    pub engine_room_lights: [Option<DigitalOutput>; 2],
    // Digital Inputs
    pub engine_battery_port: Option<DigitalInput>,
    pub engine_battery_stbd: Option<DigitalInput>,
    pub bilge_pumps_running: [Option<DigitalInput>; 3],
    // Analog Inputs
    pub fresh_water_level: Option<AnalogInput>,
    pub black_water_level: Option<AnalogInput>,
}

#[derive(Clone, Debug)]
pub struct BoardDeviceData<T> {
    pub topic_name: &'static str,
    pub position: Option<usize>,
    pub device: T,
}

impl<T> BoardDeviceData<T> {
    pub fn new(topic_name: &'static str, position: Option<usize>, device: T) -> Self {
        Self {
            topic_name,
            position,
            device,
        }
    }
}


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct FirmwareReporting {}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ServerReporting {}

#[derive(
    Eq,
    PartialEq,
    Debug,
    Clone,
    Copy,
    Default,
    Display,
    EnumIter,
    EnumString,
    Serialize,
    Deserialize,
)]
#[non_exhaustive]
pub enum Configuration {
    #[default]
    Unconfigured = 0,
    Prototype0 = 1,
    Prototype2 = 2,
    Spirit101 = 101, // Al Seer
    Spirit102 = 102, // ENATA
    Spirit103 = 103, // STREIT
    Spirit104 = 104, // Lena
    UnitTest = 9999,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wingman2IOCardStatus {
    pub is_connected_to_control_server: bool,
    pub control_server_qos: u8,
    pub control_server_received_messages: u64,
    tick: u64,
    total_uptime_ms: i64,
    pub configuration: Configuration,
}

#[allow(clippy::should_implement_trait)]
impl Wingman2IOCardStatus {
    pub const fn default() -> Self {
        Self {
            tick: 0,
            is_connected_to_control_server: false,
            control_server_qos: 0,
            control_server_received_messages: 0,
            total_uptime_ms: 0,
            configuration: Configuration::Unconfigured,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum IoCustomError {
    NoSensor(IOAddress),
    OutOfRange,
    SensorTripped,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IOAddress {
    DigitalInputAddress(DigitalInput),
    DigitalOutputAddress(DigitalOutput),
    AnalogInputAddress(AnalogInput),
    AnalogOutputAddress(AnalogOutput),
    PulseWidthModulationAddress(PulseWidthModulation),
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DigitalInput {
    pub address: usize,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DigitalOutput {
    pub address: usize,
    pub value_on_error: bool,
    pub bank_voltage: u8,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnalogInput {
    pub address: usize,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnalogOutput {
    pub address: usize,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PulseWidthModulation {
    pub address: usize,
    pub bank_voltage: u8,
}
