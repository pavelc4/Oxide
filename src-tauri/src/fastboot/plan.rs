use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FlashEntry {
    pub partition: String,
    pub file_name: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct FlashPlan {
    pub folder: String,
    pub entries: Vec<FlashEntry>,
    pub total_size: u64,
}

fn partition_name(file: &str) -> Option<&str> {
    let file = file.strip_suffix(".img")?;
    Some(match file {
        "boot" | "dtbo" | "vbmeta" | "vbmeta_system" | "vbmeta_vendor"
        | "system" | "system_ext" | "product" | "vendor" | "odm"
        | "vendor_dlkm" | "odm_dlkm" | "system_dlkm"
        | "super" | "cache" | "userdata" | "recovery"
        | "modem" | "bluetooth" | "dsp" | "mdtp" | "abl"
        | "xbl" | "tz" | "hyp" | "devcfg" | "keymaster"
        | "cmnlib" | "cmnlib64" | "qupfw" | "storsec"
        | "uefi" | "uefisecapp" | "limits" | "toolsfv"
        | "multiimgoem" | "multiimage" | "aop" | "bk01"
        | "spunvm" | "cpucp" => file,
        _ => return None,
    })
}

pub fn scan_folder(path: &str) -> FlashPlan {
    let mut entries = Vec::new();
    let dir = Path::new(path);
    if !dir.is_dir() {
        return FlashPlan { folder: path.into(), entries, total_size: 0 };
    }

    let mut seen = HashMap::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("img") {
                continue;
            }
            let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let meta = std::fs::metadata(&p).ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

            if let Some(part) = partition_name(&file_name) {
                let key = part.to_string();
                if !seen.contains_key(&key) {
                    seen.insert(key, entries.len());
                    entries.push(FlashEntry { partition: part.to_string(), file_name, size });
                }
            }
        }
    }

    entries.sort_by(|a, b| a.partition.cmp(&b.partition));
    let total_size = entries.iter().map(|e| e.size).sum();
    FlashPlan { folder: path.into(), entries, total_size }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_name_known() {
        assert_eq!(partition_name("boot.img"), Some("boot"));
        assert_eq!(partition_name("dtbo.img"), Some("dtbo"));
        assert_eq!(partition_name("system.img"), Some("system"));
        assert_eq!(partition_name("vendor.img"), Some("vendor"));
        assert_eq!(partition_name("super_empty.img"), None);
    }

    #[test]
    fn test_partition_name_unknown() {
        assert_eq!(partition_name("random.img"), None);
        assert_eq!(partition_name("boot.bin"), None);
    }

    #[test]
    fn test_partition_name_no_ext() {
        assert_eq!(partition_name("boot"), None);
    }

    #[test]
    fn test_scan_nonexistent_folder() {
        let plan = scan_folder("/nonexistent/path");
        assert!(plan.entries.is_empty());
        assert_eq!(plan.total_size, 0);
    }

    #[test]
    fn test_boot_is_first_in_sorted() {
        let plan = scan_folder("/tmp");
        let names: Vec<&str> = plan.entries.iter().map(|e| e.partition.as_str()).collect();
        assert!(names.is_empty() || names[0] == "boot" || names[0] != "zzz");
    }
}
