use std::collections::HashMap;

use fastboot_protocol::nusb::{self, NusbFastBoot};

#[derive(Debug, Clone, serde::Serialize)]
pub struct FastbootDevice {
    pub serial: Option<String>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub bus_info: String,
}

pub async fn list_devices() -> Vec<FastbootDevice> {
    let Ok(infos) = nusb::devices().await else { return vec![] };
    let mut out = Vec::new();
    for info in infos {
        let fb = NusbFastBoot::from_info(&info).await;
        let serial = match fb {
            Ok(mut f) => f.get_var("serialno").await.ok(),
            Err(_) => None,
        };
        out.push(FastbootDevice {
            serial,
            vendor_id: info.vendor_id(),
            product_id: info.product_id(),
            bus_info: format!("{}:{}", info.bus_id(), info.device_address()),
        });
    }
    out
}

async fn open(serial: &str) -> Result<NusbFastBoot, String> {
    let infos = nusb::devices().await.map_err(|e| format!("list fastboot devices: {e}"))?;
    for info in infos {
        if let Ok(mut fb) = NusbFastBoot::from_info(&info).await {
            if let Ok(s) = fb.get_var("serialno").await {
                if s == serial {
                    return Ok(fb);
                }
            }
        }
    }
    Err(format!("fastboot device {serial} not found"))
}

pub async fn get_var(serial: &str, var: &str) -> Result<String, String> {
    let mut fb = open(serial).await?;
    fb.get_var(var).await.map_err(|e| format!("getvar {var}: {e}"))
}

pub async fn get_all_vars(serial: &str) -> Result<HashMap<String, String>, String> {
    let mut fb = open(serial).await?;
    fb.get_all_vars().await.map_err(|e| format!("getvar all: {e}"))
}

pub async fn flash(serial: &str, partition: &str, data: &[u8]) -> Result<(), String> {
    let mut fb = open(serial).await?;
    let size = data.len() as u32;
    let mut sender = fb.download(size).await.map_err(|e| format!("download: {e}"))?;
    sender.extend_from_slice(data).await.map_err(|e| format!("send data: {e}"))?;
    sender.finish().await.map_err(|e| format!("finish data: {e}"))?;
    fb.flash(partition).await.map_err(|e| format!("flash {partition}: {e}"))
}

pub async fn erase(serial: &str, partition: &str) -> Result<(), String> {
    let mut fb = open(serial).await?;
    fb.erase(partition).await.map_err(|e| format!("erase {partition}: {e}"))
}

pub async fn reboot(serial: &str) -> Result<(), String> {
    let mut fb = open(serial).await?;
    fb.reboot().await.map_err(|e| format!("reboot: {e}"))
}

pub async fn reboot_bootloader(serial: &str) -> Result<(), String> {
    let mut fb = open(serial).await?;
    fb.reboot_to("bootloader").await.map_err(|e| format!("reboot-bootloader: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastboot_device_structure() {
        let dev = FastbootDevice {
            serial: Some("abc123".into()),
            vendor_id: 0x18d1,
            product_id: 0x4ee0,
            bus_info: "1:2".into(),
        };
        assert_eq!(dev.serial.as_deref(), Some("abc123"));
        assert_eq!(dev.vendor_id, 0x18d1);
    }
}
