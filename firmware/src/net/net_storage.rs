use smoltcp::{
    iface::SocketStorage,
    socket::{icmp, udp},
    wire::{IpCidr, Ipv4Address, Ipv4Cidr},
};

use super::ethernet::{
    ICMP_RX_BUFFER_SIZE, ICMP_SOCKET_METADATA_COUNT, ICMP_TX_BUFFER_SIZE, NUM_SOCKETS,
    TCP_RX_SOCKET_BUFFER_SIZE, TCP_TX_SOCKET_BUFFER_SIZE, UDP_RX_SOCKET_BUFFER_SIZE,
    UDP_SOCKET_METADATA_COUNT, UDP_TX_SOCKET_BUFFER_SIZE,
};

pub struct NetStorage {
    pub ip_addrs: [IpCidr; 1],
    pub sockets: [SocketStorage<'static>; NUM_SOCKETS],
    pub icmp_socket_storage: ICMPSocketStorage,
    pub flash_tcp_socket_storage: TcpSocketStorage,
    pub control_server_udp_storage: UdpSocketStorage,
    pub control_server_tcp_storage: TcpSocketStorage,
}

impl NetStorage {
    pub const fn new() -> Self {
        Self {
            // NOTE: IP address set at runtime
            ip_addrs: [IpCidr::Ipv4(Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 24)); 1],
            sockets: [SocketStorage::EMPTY; NUM_SOCKETS],
            icmp_socket_storage: ICMPSocketStorage::new(),
            flash_tcp_socket_storage: TcpSocketStorage::INIT,
            control_server_udp_storage: UdpSocketStorage::new(),
            control_server_tcp_storage: TcpSocketStorage::INIT,
        }
    }
}

pub struct TcpSocketStorage {
    pub rx_storage: [u8; TCP_RX_SOCKET_BUFFER_SIZE],
    pub tx_storage: [u8; TCP_TX_SOCKET_BUFFER_SIZE],
}

impl TcpSocketStorage {
    const INIT: Self = Self::new();

    const fn new() -> Self {
        Self {
            rx_storage: [0; TCP_RX_SOCKET_BUFFER_SIZE],
            tx_storage: [0; TCP_TX_SOCKET_BUFFER_SIZE],
        }
    }
}

pub struct UdpSocketStorage {
    pub rx_metadata: [udp::PacketMetadata; UDP_SOCKET_METADATA_COUNT],
    pub rx_storage: [u8; UDP_RX_SOCKET_BUFFER_SIZE],
    pub tx_metadata: [udp::PacketMetadata; UDP_SOCKET_METADATA_COUNT],
    pub tx_storage: [u8; UDP_TX_SOCKET_BUFFER_SIZE],
}

impl UdpSocketStorage {
    pub const fn new() -> Self {
        Self {
            rx_metadata: [udp::PacketMetadata::EMPTY; UDP_SOCKET_METADATA_COUNT],
            rx_storage: [0; UDP_RX_SOCKET_BUFFER_SIZE],
            tx_metadata: [udp::PacketMetadata::EMPTY; UDP_SOCKET_METADATA_COUNT],
            tx_storage: [0; UDP_TX_SOCKET_BUFFER_SIZE],
        }
    }
}

pub struct ICMPSocketStorage {
    pub rx_storage: [u8; ICMP_RX_BUFFER_SIZE],
    pub rx_metadata: [icmp::PacketMetadata; ICMP_SOCKET_METADATA_COUNT],
    pub tx_storage: [u8; ICMP_TX_BUFFER_SIZE],
    pub tx_metadata: [icmp::PacketMetadata; ICMP_SOCKET_METADATA_COUNT],
}

impl ICMPSocketStorage {
    const fn new() -> Self {
        Self {
            rx_storage: [0; ICMP_RX_BUFFER_SIZE],
            rx_metadata: [icmp::PacketMetadata::EMPTY; ICMP_SOCKET_METADATA_COUNT],
            tx_storage: [0; ICMP_TX_BUFFER_SIZE],
            tx_metadata: [icmp::PacketMetadata::EMPTY; ICMP_SOCKET_METADATA_COUNT],
        }
    }
}
