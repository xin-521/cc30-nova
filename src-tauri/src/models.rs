use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FanChannel {
    pub id: String,
    pub label: String,
    pub rpm: u32,
    pub temperature_c: u8,
    pub duty_percent: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FanCurvePoint {
    pub temperature_c: u8,
    pub duty_percent: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FanCurve {
    pub id: String,
    pub label: String,
    pub points: Vec<FanCurvePoint>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FanSnapshot {
    pub mode: u8,
    pub mode_label: String,
    pub offset: u8,
    pub fan_count: u8,
    pub custom_curve_supported: bool,
    pub channels: Vec<FanChannel>,
    pub curves: Vec<FanCurve>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub feature_raw: u32,
    pub feature_package_version: String,
    pub feature_flags: Vec<FeatureFlag>,
    pub power_modes: bool,
    pub power_mode_options: Vec<u8>,
    pub fan_control: bool,
    pub keyboard_rgb: bool,
    pub rgb_zones: u8,
    pub per_key_keyboard: bool,
    pub keyboard_controller_type: u8,
    pub cpu_overclock: bool,
    pub gpu_overclock: bool,
    pub flexikey: bool,
    pub fan_count: u8,
    pub custom_fan_curves: bool,
    pub turbo_fan: bool,
    pub slow_fan: bool,
    pub max_q_fan: bool,
    pub silent_fan: bool,
    pub anti_dust: bool,
    pub dtt: bool,
    pub fan_offset_supported: bool,
    pub low_refresh_rate: bool,
    pub blue_light_filter: bool,
    pub gpu_switch: bool,
    pub keyboard_settings: bool,
    pub flexi_charger: bool,
    pub battery_utility: bool,
    pub energy_save: bool,
    pub flexi_access: bool,
    pub page_up_down_key: bool,
    pub control_center_function: bool,
    pub amd_manual_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlag {
    pub id: String,
    pub label: String,
    pub supported: bool,
    pub access: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub connected: bool,
    pub source: String,
    pub power_mode: u8,
    pub power_mode_label: String,
    pub fan: FanSnapshot,
    pub capabilities: Capabilities,
    pub device: DeviceState,
    pub captured_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ec_version: Option<String>,
    pub gpu_mode: GpuModeState,
    pub keyboard: KeyboardSettings,
    pub charging: ChargingSettings,
    pub battery_runtime: BatteryRuntime,
    pub battery_firmware: BatteryFirmware,
    pub anti_dust: AntiDustState,
    pub system_policy: SystemPolicyState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuModeOption {
    pub value: u8,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuModeState {
    pub supported: bool,
    pub current: u8,
    pub current_label: String,
    pub options: Vec<GpuModeOption>,
    pub restart_required: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardSettings {
    pub supported: bool,
    pub controller_type: u8,
    pub fn_lock: bool,
    pub fn_lock_supported: bool,
    pub windows_key_enabled: bool,
    pub windows_key_supported: bool,
    pub win_fn_swap: bool,
    pub win_fn_supported: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSettings {
    pub supported: bool,
    pub enabled: bool,
    pub start_percent: u8,
    pub stop_percent: u8,
    pub start_options: Vec<u8>,
    pub stop_options: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryRuntime {
    pub present: bool,
    pub power_online: bool,
    pub charging: bool,
    pub discharging: bool,
    pub charge_percent: u8,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryFirmware {
    pub supported: bool,
    pub version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufactured_on: Option<String>,
    pub cycle_count: u16,
    pub full_charge_capacity_mwh: u16,
    pub design_capacity_mwh: u16,
    pub health_percent: u8,
    pub battery_status: u16,
    pub pf_status: u32,
    pub operation_status: u32,
    pub stop_charging_threshold: u8,
    pub refresh_charge_phase: bool,
    pub refresh_discharge_phase: bool,
    pub permanent_failure: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiDustState {
    pub supported: bool,
    pub running: bool,
    pub scheduler_enabled: bool,
    pub day_mask: u8,
    pub hour: u8,
    pub resume_after_boot: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPolicyState {
    pub low_refresh_rate_supported: bool,
    pub low_refresh_rate_enabled: bool,
    pub blue_light_supported: bool,
    pub blue_light_factor: f32,
    pub page_up_down_supported: bool,
    pub page_up_down_enabled: bool,
    pub turbo_fan_active: bool,
    pub slow_fan_active: bool,
    pub dtt_enabled: bool,
    pub power_mode_locked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryModule {
    pub locator: String,
    pub manufacturer: String,
    pub part_number: String,
    pub speed_mhz: u32,
    pub size_gb: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub serial_number: String,
    pub family: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub bios_date: String,
    pub board_manufacturer: String,
    pub board_product: String,
    pub board_version: String,
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub memory_total_gb: u64,
    pub memory_modules: Vec<MemoryModule>,
    pub os_name: String,
    pub os_build: String,
    pub hostname: String,
    pub gpu_names: Vec<String>,
    pub battery: BatteryInfo,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryInfo {
    pub present: bool,
    pub model: String,
    pub manufacturer: String,
    pub designed_capacity_mwh: u32,
    pub full_charged_capacity_mwh: u32,
    pub cycle_count: u32,
    pub health_percent: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticCheck {
    pub id: String,
    pub label: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dll_path: Option<String>,
    pub dll_present: bool,
    pub api_available: bool,
    pub out_log_available: bool,
    pub power_bios_log_available: bool,
    pub feature_raw: u32,
    pub wmi16: u32,
    pub wmi96: u32,
    pub wmi122: u32,
    pub native_apis: Vec<NativeApi>,
    pub checks: Vec<DiagnosticCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeApi {
    pub name: String,
    pub direction: String,
    pub purpose: String,
    pub loaded: bool,
}

pub fn power_mode_label(mode: u8) -> &'static str {
    match mode {
        0 => "静音",
        1 => "节能",
        2 => "性能",
        3 => "娱乐",
        _ => "未知",
    }
}

pub fn fan_mode_label(mode: u8) -> &'static str {
    match mode {
        0 => "自动",
        1 => "最大",
        5 => "Max-Q",
        6 => "自定义",
        3 => "静音",
        8 => "慢速",
        _ => "未知",
    }
}

pub fn mock_snapshot(notice: impl Into<String>) -> Snapshot {
    Snapshot {
        connected: false,
        source: "simulation".to_string(),
        power_mode: 2,
        power_mode_label: "性能".to_string(),
        fan: FanSnapshot {
            mode: 0,
            mode_label: "自动".to_string(),
            offset: 6,
            fan_count: 2,
            custom_curve_supported: true,
            channels: vec![
                FanChannel {
                    id: "cpu".to_string(),
                    label: "CPU 风扇".to_string(),
                    rpm: 2840,
                    temperature_c: 67,
                    duty_percent: 42,
                },
                FanChannel {
                    id: "gpu1".to_string(),
                    label: "GPU 风扇".to_string(),
                    rpm: 3120,
                    temperature_c: 61,
                    duty_percent: 48,
                },
            ],
            curves: vec![
                FanCurve {
                    id: "cpu".to_string(),
                    label: "CPU".to_string(),
                    points: vec![
                        FanCurvePoint {
                            temperature_c: 40,
                            duty_percent: 24,
                        },
                        FanCurvePoint {
                            temperature_c: 60,
                            duty_percent: 40,
                        },
                        FanCurvePoint {
                            temperature_c: 80,
                            duty_percent: 65,
                        },
                        FanCurvePoint {
                            temperature_c: 100,
                            duty_percent: 100,
                        },
                    ],
                },
                FanCurve {
                    id: "gpu1".to_string(),
                    label: "GPU".to_string(),
                    points: vec![
                        FanCurvePoint {
                            temperature_c: 40,
                            duty_percent: 24,
                        },
                        FanCurvePoint {
                            temperature_c: 60,
                            duty_percent: 40,
                        },
                        FanCurvePoint {
                            temperature_c: 80,
                            duty_percent: 64,
                        },
                        FanCurvePoint {
                            temperature_c: 100,
                            duty_percent: 100,
                        },
                    ],
                },
            ],
        },
        capabilities: Capabilities {
            feature_raw: 0x893,
            feature_package_version: "1.0".to_string(),
            feature_flags: mock_feature_flags(),
            power_modes: true,
            power_mode_options: vec![0, 2],
            fan_control: true,
            keyboard_rgb: true,
            rgb_zones: 0,
            per_key_keyboard: false,
            keyboard_controller_type: 6,
            cpu_overclock: false,
            gpu_overclock: false,
            flexikey: false,
            fan_count: 2,
            custom_fan_curves: true,
            turbo_fan: false,
            slow_fan: false,
            max_q_fan: false,
            silent_fan: false,
            anti_dust: false,
            dtt: false,
            fan_offset_supported: true,
            low_refresh_rate: true,
            blue_light_filter: true,
            gpu_switch: true,
            keyboard_settings: true,
            flexi_charger: true,
            battery_utility: false,
            energy_save: true,
            flexi_access: false,
            page_up_down_key: false,
            control_center_function: true,
            amd_manual_mode: false,
        },
        device: DeviceState {
            ec_version: Some("1.07.03".to_string()),
            gpu_mode: GpuModeState {
                supported: true,
                current: 2,
                current_label: "独显模式".to_string(),
                options: gpu_mode_options(0x06),
                restart_required: true,
            },
            keyboard: KeyboardSettings {
                supported: true,
                controller_type: 6,
                fn_lock: false,
                fn_lock_supported: true,
                windows_key_enabled: true,
                windows_key_supported: true,
                win_fn_swap: false,
                win_fn_supported: true,
            },
            charging: ChargingSettings {
                supported: true,
                enabled: true,
                start_percent: 95,
                stop_percent: 100,
                start_options: vec![40, 50, 60, 70, 80, 90, 95],
                stop_options: vec![50, 60, 70, 80, 90, 95, 100],
            },
            battery_runtime: BatteryRuntime {
                present: true,
                power_online: true,
                charging: false,
                discharging: false,
                charge_percent: 100,
            },
            battery_firmware: BatteryFirmware {
                supported: true,
                version: 1,
                manufactured_on: Some("2023-11-29".to_string()),
                cycle_count: 0,
                full_charge_capacity_mwh: 39870,
                design_capacity_mwh: 51528,
                health_percent: 77,
                battery_status: 0,
                pf_status: 0,
                operation_status: 0,
                stop_charging_threshold: 95,
                refresh_charge_phase: false,
                refresh_discharge_phase: false,
                permanent_failure: false,
            },
            anti_dust: AntiDustState::default(),
            system_policy: SystemPolicyState {
                low_refresh_rate_supported: true,
                low_refresh_rate_enabled: true,
                blue_light_supported: true,
                blue_light_factor: 1.0,
                page_up_down_supported: false,
                page_up_down_enabled: false,
                turbo_fan_active: false,
                slow_fan_active: false,
                dtt_enabled: false,
                power_mode_locked: false,
            },
        },
        captured_at: chrono::Local::now().to_rfc3339(),
        notice: Some(notice.into()),
    }
}

fn mock_feature_flags() -> Vec<FeatureFlag> {
    vec![
        feature_flag(
            "power",
            "电源模式",
            true,
            "writable",
            "固件支持静音与性能配置",
        ),
        feature_flag("fan", "风扇模式", true, "writable", "自动与最大模式"),
        feature_flag(
            "fan_offset",
            "风扇偏移",
            true,
            "writable",
            "0-100% 自动曲线偏移",
        ),
        feature_flag(
            "fan_curve",
            "风扇曲线",
            true,
            "readOnly",
            "读取固件温控映射",
        ),
        feature_flag(
            "gpu_switch",
            "GPU 模式",
            true,
            "readOnly",
            "显示当前模式与可用选项",
        ),
        feature_flag(
            "keyboard",
            "键盘设置",
            true,
            "readOnly",
            "FnLock、Win 键与 Win/Fn",
        ),
        feature_flag(
            "charging",
            "充电阈值",
            true,
            "readOnly",
            "FlexiCharger 当前配置",
        ),
        feature_flag(
            "rgb",
            "键盘 RGB",
            true,
            "readOnly",
            "键盘控制器类型 6；通用灯效能力",
        ),
        feature_flag(
            "battery",
            "电池健康",
            true,
            "readOnly",
            "Windows 电池静态数据",
        ),
        feature_flag(
            "turbo_fan",
            "Turbo Fan",
            false,
            "unsupported",
            "BIOS 未声明",
        ),
        feature_flag("slow_fan", "Slow Fan", false, "unsupported", "BIOS 未声明"),
        feature_flag("cpu_oc", "CPU 超频", false, "unsupported", "固件未声明"),
        feature_flag("gpu_oc", "GPU 超频", false, "unsupported", "固件未声明"),
    ]
}

fn feature_flag(id: &str, label: &str, supported: bool, access: &str, detail: &str) -> FeatureFlag {
    FeatureFlag {
        id: id.to_string(),
        label: label.to_string(),
        supported,
        access: access.to_string(),
        detail: detail.to_string(),
    }
}

pub fn gpu_mode_label(mode: u8) -> &'static str {
    match mode {
        1 => "集显模式",
        2 => "独显模式",
        3 => "混合模式",
        4 => "动态模式",
        _ => "未知",
    }
}

pub fn gpu_mode_options(bitmap: u8) -> Vec<GpuModeOption> {
    (1..=4)
        .filter(|mode| bitmap & (1 << (mode - 1)) != 0)
        .map(|mode| GpuModeOption {
            value: mode,
            label: gpu_mode_label(mode).to_string(),
        })
        .collect()
}
