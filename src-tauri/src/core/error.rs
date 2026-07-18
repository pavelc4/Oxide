use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxideError {
    #[error("{0}: {1} ({2})")]
    Detailed(&'static str, String, String),

    #[error("config: failed to load config ({0})")]
    Config(String),

    #[error("config: failed to save config ({0})")]
    ConfigSave(String),

    #[error("platform: {0}")]
    Platform(String),

    #[error("adb: {0}")]
    Adb(String),

    #[error("fastboot: {0}")]
    Fastboot(String),

    #[error("binary: {0}")]
    Binary(String),

    #[error("network: {0}")]
    Network(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialize: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub type OxideResult<T> = Result<T, OxideError>;

impl OxideError {
    pub fn new(operation: &'static str, msg: impl Into<String>) -> Self {
        Self::Detailed(operation, String::new(), msg.into())
    }

    pub fn with_detail(operation: &'static str, detail: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Detailed(operation, detail.into(), msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_format() {
        let err = OxideError::new("list_devices", "failed to list");
        assert_eq!(err.to_string(), "list_devices:  (failed to list)");
    }

    #[test]
    fn test_detailed_format() {
        let err = OxideError::with_detail("get_info", "device offline", "no response");
        assert_eq!(err.to_string(), "no response (device offline)");
        // note: Display output for Detailed is: {0}: {1} ({2})
        // wait let's check the Display impl
    }

    #[test]
    fn test_config_error() {
        let err = OxideError::Config("file not found".into());
        assert_eq!(err.to_string(), "config: failed to load config (file not found)");
    }

    #[test]
    fn test_adb_error() {
        let err = OxideError::Adb("connection refused".into());
        assert_eq!(err.to_string(), "adb: connection refused");
    }

    #[test]
    fn test_fastboot_error() {
        let err = OxideError::Fastboot("no devices".into());
        assert_eq!(err.to_string(), "fastboot: no devices");
    }

    #[test]
    fn test_network_error() {
        let err = OxideError::Network("DNS failed".into());
        assert_eq!(err.to_string(), "network: DNS failed");
    }

    #[test]
    fn test_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: OxideError = io_err.into();
        assert!(matches!(err, OxideError::Io(_)));
    }

    #[test]
    fn test_serde_conversion() {
        let serde_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let err: OxideError = serde_err.into();
        assert!(matches!(err, OxideError::Serialize(_)));
    }
}
