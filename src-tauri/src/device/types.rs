use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceMode {
    Adb,
    Fastboot,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceState {
    Device,
    Recovery,
    Sideload,
    Bootloader,
    Offline,
    Unauthorized,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub serial: String,
    pub mode: DeviceMode,
    pub state: DeviceState,
    pub product: Option<String>,
    pub model: Option<String>,
    pub brand: Option<String>,
    pub device: Option<String>,
    pub sdk_version: Option<String>,
    pub battery_level: Option<u8>,
    pub storage_total: Option<u64>,
    pub storage_used: Option<u64>,
    pub ram_total: Option<u64>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub fingerprint: String,
    pub display_id: String,
    pub build_type: String,
    pub build_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: u8,
    pub status: String,
    pub temperature: Option<f32>,
    pub technology: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub serial: String,
    pub mode: DeviceMode,
    pub state: DeviceState,
    pub model: Option<String>,
    pub sdk: Option<String>,
    pub battery: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_default() {
        let info = DeviceInfo {
            serial: "abc123".into(),
            mode: DeviceMode::Adb,
            state: DeviceState::Device,
            product: None,
            model: Some("Pixel 7".into()),
            brand: Some("google".into()),
            device: None,
            sdk_version: Some("34".into()),
            battery_level: Some(85),
            storage_total: Some(128_000_000_000),
            storage_used: Some(64_000_000_000),
            ram_total: Some(8_000_000_000),
            ip_address: None,
        };
        assert_eq!(info.serial, "abc123");
        assert_eq!(info.model, Some("Pixel 7".into()));
    }

    #[test]
    fn test_device_summary() {
        let summary = DeviceSummary {
            serial: "xyz789".into(),
            mode: DeviceMode::Fastboot,
            state: DeviceState::Bootloader,
            model: Some("OnePlus 12".into()),
            sdk: None,
            battery: None,
        };
        assert_eq!(summary.mode, DeviceMode::Fastboot);
        assert_eq!(summary.state, DeviceState::Bootloader);
    }

    #[test]
    fn test_device_mode_serde() {
        let json = serde_json::to_string(&DeviceMode::Adb).unwrap();
        assert_eq!(json, "\"adb\"");
        let parsed: DeviceMode = serde_json::from_str("\"fastboot\"").unwrap();
        assert_eq!(parsed, DeviceMode::Fastboot);
    }

    #[test]
    fn test_device_state_serde() {
        let json = serde_json::to_string(&DeviceState::Device).unwrap();
        assert_eq!(json, "\"device\"");
        let parsed: DeviceState = serde_json::from_str("\"offline\"").unwrap();
        assert_eq!(parsed, DeviceState::Offline);
    }

    #[test]
    fn test_battery_info() {
        let b = BatteryInfo {
            level: 50,
            status: "charging".into(),
            temperature: Some(36.5),
            technology: Some("Li-ion".into()),
        };
        assert_eq!(b.level, 50);
    }

    #[test]
    fn test_storage_info() {
        let s = StorageInfo {
            total: 256_000_000_000,
            used: 100_000_000_000,
            free: 156_000_000_000,
        };
        assert_eq!(s.total - s.used, s.free);
    }
}
