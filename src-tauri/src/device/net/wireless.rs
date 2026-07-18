use std::net::SocketAddrV4;

use adb_client::server::ADBServer;
use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

fn server() -> ADBServer {
    ADBServer::new("127.0.0.1:5037".parse::<SocketAddrV4>().unwrap())
}

fn parse_addr(host_port: &str) -> Result<SocketAddrV4, String> {
    let addr = host_port.parse::<SocketAddrV4>().map_err(|e| format!("invalid address: {e}"))?;
    Ok(addr)
}

pub fn connect_device(host_port: &str) -> Result<(), String> {
    let addr = parse_addr(host_port)?;
    let mut srv = server();
    srv.connect_device(addr).map_err(|e| format!("connect: {e}"))
}

pub fn disconnect_device(host_port: &str) -> Result<(), String> {
    let addr = parse_addr(host_port)?;
    let mut srv = server();
    srv.disconnect_device(addr).map_err(|e| format!("disconnect: {e}"))
}

pub fn pair_device(host_port: &str, code: &str) -> Result<(), String> {
    let addr = parse_addr(host_port)?;
    let mut srv = server();
    srv.pair(addr, code.to_string()).map_err(|e| format!("pair: {e}"))
}

pub fn enable_tcpip(device: &mut ADBServerDevice, port: u16) -> Result<(), String> {
    device.tcpip(port).map_err(|e| format!("tcpip: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addr_ok() {
        let addr = parse_addr("192.168.1.100:5555").unwrap();
        assert_eq!(addr.ip().to_string(), "192.168.1.100");
        assert_eq!(addr.port(), 5555);
    }

    #[test]
    fn test_parse_addr_bad_host() {
        assert!(parse_addr("not-an-ip:5555").is_err());
    }

    #[test]
    fn test_parse_addr_bad_port() {
        assert!(parse_addr("192.168.1.100:notaport").is_err());
    }

    #[test]
    fn test_parse_addr_missing_port() {
        assert!(parse_addr("192.168.1.100").is_err());
    }
}
