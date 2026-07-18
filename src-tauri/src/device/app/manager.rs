use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

pub fn list_packages(device: &mut ADBServerDevice, filter: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let cmd = match filter {
        Some(f) => format!("pm list packages {f}"),
        None => "pm list packages".to_string(),
    };
    if device.shell_command(&cmd, Some(&mut out), None).is_err() {
        return vec![];
    }
    let text = String::from_utf8_lossy(&out);
    parse_pm_output(&text)
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
    fn test_parse_pm_output_filter_third_party() {
        let output = "package:com.android.chrome\npackage:com.example.game\n";
        let pkgs = parse_pm_output(output);
        assert_eq!(pkgs.len(), 2);
    }

    #[test]
    fn test_parse_pm_output_trailing_newline() {
        let output = "package:com.example.app\n";
        let pkgs = parse_pm_output(output);
        assert_eq!(pkgs.len(), 1);
    }
}
