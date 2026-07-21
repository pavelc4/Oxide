use std::io::Read;

use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;
use adb_client::{ADBListItem, ADBListItemType};

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub modified: u32,
}

pub fn list_files(device: &mut ADBServerDevice, path: &str) -> Vec<FileEntry> {
    match device.list(path) {
        Ok(items) => items.into_iter().map(convert_entry).collect(),
        Err(_) => vec![],
    }
}

pub fn pull_file(device: &mut ADBServerDevice, remote: &str, local: &mut dyn std::io::Write) -> Result<(), String> {
    device.pull(&remote, local).map_err(|e| format!("pull: {e}"))
}

pub fn pull_item(device: &mut ADBServerDevice, remote: &str, local: &str) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(local).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let items = list_files(device, remote);
    if items.is_empty() {
        let mut file = std::fs::File::create(local).map_err(|e| format!("create {local}: {e}"))?;
        return pull_file(device, remote, &mut file);
    }

    std::fs::create_dir_all(local).map_err(|e| format!("mkdir {local}: {e}"))?;
    for item in items {
        let child_remote = format!("{}/{}", remote.trim_end_matches('/'), item.name);
        let child_local = std::path::Path::new(local).join(&item.name);
        let child_local_str = child_local.to_string_lossy();
        if item.is_dir {
            pull_item(device, &child_remote, &child_local_str)?;
        } else {
            if let Some(p) = child_local.parent() {
                let _ = std::fs::create_dir_all(p);
            }
            let mut file = std::fs::File::create(&*child_local_str).map_err(|e| format!("create {child_local_str}: {e}"))?;
            let _ = pull_file(device, &child_remote, &mut file);
        }
    }
    Ok(())
}

pub fn push_file(device: &mut ADBServerDevice, data: &mut dyn Read, remote: &str) -> Result<(), String> {
    device.push(data, remote).map_err(|e| format!("push: {e}"))
}

pub fn push_item(device: &mut ADBServerDevice, local: &str, remote: &str) -> Result<(), String> {
    let path = std::path::Path::new(local);
    if !path.exists() {
        return Err(format!("local path '{local}' does not exist"));
    }

    if path.is_dir() {
        let _ = create_dir(device, remote);
        let entries = std::fs::read_dir(path).map_err(|e| format!("read_dir {local}: {e}"))?;
        for entry in entries.flatten() {
            let child_local = entry.path();
            let file_name = child_local.file_name().unwrap_or_default().to_string_lossy();
            let child_remote = format!("{}/{}", remote.trim_end_matches('/'), file_name);

            if child_local.is_dir() {
                push_item(device, &child_local.to_string_lossy(), &child_remote)?;
            } else {
                let mut file = std::fs::File::open(&child_local).map_err(|e| format!("open {}: {e}", child_local.display()))?;
                let _ = push_file(device, &mut file, &child_remote);
            }
        }
        Ok(())
    } else {
        let mut file = std::fs::File::open(path).map_err(|e| format!("open {local}: {e}"))?;
        push_file(device, &mut file, remote)
    }
}

pub fn delete_file(device: &mut ADBServerDevice, path: &str) -> Result<(), String> {
    let mut out = Vec::new();
    device.shell_command(&format!("rm -rf {path}"), Some(&mut out), None)
        .map_err(|e| format!("rm: {e}"))?;
    Ok(())
}

pub fn create_dir(device: &mut ADBServerDevice, path: &str) -> Result<(), String> {
    let mut out = Vec::new();
    device.shell_command(&format!("mkdir -p {path}"), Some(&mut out), None)
        .map_err(|e| format!("mkdir: {e}"))?;
    Ok(())
}

pub fn rename(device: &mut ADBServerDevice, src: &str, dst: &str) -> Result<(), String> {
    let mut out = Vec::new();
    device.shell_command(&format!("mv {src} {dst}"), Some(&mut out), None)
        .map_err(|e| format!("mv: {e}"))?;
    Ok(())
}

fn convert_entry(item: ADBListItemType) -> FileEntry {
    let (adb_item, is_dir) = match item {
        ADBListItemType::Directory(i) => (i, true),
        other => (into_inner(other), false),
    };
    FileEntry {
        name: adb_item.name,
        is_dir,
        size: adb_item.size as u64,
        permissions: format!("{:o}", adb_item.permissions),
        modified: adb_item.time,
    }
}

fn into_inner(item: ADBListItemType) -> ADBListItem {
    match item {
        ADBListItemType::Fifo(i)
        | ADBListItemType::CharacterDevice(i)
        | ADBListItemType::Directory(i)
        | ADBListItemType::BlockDevice(i)
        | ADBListItemType::File(i)
        | ADBListItemType::Symlink(i)
        | ADBListItemType::Socket(i)
        | ADBListItemType::Other(i) => i,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_file_entry() {
        let item = ADBListItem {
            name: "test.txt".into(),
            time: 1234567890,
            permissions: 0o100644,
            size: 1024,
        };
        let entry = convert_entry(ADBListItemType::File(item));
        assert_eq!(entry.name, "test.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.modified, 1234567890);
    }

    #[test]
    fn test_convert_dir_entry() {
        let item = ADBListItem {
            name: "data".into(),
            time: 1234567890,
            permissions: 0o40755,
            size: 4096,
        };
        let entry = convert_entry(ADBListItemType::Directory(item));
        assert_eq!(entry.name, "data");
        assert!(entry.is_dir);
    }
}

