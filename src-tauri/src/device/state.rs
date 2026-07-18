use adb_client::server_device::ADBServerDevice;

pub fn connect_serial(serial: &str) -> ADBServerDevice {
    ADBServerDevice::new(serial.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_serial_sets_identifier() {
        let dev = connect_serial("test_serial");
        assert_eq!(dev.identifier, Some("test_serial".to_string()));
    }
}
