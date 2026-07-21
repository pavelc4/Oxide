mod audit;
mod core;
mod device;
mod fastboot;
mod setup;
mod theme;

#[tauri::command]
fn get_devices() -> Vec<device::types::DeviceSummary> {
    device::core::list::list_adb()
        .into_iter()
        .map(|d| device::types::DeviceSummary {
            serial: d.identifier,
            mode: device::types::DeviceMode::Adb,
            state: if d.state.to_string() == "device" {
                device::types::DeviceState::Device
            } else {
                device::types::DeviceState::Unknown
            },
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
fn list_packages(serial: String, filter: Option<String>) -> Result<Vec<String>, String> {
    let mut d = device::state::connect_serial(&serial);
    Ok(device::app::manager::list_packages(&mut d, filter.as_deref()))
}

#[tauri::command]
fn get_package_info(serial: String, package: String) -> Result<device::app::manager::PackageInfo, String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::get_package_info(&mut d, &package)
}

#[tauri::command]
fn start_package_app(serial: String, package: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::start_package_app(&mut d, &package)
}

#[tauri::command]
fn force_stop_package(serial: String, package: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::force_stop_package(&mut d, &package)
}

#[tauri::command]
fn enable_package(serial: String, package: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::enable_package(&mut d, &package)
}

#[tauri::command]
fn disable_package(serial: String, package: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::app::manager::disable_package(&mut d, &package)
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
fn list_files(
    serial: String,
    path: String,
) -> Result<Vec<device::file::explorer::FileEntry>, String> {
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
fn create_dir(serial: String, path: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::file::explorer::create_dir(&mut d, &path)
}

#[tauri::command]
fn rename_file(serial: String, src: String, dst: String) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::file::explorer::rename(&mut d, &src, &dst)
}

#[tauri::command]
fn get_config() -> Result<core::config::AppConfig, String> {
    core::config::AppConfig::load().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(config: core::config::AppConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
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
fn wireless_tcpip(serial: String, port: u16) -> Result<(), String> {
    let mut d = device::state::connect_serial(&serial);
    device::net::wireless::enable_tcpip(&mut d, port)
}

#[tauri::command]
fn fastboot_list() -> Vec<fastboot::flasher::FastbootDevice> {
    vec![]
}

#[tauri::command]
fn fastboot_flash(_serial: String, _partition: String, _data: Vec<u8>) -> Result<(), String> {
    Err("async command not wired yet".into())
}

#[tauri::command]
fn fastboot_erase(_serial: String, _partition: String) -> Result<(), String> {
    Err("async command not wired yet".into())
}

#[tauri::command]
fn fastboot_reboot(_serial: String) -> Result<(), String> {
    Err("async command not wired yet".into())
}

#[tauri::command]
fn generate_theme(argb: u32) -> theme::generator::ThemeColors {
    theme::generator::from_color(argb)
}

#[tauri::command]
fn get_audit_log() -> Vec<audit::log::AuditEntry> {
    audit::log::AuditLog::all()
}

#[tauri::command]
fn clear_audit_log() {
    audit::log::AuditLog::clear();
}

#[tauri::command]
fn get_setup_status() -> setup::wizard::SetupStatus {
    setup::wizard::check_status()
}

#[tauri::command]
fn complete_setup() -> Result<(), String> {
    setup::wizard::complete_setup()
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
            get_package_info,
            start_package_app,
            force_stop_package,
            enable_package,
            disable_package,
            install_apk,
            uninstall_package,
            clear_package,
            list_files,
            pull_file,
            push_file,
            delete_file,
            create_dir,
            rename_file,
            get_config,
            save_config,
            get_perf,
            wireless_connect,
            wireless_disconnect,
            wireless_tcpip,
            fastboot_list,
            fastboot_flash,
            fastboot_erase,
            fastboot_reboot,
            generate_theme,
            get_audit_log,
            get_setup_status,
            complete_setup,
            clear_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
