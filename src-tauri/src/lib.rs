mod battery;
mod dchu;
mod display;
mod event_log;
mod models;
mod state;
mod system_info;

use models::{Diagnostics, Snapshot, SystemInfo};
use state::AppState;
use tauri::Manager;

#[tauri::command]
fn get_snapshot(state: tauri::State<'_, AppState>) -> Snapshot {
    state.snapshot()
}

#[tauri::command]
fn get_system_info(state: tauri::State<'_, AppState>) -> SystemInfo {
    state.system_info()
}

#[tauri::command]
fn get_diagnostics(state: tauri::State<'_, AppState>) -> Diagnostics {
    state.diagnostics()
}

#[tauri::command]
fn set_power_mode(state: tauri::State<'_, AppState>, mode: u8) -> Result<Snapshot, String> {
    state.set_power_mode(mode)
}

#[tauri::command]
fn set_fan_mode(state: tauri::State<'_, AppState>, mode: u8) -> Result<Snapshot, String> {
    state.set_fan_mode(mode)
}

#[tauri::command]
fn set_fan_offset(state: tauri::State<'_, AppState>, offset: u8) -> Result<Snapshot, String> {
    state.set_fan_offset(offset)
}

#[tauri::command]
fn set_keyboard_settings(
    state: tauri::State<'_, AppState>,
    fn_lock: bool,
    windows_key_enabled: bool,
    win_fn_swap: bool,
) -> Result<Snapshot, String> {
    state.set_keyboard_settings(fn_lock, windows_key_enabled, win_fn_swap)
}

#[tauri::command]
fn set_charging_threshold(
    state: tauri::State<'_, AppState>,
    enabled: bool,
    start_percent: u8,
    stop_percent: u8,
) -> Result<Snapshot, String> {
    state.set_charging_threshold(enabled, start_percent, stop_percent)
}

#[tauri::command]
fn set_low_refresh_rate(
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<Snapshot, String> {
    state.set_low_refresh_rate(enabled)
}

#[tauri::command]
fn set_blue_light_factor(
    state: tauri::State<'_, AppState>,
    factor: f32,
) -> Result<Snapshot, String> {
    state.set_blue_light_factor(factor)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let resource_dir = app.path().resource_dir().ok();
            app.manage(AppState::new(resource_dir.as_deref()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_system_info,
            get_diagnostics,
            set_power_mode,
            set_fan_mode,
            set_fan_offset,
            set_keyboard_settings,
            set_charging_threshold,
            set_low_refresh_rate,
            set_blue_light_factor
        ])
        .run(tauri::generate_context!())
        .expect("运行 CC30 Nova 时出错");
}
