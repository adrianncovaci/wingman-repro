#![allow(dead_code)]

use asm_delay_embedded_time::AsmDelay;
use firmware_logic::{IoBank, IoError, IoLevel, IoState, IoSupply, IoTemp, IoType};
use core::fmt::Debug;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::{Mode, Phase, Polarity};
use embedded_time::rate::Extensions;
//use rtt_target::rprintln;
use crate::io::dio::spi::{Enabled, Spi};
use stm32h7xx_hal::device::SPI4;
use stm32h7xx_hal::spi;
use stm32h7xx_hal::spi::HalSpi;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub type Result<T, SPI, HAL> = core::result::Result<T, IoError<SPI, HAL>>;

pub struct Dio<SPI, NCS, SEL0, SEL1, SEL2, SEL3, LD> {
    sel0: SEL0,
    sel1: SEL1,
    sel2: SEL2,
    sel3: SEL3,
    spi: SPI,
    ncs: NCS,
    ld: LD,
    delay: AsmDelay,
}

impl<SPI, NCS, SEL0, SEL1, SEL2, SEL3, LD, SPIE, CSE> Dio<SPI, NCS, SEL0, SEL1, SEL2, SEL3, LD>
where
    SPI: Transfer<u8, Error = SPIE> + HalSpi + TkrtSpi,
    NCS: OutputPin<Error = CSE>,
    SEL0: OutputPin<Error = CSE>,
    SEL1: OutputPin<Error = CSE>,
    SEL2: OutputPin<Error = CSE>,
    SEL3: OutputPin<Error = CSE>,
    LD: OutputPin<Error = CSE>,
    SPIE: Debug,
    CSE: Debug,
{
    pub fn new(sel0: SEL0, sel1: SEL1, sel2: SEL2, sel3: SEL3, spi: SPI, ncs: NCS, ld: LD) -> Self {
        let delay = AsmDelay::new(200_000_000u32.Hz());

        Self {
            sel0,
            sel1,
            sel2,
            sel3,
            spi,
            ncs,
            ld,
            delay,
        }
    }

    pub fn init(&mut self) -> Result<(), SPIE, CSE> {
        // Initialize Digital Output Controllers for Banks 1-6

        for bank_n in IoBank::iter() {
            self.select_io_bank(IoType::DigitalOut, bank_n)?;

            for port_n in 0..=7 {
                // Configure port expander outputs for EN pins
                self.max7301_configure_output(4 + port_n, IoLevel::HiZ)?;

                // configure Stat inputs
                self.max7301_configure_input(12 + port_n, true)?;
            }

            // configure voltage detect inputs
            self.max7301_configure_input(20, false)?;
            self.max7301_configure_input(21, false)?;

            // configure nHot input
            self.max7301_configure_input(22, true)?;
            // configure nWarm input
            self.max7301_configure_input(23, true)?;

            self.max7301_wakeup(false)?;
        }

        // Initialize Digital Input Controllers for Bank 1-6

        Ok(())
    }

    pub fn set_do_by_num(&mut self, output: u8, level: IoLevel) -> Result<(), SPIE, CSE> {
        match output {
            1..=8 => self.set_do_by_bank(IoBank::Bank1, output, level),
            9..=16 => self.set_do_by_bank(IoBank::Bank2, output - 8, level),
            17..=24 => self.set_do_by_bank(IoBank::Bank3, output - 16, level),
            25..=32 => self.set_do_by_bank(IoBank::Bank4, output - 24, level),
            33..=40 => self.set_do_by_bank(IoBank::Bank5, output - 32, level),
            41..=48 => self.set_do_by_bank(IoBank::Bank6, output - 40, level),
            _ => Err(IoError::IllegalOutput),
        }
    }

    pub fn set_do_by_bank(
        &mut self,
        bank: IoBank,
        output: u8,
        level: IoLevel,
    ) -> Result<(), SPIE, CSE> {
        self.select_io_bank(IoType::DigitalOut, bank)?;
        if output > 8 {
            return Err(IoError::IllegalOutput);
        }

        self.max7301_write_single_port(output + 3, level)?;
        Ok(())
    }

    pub fn get_do_temp(&mut self, bank: IoBank) -> Result<IoTemp, SPIE, CSE> {
        self.select_io_bank(IoType::DigitalOut, bank)?;
        if self.max7301_read_single_port(22)? == IoLevel::Low {
            return Ok(IoTemp::Hot);
        }
        if self.max7301_read_single_port(23)? == IoLevel::Low {
            return Ok(IoTemp::Warm);
        }
        Ok(IoTemp::Normal)
    }

    pub fn get_do_supply(&mut self, bank: IoBank) -> Result<IoSupply, SPIE, CSE> {
        self.select_io_bank(IoType::DigitalOut, bank)?;
        if self.max7301_read_single_port(20)? == IoLevel::High {
            return Ok(IoSupply::Volts24);
        }
        if self.max7301_read_single_port(21)? == IoLevel::High {
            return Ok(IoSupply::Volts12);
        }
        Ok(IoSupply::Unpowered)
    }

    pub fn get_do_state_by_num(&mut self, output: u8) -> Result<IoState, SPIE, CSE> {
        match output {
            1..=8 => self.get_do_state_by_bank(IoBank::Bank1, output),
            9..=16 => self.get_do_state_by_bank(IoBank::Bank2, output - 8),
            17..=24 => self.get_do_state_by_bank(IoBank::Bank3, output - 16),
            25..=32 => self.get_do_state_by_bank(IoBank::Bank4, output - 24),
            33..=40 => self.get_do_state_by_bank(IoBank::Bank5, output - 32),
            41..=48 => self.get_do_state_by_bank(IoBank::Bank6, output - 40),
            _ => Err(IoError::IllegalOutput),
        }
    }

    pub fn get_do_state_by_bank(&mut self, bank: IoBank, output: u8) -> Result<IoState, SPIE, CSE> {
        static STATE_PIN_OFFSET: u8 = 11;

        // We have no information if the bank is unpowered
        if self.get_do_supply(bank)? == IoSupply::Unpowered {
            return Err(IoError::IllegalOutput);
        }

        self.select_io_bank(IoType::DigitalOut, bank)?;
        if output > 8 {
            return Err(IoError::IllegalOutput);
        }

        if self.max7301_read_single_port(output + STATE_PIN_OFFSET)? == IoLevel::High {
            Ok(IoState::Normal)
        } else {
            Ok(IoState::OpenOrFault)
        }
    }

    pub fn get_di_by_bank(&mut self, bank: IoBank, input: u8) -> Result<IoLevel, SPIE, CSE> {
        static INPUT_PIN_OFFSET: u8 = 1;
        self.select_io_bank(IoType::DigitalIn, bank)?;
        if input > 8 || input < 1 {
            return Err(IoError::IllegalInput);
        }

        match self.sn65hvs881_read_single_bit(input - INPUT_PIN_OFFSET) {
            Ok(level) => {
                if level {
                    Ok(IoLevel::High)
                } else {
                    Ok(IoLevel::Low)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_di_by_num(&mut self, input: u8) -> Result<IoLevel, SPIE, CSE> {
        match input {
            1..=8 => self.get_di_by_bank(IoBank::Bank1, input),
            9..=16 => self.get_di_by_bank(IoBank::Bank2, input - 8),
            17..=24 => self.get_di_by_bank(IoBank::Bank3, input - 16),
            25..=32 => self.get_di_by_bank(IoBank::Bank4, input - 24),
            33..=40 => self.get_di_by_bank(IoBank::Bank5, input - 32),
            41..=48 => self.get_di_by_bank(IoBank::Bank6, input - 40),
            _ => Err(IoError::IllegalInput),
        }
    }

    pub fn get_di_temp(&mut self, bank: IoBank) -> Result<IoTemp, SPIE, CSE> {
        self.select_io_bank(IoType::DigitalIn, bank)?;

        match self.sn65hvs881_read_single_bit(8) {
            Ok(level) => {
                if level {
                    Ok(IoTemp::Normal)
                } else {
                    Ok(IoTemp::Hot)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_di_supply(&mut self, bank: IoBank) -> Result<IoSupply, SPIE, CSE> {
        self.select_io_bank(IoType::DigitalIn, bank)?;

        match self.sn65hvs881_read_single_bit(9) {
            Ok(level) => {
                if level {
                    Ok(IoSupply::Powered)
                } else {
                    Ok(IoSupply::Unpowered)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn max7301_write_single_port(&mut self, port_no: u8, level: IoLevel) -> Result<(), SPIE, CSE> {
        let mut buf = [0u8; 2];

        if port_no < 4 || port_no > 31 {
            return Err(IoError::IllegalOutput);
        }

        // set desired output state (high / low)
        buf[0] = 0x20 + port_no; // address for single port register
        match level {
            IoLevel::High => buf[1] = 1u8,
            IoLevel::HiZ => buf[1] = 0u8,
            _ => return Err(IoError::UnsupportedLevel),
        }
        self.max7301_transfer(&mut buf)?;
        Ok(())
    }

    fn max7301_read_single_port(&mut self, port_no: u8) -> Result<IoLevel, SPIE, CSE> {
        let mut buf = [0u8; 2];

        if port_no < 4 || port_no > 31 {
            return Err(IoError::IllegalOutput);
        }

        // send read request with address of single port register
        buf[0] = (0x80 | 0x20) + port_no;
        self.max7301_transfer(&mut buf)?;

        buf[0] = 0; // send NOP command and get read result
        self.max7301_transfer(&mut buf)?;

        if buf[1] != 0 {
            Ok(IoLevel::High)
        } else {
            Ok(IoLevel::Low)
        }
    }

    fn max7301_configure_output(&mut self, port_no: u8, level: IoLevel) -> Result<(), SPIE, CSE> {
        if port_no < 4 || port_no > 31 {
            return Err(IoError::IllegalOutput);
        }

        self.max7301_write_single_port(port_no, level)?;

        self.max7301_write_port_config_bits(port_no, 0x01)?;
        Ok(())
    }

    fn max7301_configure_input(&mut self, port_no: u8, en_pullup: bool) -> Result<(), SPIE, CSE> {
        let cfg_bits;
        if port_no < 4 || port_no > 31 {
            return Err(IoError::IllegalOutput);
        }

        if en_pullup {
            cfg_bits = 0x3;
        } else {
            cfg_bits = 0x2;
        }

        self.max7301_write_port_config_bits(port_no, cfg_bits)?;
        Ok(())
    }

    fn max7301_write_port_config_bits(
        &mut self,
        port_no: u8,
        mut cfg_bits: u8,
    ) -> Result<(), SPIE, CSE> {
        let mut buf = [0u8; 2];

        cfg_bits &= 0x03;

        // read corresponding port config register
        buf[0] = (0x80 | 0x08) + (port_no / 4);
        buf[1] = 0;
        self.max7301_transfer(&mut buf)?;

        buf = [0u8; 2];
        self.max7301_transfer(&mut buf)?;

        // modify config
        let mut config_reg = buf[1];
        let shift = (port_no & 0x03) * 2; // position of config bits
        config_reg &= !(0x03 << shift); // clear both config bits for this output
        config_reg |= cfg_bits << shift; // set requested bits

        // write config
        buf[0] = 0x08 + (port_no / 4);
        buf[1] = config_reg;
        self.max7301_transfer(&mut buf)?;
        Ok(())
    }

    fn max7301_wakeup(&mut self, en_irq: bool) -> Result<(), SPIE, CSE> {
        let mut buf = [0u8; 2];

        buf[0] = 0x04; // select configuration register
        if en_irq {
            buf[1] = 0x01 | 0x80;
        } else {
            buf[1] = 0x01;
        }

        self.max7301_transfer(&mut buf)?;
        Ok(())
    }

    fn max7301_transfer(&mut self, buf: &mut [u8]) -> Result<(), SPIE, CSE> {
        self.set_spi_mode(spi::MODE_0);
        self.ncs.set_low().map_err(IoError::Hal)?;
        self.delay.delay_us(2u32);
        self.spi.transfer(buf).map_err(IoError::Spi)?;
        self.delay.delay_us(2u32);
        self.ncs.set_high().map_err(IoError::Hal)?;
        self.delay.delay_us(2u32);
        Ok(())
    }

    fn sn65hvs881_read_single_bit(&mut self, port_no: u8) -> Result<bool, SPIE, CSE> {
        let mut buf = [0x50, 0x00];

        self.ld.set_low().map_err(IoError::Hal)?;
        self.delay.delay_us(2u32);
        self.ld.set_high().map_err(IoError::Hal)?;

        self.sn65hvs881_transfer(&mut buf)?;

        let combined = ((buf[1].reverse_bits() as u16) << 8) | buf[0] as u16;

        let parity;
        if combined & (1 << 10) != 0 {
            parity = 1;
        } else {
            parity = 0;
        }

        if (combined & 0xF800) != 0x5000 {
            return Err(IoError::DiParity);
        }

        if (combined & 0x03FF).count_ones() % 2 == parity {
            return Err(IoError::DiParity);
        }

        if combined & (1 << port_no) != 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn sn65hvs881_transfer(&mut self, buf: &mut [u8]) -> Result<(), SPIE, CSE> {
        self.set_spi_mode(spi::MODE_2);

        self.ncs.set_low().map_err(IoError::Hal)?;
        self.delay.delay_us(2u32);
        self.spi.transfer(buf).map_err(IoError::Spi)?;
        self.delay.delay_us(2u32);
        self.ncs.set_high().map_err(IoError::Hal)?;
        self.delay.delay_us(2u32);
        Ok(())
    }

    fn select_io_bank(&mut self, direction: IoType, bank: IoBank) -> Result<(), SPIE, CSE> {
        if direction == IoType::DigitalIn {
            match bank {
                IoBank::Bank1 => self.select_chip(5)?,
                IoBank::Bank2 => self.select_chip(4)?,
                IoBank::Bank3 => self.select_chip(3)?,
                IoBank::Bank4 => self.select_chip(2)?,
                IoBank::Bank5 => self.select_chip(1)?,
                IoBank::Bank6 => self.select_chip(0)?,
            }
        }
        if direction == IoType::DigitalOut {
            match bank {
                IoBank::Bank1 => self.select_chip(13)?,
                IoBank::Bank2 => self.select_chip(12)?,
                IoBank::Bank3 => self.select_chip(11)?,
                IoBank::Bank4 => self.select_chip(10)?,
                IoBank::Bank5 => self.select_chip(9)?,
                IoBank::Bank6 => self.select_chip(8)?,
            }
        }
        self.delay.delay_us(2u32);
        Ok(())
    }

    fn select_chip(&mut self, chip: u8) -> Result<(), SPIE, CSE> {
        let pins: [&mut dyn OutputPin<Error = CSE>; 4] = [
            &mut self.sel0,
            &mut self.sel1,
            &mut self.sel2,
            &mut self.sel3,
        ];

        for n in 0..pins.len() {
            if chip & (1 << n) != 0 {
                pins[n].set_high().map_err(IoError::Hal)?;
            } else {
                pins[n].set_low().map_err(IoError::Hal)?;
            }
        }
        Ok(())
    }

    fn set_spi_mode(&mut self, mode: Mode) {
        self.spi.set_spi_mode(mode);
    }
}

pub trait TkrtSpi {
    fn set_spi_mode(&mut self, mode: Mode);
}

impl TkrtSpi for Spi<SPI4, Enabled> {
    fn set_spi_mode(&mut self, mode: Mode) {
        self.inner().cr1.modify(|_, w| w.spe().clear_bit());
        self.inner().cfg2.modify(|_, w| {
            w.cpha()
                .bit(mode.phase == Phase::CaptureOnSecondTransition)
                .cpol()
                .bit(mode.polarity == Polarity::IdleHigh)
        });
        self.inner().cr1.modify(|_, w| w.spe().set_bit());
    }
}
