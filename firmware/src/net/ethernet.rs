use super::net_storage::NetStorage;
use crate::lan8720a::LAN8720A;

use core::ops::Add;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use firmware_logic::{data::user_commands::UserCommands, FirmwareReporting, ServerReporting, Timestamp, UserCommand, Wingman2HardwareStatus};
use rtic_monotonics::{systick::Systick, Monotonic};
use smoltcp::{
    iface::{Interface, SocketHandle, SocketSet},
    socket::{tcp, udp},
    wire::{EthernetAddress, IpAddress, IpCidr, IpEndpoint, Ipv4Address, Ipv4Cidr},
};
use stm32h7xx_hal::{
    delay::DelayFromCountDownTimer,
    device::{ETHERNET_DMA, ETHERNET_MAC, ETHERNET_MTL, TIM1},
    ethernet::{self, EthernetDMA, EthernetMAC, PinsRMII, PHY},
    rcc::{rec, CoreClocks},
    timer::Timer,
};

/// Ethernet descriptor rings are a global singleton
pub const ETH_DES_RING_TD: usize = 4;
pub const ETH_DES_RING_RD: usize = 4;
#[link_section = ".sram3.eth"]
static mut DES_RING: ethernet::DesRing<ETH_DES_RING_TD, ETH_DES_RING_RD> = ethernet::DesRing::new();

pub type EthDMA = EthernetDMA<ETH_DES_RING_TD, ETH_DES_RING_RD>;

/// Locally administered MAC address
const MAC_ADDRESS: EthernetAddress = EthernetAddress([0x02, 0x00, 0x11, 0x22, 0x33, 0x44]);
const IP_ADDRESS: Ipv4Address = Ipv4Address::new(10, 0, 90, 194); // /24 subnet mask
const FLASH_TCP_IP_LISTENING_PORT: u16 = 6971;
const CONTROL_SERVER_UDP_LISTENING_PORT: u16 = 6972;
const CONTROL_SERVER_TCP_LISTENING_PORT: u16 = 6973;

const SERVER_IP_ADDRESS: IpAddress = IpAddress::v4(10, 0, 90, 149);

const CONTROL_SERVER_UDP_ENDPOINT: IpEndpoint =
    IpEndpoint::new(SERVER_IP_ADDRESS, CONTROL_SERVER_UDP_LISTENING_PORT);

const CONTROL_SERVER_TCP_ENDPOINT: IpEndpoint = IpEndpoint::new(
    IpAddress::Ipv4(IP_ADDRESS),
    CONTROL_SERVER_TCP_LISTENING_PORT,
);

pub const NUM_FLASH_TCP_SOCKETS: usize = 1;
pub const NUM_CONTROL_SERVER_SOCKETS: usize = 2;
pub const NUM_SOCKETS: usize = NUM_FLASH_TCP_SOCKETS + NUM_CONTROL_SERVER_SOCKETS;

pub const UDP_RX_SOCKET_BUFFER_SIZE: usize = 4_096;
pub const UDP_TX_SOCKET_BUFFER_SIZE: usize = 4_096;
pub const UDP_SOCKET_METADATA_COUNT: usize = 10;

pub const TCP_RX_SOCKET_BUFFER_SIZE: usize = 4_096;
pub const TCP_TX_SOCKET_BUFFER_SIZE: usize = 4_096;

pub const ICMP_RX_BUFFER_SIZE: usize = 512;
pub const ICMP_TX_BUFFER_SIZE: usize = 512;
pub const ICMP_SOCKET_METADATA_COUNT: usize = 512;

pub struct Ethernet {
    timer: Timer<TIM1>,
    eth_dma: EthDMA,
    eth_mac: EthernetMAC,
    iface: Interface,
    flash_tcp_socket_handle: SocketHandle,
    control_server_udp_socket_handle: SocketHandle,
    control_server_tcp_socket_handle: SocketHandle,
    socket_set: SocketSet<'static>,

    latest_control_server_timestamp: Option<smoltcp::time::Instant>,
    is_connected_to_control_server: bool,
}

impl Ethernet {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        eth_mac: ETHERNET_MAC,
        eth_mtl: ETHERNET_MTL,
        eth_dma: ETHERNET_DMA,
        pins: impl PinsRMII,
        prec: rec::Eth1Mac,
        timer: Timer<TIM1>,
        clocks: &CoreClocks,
        storage: &'static mut NetStorage,
    ) -> Self {
        rtt_debug!("Initializing Ethernet ...");

        let (mut eth_dma, eth_mac) = unsafe {
            ethernet::new(
                eth_mac,
                eth_mtl,
                eth_dma,
                pins,
                &mut DES_RING,
                MAC_ADDRESS,
                prec,
                clocks,
            )
        };

        let mut phy = LAN8720A::new(eth_mac);
        phy.phy_reset();
        phy.phy_init();

        let mut loop_time = smoltcp::time::Instant::from_millis(0);

        let mut delay = DelayFromCountDownTimer::new(timer);

        // TEST: wait for linkup on Ethernet
        loop {
            if phy.link_established() {
                rtt_debug!("ETH Link Established");
                break;
            }

            delay.delay_ms(10u16);
            loop_time = loop_time.add(smoltcp::time::Duration::from_millis(10));
            if loop_time.total_millis() > 5000i64 {
                rtt_debug!("ETH Link Timeout!!!");
                break;
            }
        }

        let config = smoltcp::iface::Config::new(MAC_ADDRESS.into());

        let timestamp = smoltcp::time::Instant::from_millis(Systick::now().ticks() as i64);
        let mut iface = smoltcp::iface::Interface::new(config, &mut eth_dma, timestamp);
        iface.update_ip_addrs(|ip_addresses| {
            if let Err(_err) = ip_addresses.push(IpCidr::Ipv4(Ipv4Cidr::new(IP_ADDRESS, 24))) {
                rtt_debug!("failed to push ip address");
            } else {
                // rtt_debug!("pushed back ip address");
            }
            // i.e. mask 255.255.255.0
        });

        let mut socket_set = SocketSet::new(&mut storage.sockets[..]);

        let rx_buffer =
            tcp::SocketBuffer::new(&mut storage.flash_tcp_socket_storage.rx_storage[..]);
        let tx_buffer =
            tcp::SocketBuffer::new(&mut storage.flash_tcp_socket_storage.tx_storage[..]);
        let mut flash_tcp_socket: tcp::Socket<'_> = tcp::Socket::new(rx_buffer, tx_buffer);
        if flash_tcp_socket.local_endpoint().is_some() {
            rtt_debug!("Binding TCP socket...");
            let local_endpoint = IpEndpoint::new(IP_ADDRESS.into(), FLASH_TCP_IP_LISTENING_PORT);
            match flash_tcp_socket.listen(local_endpoint) {
                Ok(()) => {
                    rtt_debug!("TCP socket listening");
                }
                Err(_e) => {
                    rtt_debug!("TCP socket listen error");
                }
            };
        }
        extern "C" {
            static _stack_start: u32;         // Top of stack (highest address)
            static _stack_size: u32;     // Total size of the stack
        }
        
        let stack_start = unsafe { &_stack_start as *const u32 as u32 };
        let stack_size = unsafe { &_stack_size as *const u32 as u32 };
        let mut sp: u32;
    
        unsafe {
            core::arch::asm!("mov {}, sp", out(reg) sp);
        }
    
        let stack_end = stack_start - stack_size; // Calculate the bottom of the stack
        let mut failed = false;
        if sp < stack_end {
            failed = true;
        }
        let flash_tcp_socket_handle = socket_set.add(flash_tcp_socket);

        let rx_buffer = udp::PacketBuffer::new(
            &mut storage.control_server_udp_storage.rx_metadata[..],
            &mut storage.control_server_udp_storage.rx_storage[..],
        );
        let tx_buffer = udp::PacketBuffer::new(
            &mut storage.control_server_udp_storage.tx_metadata[..],
            &mut storage.control_server_udp_storage.tx_storage[..],
        );
        let udp_socket = udp::Socket::new(rx_buffer, tx_buffer);

        let control_server_udp_socket_handle = socket_set.add(udp_socket);

        let rx_buffer =
            tcp::SocketBuffer::new(&mut storage.control_server_tcp_storage.rx_storage[..]);
        let tx_buffer =
            tcp::SocketBuffer::new(&mut storage.control_server_tcp_storage.tx_storage[..]);
        let mut control_center_tcp_socket: tcp::Socket<'_> = tcp::Socket::new(rx_buffer, tx_buffer);
        if control_center_tcp_socket.local_endpoint().is_some() {
            rtt_debug!("Binding TCP socket...");
            let local_endpoint =
                IpEndpoint::new(IP_ADDRESS.into(), CONTROL_SERVER_TCP_LISTENING_PORT);
            match control_center_tcp_socket.listen(local_endpoint) {
                Ok(()) => {
                    rtt_debug!("TCP socket listening");
                }
                Err(_e) => {
                    rtt_debug!("TCP socket listen error");
                }
            };
        }
        let control_server_tcp_socket_handle = socket_set.add(control_center_tcp_socket);

        rtt_debug!("Ethernet initialized");

        Self {
            timer: delay.free(),
            eth_dma,
            eth_mac: phy.free(),
            iface,
            socket_set,
            flash_tcp_socket_handle,
            control_server_udp_socket_handle,
            control_server_tcp_socket_handle,
            latest_control_server_timestamp: None,
            is_connected_to_control_server: false,
        }
    }

    pub fn loop_update(&mut self) {
        let timestamp = smoltcp::time::Instant::from_millis(Systick::now().ticks() as i64);
        self.iface
            .poll(timestamp, &mut self.eth_dma, &mut self.socket_set);

        #[allow(unreachable_code)]
        {
            // rprintln!("Exit Ethernet::loop_update!!!");
        }
    }

    pub fn is_connected_to_control_server(&self) -> bool {
        self.is_connected_to_control_server
    }

    pub fn synchronize_control_server_socket(
        &mut self,
        user_commands: &mut UserCommands,
        hardware_status: &Wingman2HardwareStatus,
        reporting: &FirmwareReporting,
    ) {
        let timestamp = smoltcp::time::Instant::from_millis(Systick::now().ticks() as i64);

        self.is_connected_to_control_server = self
            .latest_control_server_timestamp
            .map(|latest| latest.add(smoltcp::time::Duration::from_millis(5_000)) > timestamp)
            .unwrap_or(false);

        self.iface
            .poll(timestamp, &mut self.eth_dma, &mut self.socket_set);

        let tcp_socket = self
            .socket_set
            .get_mut::<tcp::Socket>(self.control_server_tcp_socket_handle);

        if !tcp_socket.is_open() {
            tcp_socket.listen(CONTROL_SERVER_TCP_ENDPOINT).unwrap();
        }
        if !tcp_socket.may_recv() && tcp_socket.may_send() {
            tcp_socket.close();
        }

        if tcp_socket.can_recv() {
            let mut buf = [0u8; 1024];
            match tcp_socket.recv_slice(&mut buf) {
                Ok(size) => {
                    match postcard::from_bytes::<UserCommand>(&buf[..size])
                    {
                        Ok(mut command_received) => {
                            let timestamp = Timestamp::new(Systick::now().ticks());
                            command_received.set_timestamp(timestamp);
                        }
                        Err(_err) => {
                            rtt_debug!("Error receiving from control server");
                            // TODO: SHOW ERROR ON DISPLAY
                        } // self.latest_control_server_timestamp = Some(timestamp);
                          // card_response.update_control_center_qos();

                          // card_status.update_from(&card_response);
                    }
                }
                Err(err) => {
                    rtt_debug!("Error receiving from control server {}", err);
                }
            }
        }

        let udp_socket = self
            .socket_set
            .get_mut::<udp::Socket>(self.control_server_udp_socket_handle);

        if !udp_socket.endpoint().is_specified() {
            rtt_debug!("Binding UDP socket...");
            let local_endpoint =
                IpEndpoint::new(IP_ADDRESS.into(), CONTROL_SERVER_UDP_LISTENING_PORT);
            match udp_socket.bind(local_endpoint) {
                Ok(()) => rtt_debug!("UDP socket bound."),
                Err(_e) => rtt_debug!("UDP socket bind error"),
            };
        }
        
        if udp_socket.can_recv() {
            if let Ok((data, _)) = udp_socket.recv() {
                let _server_reporting = postcard::from_bytes::<ServerReporting>(&data).unwrap();
            }
        }

        if udp_socket.can_send() {
            let mut buf = [0u8; 1024];
            let buf = postcard::to_slice(hardware_status, &mut buf).unwrap();
            send_udp_slice(udp_socket, buf, CONTROL_SERVER_UDP_ENDPOINT);

            let mut buf = [0u8; 1024];
            let buf = postcard::to_slice(reporting, &mut buf).unwrap();
            send_udp_slice(udp_socket, buf, CONTROL_SERVER_UDP_ENDPOINT);
        }
    }
}

fn send_tcp_slice(socket: &mut tcp::Socket, buf: &[u8]) {
    match socket.send_slice(buf) {
        Ok(_) => {
            rtt_debug!("Sent status to control server");
        }
        Err(err) => {
            rtt_debug!("Error sending value to control server {}", err);
        }
    }
}

fn send_udp_slice(socket: &mut udp::Socket, buf: &[u8], endpoint: IpEndpoint) {
    match socket.send_slice(buf, endpoint) {
        Ok(_) => {
            rtt_debug!("Sent status to control server");
        }
        Err(err) => {
            rtt_debug!("Error sending value to control server {}", err);
        }
    }
}
