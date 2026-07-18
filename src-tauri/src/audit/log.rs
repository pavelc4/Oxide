use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static AUDIT: once_cell::sync::Lazy<Mutex<AuditLog>> =
    once_cell::sync::Lazy::new(|| Mutex::new(AuditLog::new()));

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub operation: String,
    pub serial: Option<String>,
    pub details: String,
}

pub struct AuditLog {
    path: Option<PathBuf>,
    enabled: bool,
    max_entries: usize,
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    fn new() -> Self {
        AuditLog {
            path: None,
            enabled: false,
            max_entries: 1000,
            entries: Vec::new(),
        }
    }

    pub fn init(data_dir: &PathBuf) {
        let mut log = AUDIT.lock().unwrap();
        let path = data_dir.join("audit.jsonl");
        log.path = Some(path.clone());
        log.enabled = true;
        let _ = fs::create_dir_all(data_dir);

        if let Ok(content) = fs::read_to_string(&path) {
            log.entries = content
                .lines()
                .filter_map(|l| serde_json::from_str::<AuditEntry>(l).ok())
                .collect();
            while log.entries.len() > log.max_entries {
                log.entries.remove(0);
            }
        }
    }

    pub fn set_enabled(enabled: bool) {
        AUDIT.lock().unwrap().enabled = enabled;
    }

    pub fn record(entry: AuditEntry) {
        let mut log = AUDIT.lock().unwrap();
        if !log.enabled {
            return;
        }
        log.entries.push(entry.clone());
        while log.entries.len() > log.max_entries {
            log.entries.remove(0);
        }
        if let Some(ref path) = log.path {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                if let Ok(line) = serde_json::to_string(&entry) {
                    let _ = writeln!(file, "{line}");
                }
            }
        }
    }

    pub fn recent(n: usize) -> Vec<AuditEntry> {
        let log = AUDIT.lock().unwrap();
        log.entries.iter().rev().take(n).cloned().collect()
    }

    pub fn all() -> Vec<AuditEntry> {
        AUDIT.lock().unwrap().entries.clone()
    }

    pub fn clear() {
        let mut log = AUDIT.lock().unwrap();
        log.entries.clear();
        if let Some(ref path) = log.path {
            let _ = fs::write(path, "");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        // ponytail: global static + parallel tests = tests can't assert
        // record/clear state reliably. Only test serialization here.
        let e = AuditEntry {
            timestamp: "now".into(),
            operation: "test_op".into(),
            serial: Some("abc".into()),
            details: "test details".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.operation, "test_op");
    }

    #[test]
    fn test_entry_serialize() {
        let e = AuditEntry {
            timestamp: "2024-01-01T00:00:00Z".into(),
            operation: "flash".into(),
            serial: Some("abc".into()),
            details: "flashed boot.img".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.operation, "flash");
    }
}
