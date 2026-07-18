pub fn adb_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    }
}

pub fn fastboot_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "fastboot.exe"
    } else {
        "fastboot"
    }
}

pub fn scrcpy_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "scrcpy.exe"
    } else {
        "scrcpy"
    }
}

pub fn platform_tools_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "platform-tools-latest-windows.zip"
    } else if cfg!(target_os = "macos") {
        "platform-tools-latest-darwin.zip"
    } else {
        "platform-tools-latest-linux.zip"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_binary() {
        let name = adb_binary_name();
        #[cfg(target_os = "windows")]
        assert_eq!(name, "adb.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(name, "adb");
    }

    #[test]
    fn test_fastboot_binary() {
        let name = fastboot_binary_name();
        #[cfg(target_os = "windows")]
        assert_eq!(name, "fastboot.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(name, "fastboot");
    }

    #[test]
    fn test_platform_tools_name() {
        let name = platform_tools_name();
        assert!(name.contains("platform-tools-latest-"));
    }
}
