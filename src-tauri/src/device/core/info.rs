use std::collections::HashMap;

use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

use crate::device::types::{BatteryInfo, StorageInfo, DeviceInfo, DeviceMode, DeviceState};

pub fn get_info(device: &mut ADBServerDevice) -> DeviceInfo {
    let props = get_props(device);
    let battery = fetch_battery(device);
    let storage = fetch_storage(device);
    let ram = fetch_ram(device);
    let ip = get_ip(device);

    DeviceInfo {
        serial: device.identifier.clone().unwrap_or_default(),
        mode: DeviceMode::Adb,
        state: DeviceState::Device,
        product: props.get("ro.product.name").cloned(),
        model: props.get("ro.product.model").cloned(),
        brand: props.get("ro.product.brand").cloned(),
        device: props.get("ro.product.device").cloned(),
        sdk_version: props.get("ro.build.version.sdk").cloned(),
        battery_level: battery.map(|b| b.level),
        storage_total: storage.as_ref().map(|s| s.total),
        storage_used: storage.as_ref().map(|s| s.used),
        ram_total: ram,
        ip_address: ip,
    }
}

fn get_props(device: &mut ADBServerDevice) -> HashMap<String, String> {
    let mut out = Vec::new();
    if device.shell_command(&"getprop", Some(&mut out), None).is_err() {
        return HashMap::new();
    }
    let text = String::from_utf8_lossy(&out);
    parse_getprop_output(&text)
}

fn fetch_battery(device: &mut ADBServerDevice) -> Option<BatteryInfo> {
    let mut out = Vec::new();
    device.shell_command(&"dumpsys battery", Some(&mut out), None).ok()?;
    let text = String::from_utf8_lossy(&out);
    Some(parse_battery_output(&text))
}

fn fetch_storage(device: &mut ADBServerDevice) -> Option<StorageInfo> {
    let mut out = Vec::new();
    device.shell_command(&"df /data", Some(&mut out), None).ok()?;
    let text = String::from_utf8_lossy(&out);
    parse_df_output(&text)
}

fn fetch_ram(device: &mut ADBServerDevice) -> Option<u64> {
    let mut out = Vec::new();
    device.shell_command(&"cat /proc/meminfo", Some(&mut out), None).ok()?;
    let text = String::from_utf8_lossy(&out);
    parse_meminfo_output(&text)
}

fn get_ip(device: &mut ADBServerDevice) -> Option<String> {
    let keys = ["dhcp.wlan0.ipaddress", "dhcp.eth0.ipaddress", "dhcp.wlan1.ipaddress"];
    for key in &keys {
        let mut out = Vec::new();
        if device.shell_command(&format!("getprop {key}"), Some(&mut out), None).is_err() {
            continue;
        }
        let val = String::from_utf8_lossy(&out).trim().to_string();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

pub fn parse_getprop_output(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once("]: [") {
            let key = key.trim_start_matches('[').to_string();
            let val = val.trim_end_matches(']').to_string();
            map.insert(key, val);
        }
    }
    map
}

pub fn parse_battery_output(output: &str) -> BatteryInfo {
    let mut level = 0u8;
    let mut status = "unknown".to_string();
    let mut temperature = None;
    let mut technology = None;

    for line in output.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once(": ") {
            match key {
                "level" => level = val.trim().parse().unwrap_or(0),
                "status" => status = parse_battery_status(val.trim()),
                "temperature" => temperature = val.trim().parse::<f32>().ok().map(|t| t / 10.0),
                "technology" => technology = Some(val.trim().to_string()),
                _ => {}
            }
        }
    }

    BatteryInfo { level, status, temperature, technology }
}

fn parse_battery_status(code: &str) -> String {
    match code {
        "1" => "unknown",
        "2" => "charging",
        "3" => "discharging",
        "4" => "not_charging",
        "5" => "full",
        _ => code,
    }.to_string()
}

pub fn parse_df_output(output: &str) -> Option<StorageInfo> {
    let mut lines = output.lines();
    lines.next()?;
    let line = lines.next()?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 { return None; }
    let total = parts[1].parse::<u64>().ok()? * 1024;
    let used = parts[2].parse::<u64>().ok()? * 1024;
    let free = parts[3].parse::<u64>().ok()? * 1024;
    Some(StorageInfo { total, used, free })
}

pub(crate) fn parse_meminfo_output(output: &str) -> Option<u64> {
    for line in output.lines() {
        if line.starts_with("MemTotal:") {
            let val = line.split_whitespace().nth(1)?;
            let kb: u64 = val.parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_getprop() {
        let output = "[ro.product.model]: [Pixel 7]\n[ro.build.version.sdk]: [34]\n[ro.product.brand]: [google]\n";
        let map = parse_getprop_output(output);
        assert_eq!(map.get("ro.product.model").unwrap(), "Pixel 7");
        assert_eq!(map.get("ro.build.version.sdk").unwrap(), "34");
        assert_eq!(map.get("ro.product.brand").unwrap(), "google");
    }

    #[test]
    fn test_parse_getprop_empty() {
        let map = parse_getprop_output("");
        assert!(map.is_empty());
    }

    #[test]
    fn test_parse_getprop_malformed() {
        let map = parse_getprop_output("not a prop line\n[partial]: [val\nnope]");
        assert!(map.is_empty() || map.len() == 1);
    }

    #[test]
    fn test_parse_battery() {
        let output = "  level: 85\n  status: 2\n  temperature: 300\n  technology: Li-ion\n";
        let b = parse_battery_output(output);
        assert_eq!(b.level, 85);
        assert_eq!(b.status, "charging");
        assert_eq!(b.temperature, Some(30.0));
        assert_eq!(b.technology, Some("Li-ion".into()));
    }

    #[test]
    fn test_parse_battery_defaults() {
        let b = parse_battery_output("");
        assert_eq!(b.level, 0);
        assert_eq!(b.status, "unknown");
    }

    #[test]
    fn test_parse_battery_status_codes() {
        assert_eq!(parse_battery_output("  status: 5\n").status, "full");
        assert_eq!(parse_battery_output("  status: 3\n").status, "discharging");
        assert_eq!(parse_battery_output("  status: 4\n").status, "not_charging");
        assert_eq!(parse_battery_output("  status: 1\n").status, "unknown");
    }

    #[test]
    fn test_parse_df() {
        let output = "Filesystem      1K-blocks     Used    Available Use% Mounted on\n\
                       /data           12345678   6543210    5802468  53% /data\n";
        let s = parse_df_output(output).unwrap();
        assert_eq!(s.total, 12345678 * 1024);
        assert_eq!(s.used, 6543210 * 1024);
        assert_eq!(s.free, 5802468 * 1024);
    }

    #[test]
    fn test_parse_df_no_header() {
        assert!(parse_df_output("").is_none());
    }

    #[test]
    fn test_parse_meminfo() {
        let output = "MemTotal:       8187124 kB\nMemFree:        4123456 kB\n";
        let ram = parse_meminfo_output(output).unwrap();
        assert_eq!(ram, 8187124 * 1024);
    }

    #[test]
    fn test_parse_meminfo_missing() {
        assert!(parse_meminfo_output("MemFree: 1234 kB\n").is_none());
    }
}
