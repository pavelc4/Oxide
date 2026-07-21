use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageInfo {
    pub package_name: String,
    pub version_name: Option<String>,
    pub version_code: Option<u64>,
    pub label: Option<String>,
    pub install_location: Option<String>,
    pub flags: Vec<String>,
    pub first_install_time: Option<String>,
    pub last_update_time: Option<String>,
    pub apk_path: Option<String>,
    pub data_dir: Option<String>,
    pub is_system_app: bool,
    pub is_enabled: bool,
}

pub fn list_packages(device: &mut ADBServerDevice, filter: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let cmd = match filter {
        Some("thirdparty") => "pm list packages -3".to_string(),
        Some("system") => "pm list packages -s".to_string(),
        Some("disabled") => "pm list packages -d".to_string(),
        Some("enabled") => "pm list packages -e".to_string(),
        Some(f) if !f.is_empty() && f != "all" => format!("pm list packages {f}"),
        _ => "pm list packages".to_string(),
    };
    if device.shell_command(&cmd, Some(&mut out), None).is_err() {
        return vec![];
    }
    let text = String::from_utf8_lossy(&out);
    parse_pm_output(&text)
}

pub fn get_package_info(device: &mut ADBServerDevice, package: &str) -> Result<PackageInfo, String> {
    let mut path_out = Vec::new();
    let _ = device.shell_command(&format!("pm path {package}"), Some(&mut path_out), None);
    let path_str = String::from_utf8_lossy(&path_out);

    let mut dumpsys_out = Vec::new();
    let _ = device.shell_command(&format!("dumpsys package {package}"), Some(&mut dumpsys_out), None);
    let dumpsys_str = String::from_utf8_lossy(&dumpsys_out);

    Ok(parse_dumpsys_package(package, &path_str, &dumpsys_str))
}

pub fn start_package_app(device: &mut ADBServerDevice, package: &str) -> Result<(), String> {
    let mut out = Vec::new();

    // 1. Try resolving main launcher component via cmd package
    let cmd_resolve = format!("cmd package resolve-activity --brief {package}");
    if device.shell_command(&cmd_resolve, Some(&mut out), None).is_ok() {
        let text = String::from_utf8_lossy(&out);
        let activity = text
            .lines()
            .last()
            .map(|l| l.trim())
            .unwrap_or("");

        if activity.contains('/') && !activity.contains("No activity found") {
            let mut am_out = Vec::new();
            let am_cmd = format!("am start -n {activity}");
            if device.shell_command(&am_cmd, Some(&mut am_out), None).is_ok() {
                let am_text = String::from_utf8_lossy(&am_out);
                if !am_text.contains("Error:") {
                    return Ok(());
                }
            }
        }
    }

    // 2. Fallback to monkey launcher trigger
    out.clear();
    let monkey_cmd = format!("monkey -p {package} -c android.intent.category.LAUNCHER 1");
    let _ = device.shell_command(&monkey_cmd, Some(&mut out), None);
    let monkey_text = String::from_utf8_lossy(&out);

    if monkey_text.contains("No activities found") {
        return Err(format!("No launchable main activity screen found for package '{package}'"));
    }

    Ok(())
}

pub fn force_stop_package(device: &mut ADBServerDevice, package: &str) -> Result<(), String> {
    let mut out = Vec::new();
    device.shell_command(&format!("am force-stop {package}"), Some(&mut out), None)
        .map_err(|e| format!("force_stop: {e}"))?;
    Ok(())
}

pub fn enable_package(device: &mut ADBServerDevice, package: &str) -> Result<(), String> {
    let mut out = Vec::new();
    device.shell_command(&format!("pm enable {package}"), Some(&mut out), None)
        .map_err(|e| format!("enable: {e}"))?;
    Ok(())
}

pub fn disable_package(device: &mut ADBServerDevice, package: &str) -> Result<(), String> {
    let mut out = Vec::new();
    device.shell_command(&format!("pm disable-user --user 0 {package}"), Some(&mut out), None)
        .map_err(|e| format!("disable: {e}"))?;
    Ok(())
}

pub fn install_apk(device: &mut ADBServerDevice, apk_path: &str, user: Option<&str>) -> Result<(), String> {
    device.install(apk_path, user).map_err(|e| format!("install: {e}"))
}

pub fn uninstall_package(device: &mut ADBServerDevice, package: &str, user: Option<&str>) -> Result<(), String> {
    device.uninstall(package, user).map_err(|e| format!("uninstall: {e}"))
}

pub fn clear_package(device: &mut ADBServerDevice, package: &str) -> Result<(), String> {
    let mut out = Vec::new();
    let cmd = format!("pm clear {package}");
    device.shell_command(&cmd, Some(&mut out), None)
        .map_err(|e| format!("clear: {e}"))?;
    let text = String::from_utf8_lossy(&out);
    if text.contains("Success") {
        Ok(())
    } else {
        Err(format!("clear: failed to clear {package}"))
    }
}

pub fn parse_pm_output(output: &str) -> Vec<String> {
    output.lines()
        .map(|l| l.trim())
        .filter(|l| l.starts_with("package:"))
        .map(|l| l.strip_prefix("package:").unwrap_or(l).to_string())
        .collect()
}

pub fn parse_dumpsys_package(package: &str, path_out: &str, dumpsys_out: &str) -> PackageInfo {
    let apk_path = path_out
        .lines()
        .find(|l| l.starts_with("package:"))
        .map(|l| l.strip_prefix("package:").unwrap_or(l).trim().to_string());

    let mut version_name = None;
    let mut version_code = None;
    let mut first_install_time = None;
    let mut last_update_time = None;
    let mut data_dir = None;
    let mut flags = Vec::new();
    let mut is_enabled = true;

    for line in dumpsys_out.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("versionName=") {
            version_name = Some(trimmed.strip_prefix("versionName=").unwrap_or("").to_string());
        } else if trimmed.starts_with("versionCode=") {
            if let Some(val) = trimmed.strip_prefix("versionCode=").map(|s| s.split_whitespace().next().unwrap_or(s)) {
                version_code = val.parse::<u64>().ok();
            }
        } else if trimmed.starts_with("firstInstallTime=") {
            first_install_time = Some(trimmed.strip_prefix("firstInstallTime=").unwrap_or("").to_string());
        } else if trimmed.starts_with("lastUpdateTime=") {
            last_update_time = Some(trimmed.strip_prefix("lastUpdateTime=").unwrap_or("").to_string());
        } else if trimmed.starts_with("dataDir=") {
            data_dir = Some(trimmed.strip_prefix("dataDir=").unwrap_or("").to_string());
        } else if trimmed.starts_with("pkgFlags=[") || trimmed.starts_with("flags=[") {
            if let Some(content) = trimmed.split('[').nth(1).and_then(|s| s.split(']').next()) {
                flags = content.split_whitespace().map(|s| s.trim_matches(',').to_string()).collect();
            }
        } else if trimmed.contains("DISABLED_USER") || trimmed.contains("DISABLED") {
            is_enabled = false;
        }
    }

    let is_system_app = flags.iter().any(|f| f.contains("SYSTEM"))
        || apk_path.as_deref().unwrap_or("").starts_with("/system")
        || apk_path.as_deref().unwrap_or("").starts_with("/product")
        || apk_path.as_deref().unwrap_or("").starts_with("/apex");

    let name_part = package.split('.').last().unwrap_or(package);
    let label = Some(format!("{}{}", &name_part[..1].to_uppercase(), &name_part[1..]));

    PackageInfo {
        package_name: package.to_string(),
        version_name,
        version_code,
        label,
        install_location: Some("Internal".into()),
        flags,
        first_install_time,
        last_update_time,
        apk_path,
        data_dir,
        is_system_app,
        is_enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pm_output() {
        let output = "package:com.example.app\npackage:com.android.chrome\npackage:com.google.gms\n";
        let pkgs = parse_pm_output(output);
        assert_eq!(pkgs.len(), 3);
        assert_eq!(pkgs[0], "com.example.app");
        assert_eq!(pkgs[1], "com.android.chrome");
    }

    #[test]
    fn test_parse_pm_output_empty() {
        let pkgs = parse_pm_output("");
        assert!(pkgs.is_empty());
    }

    #[test]
    fn test_parse_pm_output_no_prefix() {
        let pkgs = parse_pm_output("some random output\npackage:valid\n");
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0], "valid");
    }

    #[test]
    fn test_parse_dumpsys_package() {
        let dumpsys = "
Packages:
  Package [com.whatsapp] (123456):
    versionCode=241578 minSdk=21 targetSdk=34
    versionName=2.24.15.78
    dataDir=/data/user/0/com.whatsapp
    pkgFlags=[ HAS_CODE ALLOW_CLEAR_USER_DATA ]
    firstInstallTime=2024-01-10 12:00:00
    lastUpdateTime=2024-05-15 08:30:00
";
        let path = "package:/data/app/~~hash/com.whatsapp-1/base.apk\n";
        let info = parse_dumpsys_package("com.whatsapp", path, dumpsys);

        assert_eq!(info.package_name, "com.whatsapp");
        assert_eq!(info.version_name.unwrap(), "2.24.15.78");
        assert_eq!(info.version_code.unwrap(), 241578);
        assert_eq!(info.apk_path.unwrap(), "/data/app/~~hash/com.whatsapp-1/base.apk");
        assert_eq!(info.data_dir.unwrap(), "/data/user/0/com.whatsapp");
        assert!(!info.is_system_app);
        assert!(info.is_enabled);
    }
}
