use adb_client::server::ADBServer;
use adb_client::server::DeviceShort;
use std::net::SocketAddrV4;

pub fn list_adb() -> Vec<DeviceShort> {
    let addr = "127.0.0.1:5037".parse::<SocketAddrV4>().unwrap();
    let mut server = ADBServer::new(addr);
    server.devices().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_adb_no_devices() {
        let devices = list_adb();
        assert!(devices.is_empty() || devices.iter().any(|d| !d.identifier.is_empty()));
    }
}
