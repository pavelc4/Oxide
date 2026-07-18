use crate::core::config::AppConfig;
use std::net::TcpStream;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SetupStatus {
    pub adb_available: bool,
    pub fastboot_available: bool,
    pub config_exists: bool,
    pub completed: bool,
}

pub fn check_status() -> SetupStatus {
    SetupStatus {
        adb_available: check_adb_server(),
        fastboot_available: false,
        config_exists: AppConfig::load().is_ok(),
        completed: AppConfig::load().map(|c| c.setup_completed).unwrap_or(false),
    }
}

fn check_adb_server() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:5037".parse().unwrap(),
        std::time::Duration::from_secs(1),
    )
    .is_ok()
}

pub fn complete_setup() -> Result<(), String> {
    let mut config = AppConfig::load().unwrap_or_default();
    config.setup_completed = true;
    config.save().map_err(|e| format!("save config: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_returns() {
        let s = check_status();
        let _ = format!("{:?}", s);
    }
}
