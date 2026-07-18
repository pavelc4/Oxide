mod core;
mod device;
mod theme;

#[tauri::command]
fn get_devices() -> Vec<device::types::DeviceSummary> {
    device::core::list::list_adb()
        .into_iter()
        .map(|d| device::types::DeviceSummary {
            serial: d.identifier,
            mode: device::types::DeviceMode::Adb,
            state: if d.state.to_string() == "device" { device::types::DeviceState::Device }
                   else { device::types::DeviceState::Unknown },
            model: None,
            sdk: None,
            battery: None,
        })
        .collect()
}

#[tauri::command]
fn get_device_info(serial: String) -> Result<device::types::DeviceInfo, String> {
    let mut d = device::state::connect_serial(&serial);
    Ok(device::core::info::get_info(&mut d))
}

#[tauri::command]
fn device_shell(serial: String, command: String) -> Result<String, String> {
    let mut d = device::state::connect_serial(&serial);
    device::shell::logcat::shell_cmd(&mut d, &command)
}

#[tauri::command]
fn list_packages(serial: String) -> Result<Vec<String>, String> {
    let mut d = device::state::connect_serial(&serial);
    Ok(device::app::manager::list_packages(&mut d, None))
}

#[tauri::command]
fn install_apk(serial: String, path: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::install_apk(&mut d, &path, None)
}

#[tauri::command]
fn uninstall_package(serial: String, package: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::uninstall_package(&mut d, &package, None)
}

#[tauri::command]
fn clear_package(serial: String, package: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::clear_package(&mut d, &package)
}

#[tauri::command]
fn list_files(serial: String, path: String) -> Result<Vec<device::file::explorer::FileEntry>, String> {
    let mut d = device::state::connect_serial(&serial);
    Ok(device::file::explorer::list_files(&mut d, &path))
}

#[tauri::command]
fn pull_file(serial: String, remote: String, local: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    let mut file = std::fs::File::create(&local).map_err(|e| format!("create {local}: {e}"))?;
    device::file::explorer::pull_file(&mut d, &remote, &mut file)
}

#[tauri::command]
fn push_file(serial: String, local: String, remote: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    let mut file = std::fs::File::open(&local).map_err(|e| format!("open {local}: {e}"))?;
    device::file::explorer::push_file(&mut d, &mut file, &remote)
}

#[tauri::command]
fn delete_file(serial: String, path: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::file::explorer::delete_file(&mut d, &path)
}

#[tauri::command]
fn get_perf(serial: String) -> device::perf::monitor::PerfSnapshot {
    let mut d = device::state::connect_serial(&serial);
    device::perf::monitor::get_snapshot(&mut d)
}

#[tauri::command]
fn wireless_connect(host: String) -> Result<(), String> {
    device::net::wireless::connect_device(&host)
}

#[tauri::command]
fn wireless_disconnect(host: String) -> Result<(), String> {
    device::net::wireless::disconnect_device(&host)
}

#[tauri::command]
fn generate_theme(argb: u32) -> theme::generator::ThemeColors {
    theme::generator::from_color(argb)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_devices,
            get_device_info,
            device_shell,
            list_packages,
            install_apk,
            uninstall_package,
            clear_package,
            list_files,
            pull_file,
            push_file,
            delete_file,
            get_perf,
            wireless_connect,
            wireless_disconnect,
            generate_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
