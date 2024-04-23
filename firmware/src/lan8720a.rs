//! SMSC LAN8720A Ethernet PHY

use stm32h7xx_hal::ethernet::{StationManagement, PHY};

#[allow(dead_code)]
mod phy_consts {
    pub const PHY_REG_BCR: u8 = 0x00;
    pub const PHY_REG_BSR: u8 = 0x01;
    pub const PHY_REG_ID1: u8 = 0x02;
    pub const PHY_REG_ID2: u8 = 0x03;
    // pub const PHY_REG_ANTX: u8 = 0x04;
    // pub const PHY_REG_ANRX: u8 = 0x05;
    // pub const PHY_REG_ANEXP: u8 = 0x06;
    // pub const PHY_REG_ANNPTX: u8 = 0x07;
    // pub const PHY_REG_ANNPRX: u8 = 0x08;
    // pub const PHY_REG_SSR: u8 = 0x1F; // Special Status Register
    // pub const PHY_REG_CTL: u8 = 0x0D; // Ethernet PHY Register Control
    // pub const PHY_REG_ADDAR: u8 = 0x0E; // Ethernet PHY Address or Data

    pub const PHY_REG_WUCSR: u16 = 0x8010;

    pub const PHY_REG_BCR_COLTEST: u16 = 1 << 7;
    pub const PHY_REG_BCR_FD: u16 = 1 << 8;
    pub const PHY_REG_BCR_ANRST: u16 = 1 << 9;
    pub const PHY_REG_BCR_ISOLATE: u16 = 1 << 10;
    pub const PHY_REG_BCR_POWERDN: u16 = 1 << 11;
    pub const PHY_REG_BCR_AN: u16 = 1 << 12;
    pub const PHY_REG_BCR_100M: u16 = 1 << 13;
    pub const PHY_REG_BCR_LOOPBACK: u16 = 1 << 14;
    pub const PHY_REG_BCR_RESET: u16 = 1 << 15;

    pub const PHY_REG_BSR_JABBER: u16 = 1 << 1;
    pub const PHY_REG_BSR_UP: u16 = 1 << 2;
    pub const PHY_REG_BSR_ANABLE: u16 = 1 << 3;
    pub const PHY_REG_BSR_FAULT: u16 = 1 << 4;
    pub const PHY_REG_BSR_ANDONE: u16 = 1 << 5;
    pub const PHY_REG_BSR_EXTST: u16 = 1 << 8;
    pub const PHY_REG_BSR_100BASE_T2_HD: u16 = 1 << 9;
    pub const PHY_REG_BSR_100BASE_T2_FD: u16 = 1 << 10;
    pub const PHY_REG_BSR_10BASE_T_HD: u16 = 1 << 11;
    pub const PHY_REG_BSR_10BASE_T_FD: u16 = 1 << 12;
    pub const PHY_REG_BSR_100BASE_TX_HD: u16 = 1 << 13;
    pub const PHY_REG_BSR_100BASE_TX_FD: u16 = 1 << 14;
    pub const PHY_REG_BSR_100BASE_T4: u16 = 1 << 15;
}
use self::phy_consts::*;

/// SMSC LAN8742A Ethernet PHY
pub struct LAN8720A<MAC: StationManagement> {
    mac: MAC,
}

impl<MAC: StationManagement> PHY for LAN8720A<MAC> {
    /// Reset PHY and wait for it to come out of reset.
    fn phy_reset(&mut self) {
        self.mac.smi_write(PHY_REG_BCR, PHY_REG_BCR_RESET);
        while self.mac.smi_read(PHY_REG_BCR) & PHY_REG_BCR_RESET == PHY_REG_BCR_RESET {}
    }

    /// PHY initialisation.
    fn phy_init(&mut self) {
        // Enable auto-negotiation
        self.mac.smi_write(
            PHY_REG_BCR,
            PHY_REG_BCR_AN | PHY_REG_BCR_ANRST | PHY_REG_BCR_100M,
        );
    }
}

/// Public functions for the LAN8742A
impl<MAC: StationManagement> LAN8720A<MAC> {
    /// Create LAN8742A instance from ETHMAC peripheral
    pub fn new(mac: MAC) -> Self {
        LAN8720A { mac }
    }
    /// Returns a reference to the inner ETHMAC peripheral
    pub fn inner(&self) -> &MAC {
        &self.mac
    }
    /// Returns a mutable reference to the inner ETHMAC peripheral
    pub fn inner_mut(&mut self) -> &mut MAC {
        &mut self.mac
    }
    /// Releases the ETHMAC peripheral
    pub fn free(self) -> MAC {
        self.mac
    }

    /// Poll PHY to determine link status.
    pub fn poll_link(&mut self) -> bool {
        let bsr = self.mac.smi_read(PHY_REG_BSR);

        // No link without autonegotiate
        if bsr & PHY_REG_BSR_ANDONE == 0 {
            return false;
        }
        // No link if link is down
        if bsr & PHY_REG_BSR_UP == 0 {
            return false;
        }
        // No link if autonegotiate incomplete
        if bsr & PHY_REG_BSR_ANDONE == 0 {
            return false;
        }
        // No link if other side isn't 100Mbps full duplex
        if bsr & PHY_REG_BSR_100BASE_TX_FD == 0 {
            return false;
        }

        // Got link
        true
    }

    pub fn link_established(&mut self) -> bool {
        self.poll_link()
    }

    pub fn block_until_link(&mut self) {
        while !self.link_established() {}
    }
}
