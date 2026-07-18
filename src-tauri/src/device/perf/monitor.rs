use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

use crate::device::core::info::parse_meminfo_output;
use crate::device::types::BatteryInfo;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CpuInfo {
    pub user: f32,
    pub system: f32,
    pub idle: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RamInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PerfSnapshot {
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub battery: Option<BatteryInfo>,
}

fn shell_read(device: &mut ADBServerDevice, cmd: &str) -> Option<String> {
    let mut out = Vec::new();
    device.shell_command(&cmd, Some(&mut out), None).ok()?;
    let text = String::from_utf8_lossy(&out).to_string();
    if text.trim().is_empty() { None } else { Some(text) }
}

pub fn get_cpu(device: &mut ADBServerDevice) -> CpuInfo {
    let text = shell_read(device, "cat /proc/stat").unwrap_or_default();
    parse_cpu_output(&text)
}

pub fn get_ram(device: &mut ADBServerDevice) -> RamInfo {
    let text = shell_read(device, "cat /proc/meminfo").unwrap_or_default();
    parse_ram_output(&text)
}

pub fn get_battery(device: &mut ADBServerDevice) -> Option<BatteryInfo> {
    let text = shell_read(device, "dumpsys battery")?;
    Some(crate::device::core::info::parse_battery_output(&text))
}

pub fn get_snapshot(device: &mut ADBServerDevice) -> PerfSnapshot {
    PerfSnapshot {
        cpu: get_cpu(device),
        ram: get_ram(device),
        battery: get_battery(device),
    }
}

pub fn parse_cpu_output(output: &str) -> CpuInfo {
    let line = output.lines().next().unwrap_or("");
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return CpuInfo { user: 0.0, system: 0.0, idle: 0.0 };
    }

    let user = parts[1].parse::<f64>().unwrap_or(0.0);
    let nice = parts[2].parse::<f64>().unwrap_or(0.0);
    let system = parts[3].parse::<f64>().unwrap_or(0.0);
    let idle = parts[4].parse::<f64>().unwrap_or(0.0);
    let total = user + nice + system + idle;

    if total == 0.0 {
        return CpuInfo { user: 0.0, system: 0.0, idle: 0.0 };
    }

    CpuInfo {
        user: ((user + nice) / total * 100.0) as f32,
        system: (system / total * 100.0) as f32,
        idle: (idle / total * 100.0) as f32,
    }
}

pub fn parse_ram_output(output: &str) -> RamInfo {
    let total = parse_meminfo_output(output).unwrap_or(0);
    let mut free = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;

    for line in output.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once(':') {
            let val = val.trim().split_whitespace().next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) * 1024;
            match key {
                "MemFree" => free = val,
                "Buffers" => buffers = val,
                "Cached" => cached = val,
                _ => {}
            }
        }
    }

    let used = total.saturating_sub(free.saturating_add(buffers).saturating_add(cached));
    RamInfo { total, used, free }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu() {
        let output = "cpu  12345 678 9012 345678 90 12 34 0 0 0\n";
        let cpu = parse_cpu_output(output);
        assert!(cpu.idle > cpu.user);
        assert!(cpu.idle > cpu.system);
        assert!((cpu.user + cpu.system + cpu.idle - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_parse_cpu_empty() {
        let cpu = parse_cpu_output("");
        assert_eq!(cpu.user, 0.0);
    }

    #[test]
    fn test_parse_cpu_single_core() {
        let output = "cpu  100 0 200 700 0 0 0 0 0 0\n";
        let cpu = parse_cpu_output(output);
        assert!((cpu.user - 10.0).abs() < 0.1);
        assert!((cpu.system - 20.0).abs() < 0.1);
        assert!((cpu.idle - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_ram() {
        let output = "MemTotal:        8000000 kB\nMemFree:         2000000 kB\nBuffers:          500000 kB\nCached:          1000000 kB\n";
        let ram = parse_ram_output(output);
        assert_eq!(ram.total, 8_000_000 * 1024);
        assert_eq!(ram.free, 2_000_000 * 1024);
        assert_eq!(ram.used, (8_000_000 - 2_000_000 - 500_000 - 1_000_000) * 1024);
    }

    #[test]
    fn test_parse_ram_empty() {
        let ram = parse_ram_output("");
        assert_eq!(ram.total, 0);
    }
}
