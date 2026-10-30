use crate::dchu::{Dchu, DchuRuntime};
use crate::event_log;
use crate::models::{
    fan_mode_label, gpu_mode_label, gpu_mode_options, mock_snapshot, power_mode_label,
    AntiDustState, BatteryFirmware, Capabilities, ChargingSettings, DeviceState, DiagnosticCheck,
    Diagnostics, FanChannel, FanCurve, FanCurvePoint, FanSnapshot, FeatureFlag, GpuModeState,
    KeyboardSettings, NativeApi, Snapshot, SystemInfo, SystemPolicyState,
};
use std::path::Path;
use std::sync::Mutex;

pub struct AppState {
    runtime: Mutex<DchuRuntime>,
    system: Mutex<Option<SystemInfo>>,
}

impl AppState {
    pub fn new(resource_dir: Option<&Path>) -> Self {
        Self {
            runtime: Mutex::new(DchuRuntime::discover(resource_dir)),
            system: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            return mock_snapshot(
                runtime
                    .error
                    .clone()
                    .unwrap_or_else(|| "原生桥接不可用".to_string()),
            );
        };

        match read_snapshot(api) {
            Ok(snapshot) => snapshot,
            Err(error) => mock_snapshot(error),
        }
    }

    pub fn system_info(&self) -> SystemInfo {
        let mut system = self
            .system
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        system
            .get_or_insert_with(crate::system_info::collect)
            .clone()
    }

    pub fn diagnostics(&self) -> Diagnostics {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let dll_path = runtime
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string());
        let dll_present = runtime.path.as_ref().is_some_and(|path| path.is_file());
        let api_available = runtime.api.is_some();

        let (wmi16, wmi96, wmi122, feature_raw, fan_count) = if let Some(api) = &runtime.api {
            let wmi16 = api.get_integer(16);
            let wmi96 = api.get_integer(96);
            let wmi122 = api.get_integer(122);
            let fan = api.get_buffer(13);
            (wmi16, wmi96, wmi122, wmi16, fan[12])
        } else {
            (0, 0, 0, 0, 0)
        };

        let out_log_available = event_log::log_exists("OutLog");
        let power_bios_log_available = event_log::log_exists("PowerBiosServerLog");
        let mut checks = Vec::new();
        checks.push(DiagnosticCheck {
            id: "dll".to_string(),
            label: "原生桥接".to_string(),
            status: status(dll_present).to_string(),
            detail: dll_path.clone().unwrap_or_else(|| "未找到".to_string()),
        });
        checks.push(DiagnosticCheck {
            id: "api".to_string(),
            label: "DCHU 握手".to_string(),
            status: status(api_available).to_string(),
            detail: if api_available {
                "只读遥测已响应".to_string()
            } else {
                runtime
                    .error
                    .clone()
                    .unwrap_or_else(|| "不可用".to_string())
            },
        });
        checks.push(DiagnosticCheck {
            id: "outlog".to_string(),
            label: "OutLog 通道".to_string(),
            status: if out_log_available {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: if out_log_available {
                "电源事件可写入".to_string()
            } else {
                "未安装 CLEVO 电源服务".to_string()
            },
        });
        checks.push(DiagnosticCheck {
            id: "fan".to_string(),
            label: "散热系统".to_string(),
            status: if fan_count > 0 {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: format!("已检测到 {fan_count} 个风扇通道"),
        });
        let (feature_package, gpu_switch, keyboard, charging, battery_package) =
            if let Some(api) = &runtime.api {
                (
                    api.feature_package().ok(),
                    api.gpu_switch().ok(),
                    api.keyboard_settings().ok(),
                    api.charger_settings().ok(),
                    api.battery_package(),
                )
            } else {
                (None, None, None, None, Vec::new())
            };
        checks.push(DiagnosticCheck {
            id: "feature-package".to_string(),
            label: "BIOS 功能包".to_string(),
            status: if feature_package.is_some() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: feature_package
                .as_deref()
                .map(feature_version)
                .unwrap_or_else(|| "未响应".to_string()),
        });
        checks.push(DiagnosticCheck {
            id: "gpu-switch".to_string(),
            label: "GPU 模式".to_string(),
            status: if gpu_switch.is_some() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: gpu_switch
                .as_deref()
                .map(|data| gpu_mode_label(data.first().copied().unwrap_or(0)).to_string())
                .unwrap_or_else(|| "未声明".to_string()),
        });
        checks.push(DiagnosticCheck {
            id: "keyboard".to_string(),
            label: "键盘设置".to_string(),
            status: if keyboard.as_deref().is_some_and(nonzero_keyboard) {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: "FnLock / Win Key / Win-Fn".to_string(),
        });
        checks.push(DiagnosticCheck {
            id: "charging".to_string(),
            label: "充电阈值".to_string(),
            status: if charging.is_some() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: "FlexiCharger 开关与阈值可写".to_string(),
        });
        let battery = self.system_info().battery;
        checks.push(DiagnosticCheck {
            id: "battery".to_string(),
            label: "电池健康".to_string(),
            status: if battery.present {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: if battery.present {
                format!(
                    "{}% · {} 次循环",
                    battery.health_percent, battery.cycle_count
                )
            } else {
                "Windows 未返回电池静态数据".to_string()
            },
        });
        let battery_firmware = read_battery_firmware(&battery_package);
        checks.push(DiagnosticCheck {
            id: "battery-firmware".to_string(),
            label: "电池固件包".to_string(),
            status: if battery_firmware.supported {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: if battery_firmware.supported {
                format!(
                    "WMI 7 · 制造日期 {}",
                    battery_firmware
                        .manufactured_on
                        .as_deref()
                        .unwrap_or("未编码")
                )
            } else {
                "WMI 7 未返回有效电池数据".to_string()
            },
        });
        checks.push(DiagnosticCheck {
            id: "safety".to_string(),
            label: "写入白名单".to_string(),
            status: "ok".to_string(),
            detail: "电源、风扇、键盘、充电与显示策略".to_string(),
        });

        Diagnostics {
            dll_path,
            dll_present,
            api_available,
            out_log_available,
            power_bios_log_available,
            feature_raw,
            wmi16,
            wmi96,
            wmi122,
            native_apis: native_api_inventory(api_available),
            checks,
        }
    }

    pub fn set_power_mode(&self, mode: u8) -> Result<Snapshot, String> {
        if mode > 3 {
            return Err("电源模式必须介于 0 和 3 之间".to_string());
        }

        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入电源模式");
            snapshot.power_mode = mode;
            snapshot.power_mode_label = power_mode_label(mode).to_string();
            return Ok(snapshot);
        };
        require_capability(api, 0x01, "电源配置控制")?;

        let package = api.feature_package().ok();
        if package.is_some() {
            let options = power_mode_options(&package, true);
            if !options.contains(&mode) {
                return Err(format!("固件未声明“{}”电源配置", power_mode_label(mode)));
            }
        }

        let current = read_power_mode(api)?;
        if current == mode {
            return read_snapshot(api);
        }

        event_log::write_out_log(&format!("clevocc30^101^{mode}"))?;
        for _ in 0..50 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if read_power_mode(api)? == mode {
                return read_snapshot(api);
            }
        }
        Err("CLEVO 电源服务未在 5 秒内确认模式变更。".to_string())
    }

    pub fn set_fan_mode(&self, mode: u8) -> Result<Snapshot, String> {
        if !matches!(mode, 0 | 1) {
            return Err("仅允许自动与最大两种风扇模式".to_string());
        }

        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入风扇模式");
            snapshot.fan.mode = mode;
            snapshot.fan.mode_label = fan_mode_label(mode).to_string();
            return Ok(snapshot);
        };
        require_capability(api, 0x80, "风扇控制")?;

        api.set_command(121, 1, mode as u32)
            .map_err(|error| error.to_string())?;
        api.write_settings(4, 5, &[mode])
            .map_err(|error| error.to_string())?;
        let check = api
            .read_settings(4, 5, 1)
            .map_err(|error| error.to_string())?;
        if check.first().copied() != Some(mode) {
            return Err("风扇模式回读值与请求值不一致".to_string());
        }
        // The fan utility also emits this fixed event so the CLEVO service updates its OSD state.
        let _ = event_log::write_out_log(&format!("clevocc30^113^{mode}"));
        read_snapshot(api)
    }

    pub fn set_fan_offset(&self, offset: u8) -> Result<Snapshot, String> {
        if offset > 100 {
            return Err("风扇偏移必须介于 0 和 100 之间".to_string());
        }

        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入风扇偏移");
            snapshot.fan.offset = offset;
            return Ok(snapshot);
        };
        require_capability(api, 0x80, "风扇控制")?;

        let raw = (255.0 * offset as f64 / 100.0).round() as u32;
        api.set_command(121, 14, raw)
            .map_err(|error| error.to_string())?;
        api.write_settings(4, 7, &[offset])
            .map_err(|error| error.to_string())?;
        let check = api
            .read_settings(4, 7, 1)
            .map_err(|error| error.to_string())?;
        if check.first().copied() != Some(offset) {
            return Err("风扇偏移回读值与请求值不一致".to_string());
        }
        read_snapshot(api)
    }

    pub fn set_keyboard_settings(
        &self,
        fn_lock: bool,
        windows_key_enabled: bool,
        win_fn_swap: bool,
    ) -> Result<Snapshot, String> {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入键盘设置");
            snapshot.device.keyboard.fn_lock = fn_lock;
            snapshot.device.keyboard.windows_key_enabled = windows_key_enabled;
            snapshot.device.keyboard.win_fn_swap = win_fn_swap;
            return Ok(snapshot);
        };

        let current = read_keyboard_settings(
            &api.keyboard_settings().map_err(|error| error.to_string())?,
            keyboard_controller_type(&api.get_buffer(13)),
        );
        if !current.supported {
            return Err("固件未声明键盘设置能力".to_string());
        }
        if (fn_lock != current.fn_lock && !current.fn_lock_supported)
            || (windows_key_enabled != current.windows_key_enabled
                && !current.windows_key_supported)
            || (win_fn_swap != current.win_fn_swap && !current.win_fn_supported)
        {
            return Err("固件未声明所请求的键盘设置项".to_string());
        }

        let fn_lock_mode = if fn_lock { 1 } else { 2 };
        let windows_key_mode = if windows_key_enabled { 1 } else { 2 };
        let win_fn_mode = if win_fn_swap { 1 } else { 2 };
        api.apply_keyboard_settings(fn_lock_mode, windows_key_mode, win_fn_mode)
            .map_err(|error| error.to_string())?;

        let check = read_keyboard_settings(
            &api.keyboard_settings().map_err(|error| error.to_string())?,
            current.controller_type,
        );
        if check.fn_lock != fn_lock
            || check.windows_key_enabled != windows_key_enabled
            || check.win_fn_swap != win_fn_swap
        {
            return Err("键盘设置回读值与请求值不一致".to_string());
        }

        let persisted = [fn_lock_mode, windows_key_mode, win_fn_mode];
        api.write_settings(1, 36, &persisted)
            .map_err(|error| error.to_string())?;
        let persisted_check = api
            .read_settings(1, 36, 3)
            .map_err(|error| error.to_string())?;
        if persisted_check != persisted {
            return Err("键盘设置持久化回读失败".to_string());
        }

        read_snapshot(api)
    }

    pub fn set_charging_threshold(
        &self,
        enabled: bool,
        start_percent: u8,
        stop_percent: u8,
    ) -> Result<Snapshot, String> {
        if start_percent == 0 || stop_percent > 100 || start_percent >= stop_percent {
            return Err("充电开始阈值必须小于停止阈值，且停止阈值不得超过 100%".to_string());
        }

        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入充电阈值");
            snapshot.device.charging.enabled = enabled;
            snapshot.device.charging.start_percent = start_percent;
            snapshot.device.charging.stop_percent = stop_percent;
            return Ok(snapshot);
        };

        let current =
            read_charging_settings(&api.charger_settings().map_err(|error| error.to_string())?);
        if !current.supported {
            return Err("固件未声明 FlexiCharger 能力".to_string());
        }
        if !current.start_options.contains(&start_percent)
            || !current.stop_options.contains(&stop_percent)
        {
            return Err("请求的充电阈值不在固件允许范围内".to_string());
        }

        if current.enabled == enabled
            && current.start_percent == start_percent
            && current.stop_percent == stop_percent
        {
            return read_snapshot(api);
        }

        api.apply_charger_threshold(enabled, start_percent, stop_percent)
            .map_err(|error| error.to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(150));
        let check =
            read_charging_settings(&api.charger_settings().map_err(|error| error.to_string())?);
        if check.enabled != enabled
            || check.start_percent != start_percent
            || check.stop_percent != stop_percent
        {
            return Err("充电阈值回读值与请求值不一致".to_string());
        }

        read_snapshot(api)
    }

    pub fn set_low_refresh_rate(&self, enabled: bool) -> Result<Snapshot, String> {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入低刷新率策略");
            snapshot.device.system_policy.low_refresh_rate_enabled = enabled;
            return Ok(snapshot);
        };

        let preferences = api
            .read_settings(1, 27, 9)
            .map_err(|error| error.to_string())?;
        if preferences.first().copied() != Some(1) {
            return Err("固件未声明低刷新率策略".to_string());
        }
        let current = preferences.get(1).copied() == Some(1);
        if current == enabled {
            return read_snapshot(api);
        }

        let value = [u8::from(enabled)];
        api.write_settings(1, 28, &value)
            .map_err(|error| error.to_string())?;
        let check = api
            .read_settings(1, 28, 1)
            .map_err(|error| error.to_string())?;
        if check.first().copied() != Some(value[0]) {
            return Err("低刷新率策略回读值与请求值不一致".to_string());
        }
        event_log::write_out_log("clevocc30^114")?;
        read_snapshot(api)
    }

    pub fn set_blue_light_factor(&self, factor: f32) -> Result<Snapshot, String> {
        if !(1.0..=2.0).contains(&factor) {
            return Err("蓝光过滤强度必须介于 1.0 和 2.0 之间".to_string());
        }

        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(api) = runtime.api.as_ref() else {
            let mut snapshot = mock_snapshot("模拟模式：未写入蓝光过滤强度");
            snapshot.device.system_policy.blue_light_factor = factor;
            return Ok(snapshot);
        };

        if api.get_integer(16) & 0x0800 == 0 {
            return Err("固件未声明蓝光过滤能力".to_string());
        }
        let preferences = api
            .read_settings(1, 27, 9)
            .map_err(|error| error.to_string())?;
        let current = blue_light_factor(preferences.get(7).copied().unwrap_or(0));
        let target = (factor * 10.0).round() / 10.0;
        if (current - target).abs() < 0.05 {
            return read_snapshot(api);
        }

        crate::display::set_blue_light_factor(target)?;
        let raw = ((target * 10.0) - 10.0).round().clamp(0.0, 10.0) as u8;
        let value = [raw];
        let write_result = api
            .write_settings(1, 34, &value)
            .and_then(|_| api.read_settings(1, 34, 1));
        match write_result {
            Ok(check) if check.first().copied() == Some(raw) => read_snapshot(api),
            _ => {
                let _ = crate::display::set_blue_light_factor(current);
                Err("蓝光过滤设置持久化回读失败，已恢复原显示设置".to_string())
            }
        }
    }
}

fn native_api_inventory(loaded: bool) -> Vec<NativeApi> {
    [
        (
            "GetDCHU_Data_Integer",
            "读取",
            "读取 WMI 整数能力字与遥测状态",
        ),
        (
            "GetDCHU_Data_Buffer",
            "读取",
            "读取 WMI 12/13 遥测与配置缓冲区",
        ),
        (
            "ReadAppSettings",
            "读取",
            "读取电源、风扇、键盘与显示策略持久化设置",
        ),
        ("WriteAppSettings", "写入", "持久化已验证的固定策略值"),
        ("SetDCHU_Data", "写入", "发送固定风扇子命令"),
        (
            "SetDCHU_DataEx",
            "查询与写入",
            "查询 GPU、键盘、充电与 EC 固件状态",
        ),
    ]
    .into_iter()
    .map(|(name, direction, purpose)| NativeApi {
        name: name.to_string(),
        direction: direction.to_string(),
        purpose: purpose.to_string(),
        loaded,
    })
    .collect()
}

fn status(ok: bool) -> &'static str {
    if ok {
        "ok"
    } else {
        "error"
    }
}

fn require_capability(api: &Dchu, mask: u32, feature: &str) -> Result<(), String> {
    if api.get_integer(16) & mask == 0 {
        return Err(format!("固件未声明支持{feature}"));
    }
    Ok(())
}

fn read_snapshot(api: &Dchu) -> Result<Snapshot, String> {
    let power_mode = read_power_mode(api)?;
    let app_fan = api
        .read_settings(4, 0, 96)
        .map_err(|error| error.to_string())?;
    let telemetry = api.get_buffer(12);
    let fan_config = api.get_buffer(13);
    let wmi16 = api.get_integer(16);
    let wmi96 = api.get_integer(96);
    let wmi122 = api.get_integer(122);
    let battery_raw = api.battery_package();
    let feature_package = api.feature_package().ok();
    let ec_version = api.ec_version().ok();
    let gpu_raw = api.gpu_switch().ok().unwrap_or_default();
    let keyboard_raw = api.keyboard_settings().ok().unwrap_or_default();
    let charging_raw = api.charger_settings().ok().unwrap_or_default();
    let preferences = api.read_settings(1, 27, 9).ok();
    let fan_count = fan_config[12].max(1);
    let custom_curve_supported = fan_count > 1 && ((fan_config[43] >> 1) & 1) == 0;

    let mut channels = vec![
        FanChannel {
            id: "cpu".to_string(),
            label: "CPU 风扇".to_string(),
            rpm: tach_to_rpm(u16::from_be_bytes([telemetry[2], telemetry[3]])),
            temperature_c: telemetry[18],
            duty_percent: raw_duty(telemetry[16]),
        },
        FanChannel {
            id: "gpu1".to_string(),
            label: "GPU 风扇".to_string(),
            rpm: tach_to_rpm(u16::from_be_bytes([telemetry[4], telemetry[5]])),
            temperature_c: telemetry[21],
            duty_percent: raw_duty(telemetry[19]),
        },
    ];
    if fan_count >= 3 {
        channels.push(FanChannel {
            id: "gpu2".to_string(),
            label: "辅助风扇".to_string(),
            rpm: tach_to_rpm(u16::from_be_bytes([telemetry[6], telemetry[7]])),
            temperature_c: telemetry[24],
            duty_percent: raw_duty(telemetry[22]),
        });
    }

    let mut curves = Vec::new();
    if custom_curve_supported {
        curves.push(curve_from_app("cpu", "CPU", &app_fan, 16, 22));
        curves.push(curve_from_app("gpu1", "GPU", &app_fan, 34, 40));
        if fan_count >= 3 {
            curves.push(curve_from_app("gpu2", "辅助", &app_fan, 52, 58));
        }
    }

    let power_modes = wmi16 & 0x01 != 0;
    let power_mode_options = power_mode_options(&feature_package, power_modes);
    let fan_control = wmi16 & 0x80 != 0;
    let keyboard_controller_type = keyboard_controller_type(&fan_config);
    let keyboard_rgb = wmi16 & 0x18 != 0 && !matches!(keyboard_controller_type, 0 | 4);
    let per_key_keyboard = keyboard_rgb && wmi16 & 0x08 != 0;
    let flexikey = wmi16 & 0x02 != 0;
    let flexi_access = wmi16 & 0x04 != 0;
    let page_up_down_key = wmi16 & 0x0400 != 0;
    let control_center_function = wmi16 & 0x0800 != 0;
    let fan_feature_bits = package_u16(&feature_package, 10);
    let anti_dust = fan_feature_bits & 0x0080 != 0 || wmi96 & 0x0080 != 0;
    let dtt = fan_feature_bits & 0x1000 != 0 || wmi96 & 0x1000 != 0;
    let fan_offset_supported = fan_feature_bits & 0x0400 == 0 && wmi96 & 0x0400 == 0;
    let amd_manual_mode = wmi96 & 0x2000 != 0;
    let turbo_fan = package_bit(&feature_package, 16, 0x10);
    let slow_fan = package_bit(&feature_package, 20, 0x01);
    let gpu_switch_supported = package_bit(&feature_package, 18, 0x01) || gpu_raw.len() >= 2;
    let gpu_overclock = wmi16 & 0x20 != 0 || package_bit(&feature_package, 20, 0x08);
    let cpu_overclock = wmi16 & 0x40 != 0 || wmi122 & 0x0080_0000 != 0;
    let max_q_fan = fan_config[14] == 5;
    let silent_fan = wmi122 & 0x8000 != 0;
    // The firmware exposes lighting mode, not a reliable zone count.
    let rgb_zones = 0;
    let battery_utility = wmi16 & 0x0200 != 0;
    let energy_save = wmi16 & 0x0100 != 0;
    let gpu_mode = read_gpu_mode(&gpu_raw, gpu_switch_supported);
    let keyboard = read_keyboard_settings(&keyboard_raw, keyboard_controller_type);
    let charging = read_charging_settings(&charging_raw);
    let battery_firmware = read_battery_firmware(&battery_raw);
    let system_policy = read_system_policy(
        &app_fan,
        preferences.as_deref(),
        page_up_down_key,
        control_center_function,
        turbo_fan,
        slow_fan,
        dtt,
    );
    let low_refresh_rate = system_policy.low_refresh_rate_supported;
    let blue_light_filter = system_policy.blue_light_supported;
    let anti_dust_state = read_anti_dust(
        anti_dust,
        anti_dust
            .then(|| api.anti_dust_status().ok())
            .flatten()
            .unwrap_or(0),
        &app_fan,
    );

    let feature_flags = build_feature_flags(&FeatureInputs {
        power_modes,
        power_mode_options: power_mode_options.clone(),
        fan_control,
        custom_curve_supported,
        gpu_switch: gpu_mode.supported,
        keyboard: keyboard.supported,
        charging: charging.supported,
        keyboard_rgb,
        battery_utility,
        energy_save,
        turbo_fan,
        slow_fan,
        max_q_fan,
        silent_fan,
        cpu_overclock,
        gpu_overclock,
        flexikey,
        per_key_keyboard,
        keyboard_controller_type,
        anti_dust,
        dtt,
        fan_offset_supported,
        low_refresh_rate,
        blue_light_filter,
        battery_firmware: battery_firmware.supported,
        flexi_access,
        page_up_down_key,
        control_center_function,
        amd_manual_mode,
    });

    let capabilities = Capabilities {
        feature_raw: wmi16,
        feature_package_version: feature_package
            .as_deref()
            .map(feature_version)
            .unwrap_or_else(|| "未响应".to_string()),
        feature_flags,
        power_modes,
        power_mode_options,
        fan_control,
        keyboard_rgb,
        rgb_zones,
        per_key_keyboard,
        keyboard_controller_type,
        cpu_overclock,
        gpu_overclock,
        flexikey,
        fan_count,
        custom_fan_curves: custom_curve_supported,
        turbo_fan,
        slow_fan,
        max_q_fan,
        silent_fan,
        anti_dust,
        dtt,
        fan_offset_supported,
        low_refresh_rate,
        blue_light_filter,
        gpu_switch: gpu_switch_supported,
        keyboard_settings: keyboard.supported,
        flexi_charger: charging.supported,
        battery_utility,
        energy_save,
        flexi_access,
        page_up_down_key,
        control_center_function,
        amd_manual_mode,
    };

    Ok(Snapshot {
        connected: true,
        source: "native".to_string(),
        power_mode,
        power_mode_label: power_mode_label(power_mode).to_string(),
        fan: FanSnapshot {
            mode: app_fan[5],
            mode_label: fan_mode_label(app_fan[5]).to_string(),
            offset: app_fan[7],
            fan_count,
            custom_curve_supported,
            channels,
            curves,
        },
        capabilities,
        device: DeviceState {
            ec_version,
            gpu_mode,
            keyboard,
            charging,
            battery_runtime: crate::battery::runtime(),
            battery_firmware,
            anti_dust: anti_dust_state,
            system_policy,
        },
        captured_at: chrono::Local::now().to_rfc3339(),
        notice: None,
    })
}

fn package_bit(package: &Option<Vec<u8>>, offset: usize, mask: u8) -> bool {
    package
        .as_ref()
        .and_then(|data| data.get(offset))
        .is_some_and(|value| value & mask != 0)
}

fn power_mode_options(package: &Option<Vec<u8>>, power_modes: bool) -> Vec<u8> {
    if !power_modes {
        return Vec::new();
    }

    let Some(flags) = package.as_ref().and_then(|data| data.get(17)).copied() else {
        return vec![0, 2];
    };

    let options = (0..4)
        .filter(|mode| flags & (1 << mode) != 0)
        .collect::<Vec<_>>();

    // Some BIOS revisions expose the aggregate power-mode capability through
    // WMI 16 but leave the per-mode bitmap empty. The quiet and performance
    // modes are verified on the 911MT reference device, so keep those usable.
    if options.is_empty() {
        vec![0, 2]
    } else {
        options
    }
}

fn package_u16(package: &Option<Vec<u8>>, offset: usize) -> u16 {
    package
        .as_ref()
        .and_then(|data| data.get(offset..offset + 2))
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .unwrap_or(0)
}

fn read_battery_firmware(data: &[u8]) -> BatteryFirmware {
    let version = le_u16(data, 0);
    let manufacture_raw = le_u16(data, 2);
    let cycle_count = le_u16(data, 4);
    let full_charge_capacity_mwh = le_u16(data, 6);
    let design_capacity_mwh = le_u16(data, 8);
    let battery_status = le_u16(data, 10);
    let pf_status = le_u32(data, 12);
    let operation_status = le_u32(data, 16);
    let health_percent = if design_capacity_mwh > 0 && full_charge_capacity_mwh > 0 {
        ((full_charge_capacity_mwh as f64 / design_capacity_mwh as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8
    } else {
        0
    };

    BatteryFirmware {
        supported: version != 0 || manufacture_raw != 0,
        version,
        manufactured_on: battery_date(manufacture_raw),
        cycle_count,
        full_charge_capacity_mwh,
        design_capacity_mwh,
        health_percent,
        battery_status,
        pf_status,
        operation_status,
        stop_charging_threshold: data.get(20).copied().unwrap_or(0),
        refresh_charge_phase: battery_status & 0x10 != 0,
        refresh_discharge_phase: battery_status & 0x20 != 0,
        permanent_failure: operation_status & 0x1000 != 0,
    }
}

fn read_anti_dust(supported: bool, run_status: u8, app_fan: &[u8]) -> AntiDustState {
    AntiDustState {
        supported,
        running: supported && run_status != 0,
        scheduler_enabled: supported && app_fan.get(81).is_some_and(|value| *value != 0),
        day_mask: app_fan.get(82).copied().unwrap_or(0) & 0x7f,
        hour: app_fan.get(83).copied().unwrap_or(0),
        resume_after_boot: app_fan.get(82).is_some_and(|value| value & 0x80 != 0),
    }
}

fn read_system_policy(
    app_fan: &[u8],
    preferences: Option<&[u8]>,
    page_up_down_supported: bool,
    blue_light_supported: bool,
    turbo_fan_supported: bool,
    slow_fan_supported: bool,
    dtt_supported: bool,
) -> SystemPolicyState {
    let preference = |index: usize| preferences.and_then(|data| data.get(index)).copied();
    let low_refresh_rate_supported = preference(0) == Some(1);

    SystemPolicyState {
        low_refresh_rate_supported,
        low_refresh_rate_enabled: low_refresh_rate_supported && preference(1) == Some(1),
        blue_light_supported,
        blue_light_factor: if blue_light_supported {
            blue_light_factor(preference(7).unwrap_or(0))
        } else {
            1.0
        },
        page_up_down_supported,
        page_up_down_enabled: page_up_down_supported && preference(6) == Some(0),
        turbo_fan_active: turbo_fan_supported && app_fan.get(8).copied() == Some(1),
        slow_fan_active: slow_fan_supported && app_fan.get(9).copied() == Some(1),
        dtt_enabled: dtt_supported && preference(5) == Some(1),
        power_mode_locked: preference(8) == Some(1),
    }
}

fn blue_light_factor(raw: u8) -> f32 {
    (u16::from(raw.min(10)) + 10) as f32 / 10.0
}

fn le_u16(data: &[u8], offset: usize) -> u16 {
    data.get(offset..offset + 2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .unwrap_or(0)
}

fn le_u32(data: &[u8], offset: usize) -> u32 {
    data.get(offset..offset + 4)
        .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .unwrap_or(0)
}

fn battery_date(raw: u16) -> Option<String> {
    let year = 1980 + i32::from(raw >> 9);
    let month = (raw >> 5) & 0x0f;
    let day = raw & 0x1f;
    if !(2000..=2100).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn feature_version(data: &[u8]) -> String {
    let major = u16::from(data.first().copied().unwrap_or(0));
    let minor = u16::from(data.get(1).copied().unwrap_or(0));
    let version = (major << 8) | minor;
    format!("{}.{}", version >> 8, version & 0xff)
}

fn nonzero_keyboard(data: &[u8]) -> bool {
    data.iter().take(3).any(|value| *value != 0)
}

fn read_gpu_mode(data: &[u8], supported: bool) -> GpuModeState {
    let current = data.first().copied().unwrap_or(0);
    let option_bitmap = data.get(1).copied().unwrap_or(0);
    let options = gpu_mode_options(option_bitmap);
    GpuModeState {
        supported: supported && !options.is_empty(),
        current,
        current_label: gpu_mode_label(current).to_string(),
        options,
        restart_required: true,
    }
}

fn read_keyboard_settings(data: &[u8], controller_type: u8) -> KeyboardSettings {
    let fn_lock_supported = data.first().copied().unwrap_or(0) != 0;
    let windows_key_supported = data.get(1).copied().unwrap_or(0) != 0;
    let win_fn_supported = data.get(2).copied().unwrap_or(0) != 0;
    KeyboardSettings {
        supported: fn_lock_supported || windows_key_supported || win_fn_supported,
        controller_type,
        fn_lock: data.first().copied() == Some(1),
        fn_lock_supported,
        windows_key_enabled: data.get(1).copied() != Some(2),
        windows_key_supported,
        win_fn_swap: data.get(2).copied() == Some(1),
        win_fn_supported,
    }
}

fn keyboard_controller_type(fan_config: &[u8]) -> u8 {
    fan_config.get(15).copied().unwrap_or(0) & 0x0f
}

fn read_charging_settings(data: &[u8]) -> ChargingSettings {
    let start_options = data
        .get(16..32)
        .unwrap_or_default()
        .iter()
        .copied()
        .filter(|value| *value > 0 && *value <= 100)
        .collect::<Vec<_>>();
    let stop_options = data
        .get(32..48)
        .unwrap_or_default()
        .iter()
        .copied()
        .filter(|value| *value > 0 && *value <= 100)
        .collect::<Vec<_>>();
    ChargingSettings {
        supported: !start_options.is_empty() || !stop_options.is_empty(),
        enabled: data.first().copied().unwrap_or(0) == 1,
        start_percent: data.get(1).copied().unwrap_or(0),
        stop_percent: data.get(2).copied().unwrap_or(0),
        start_options,
        stop_options,
    }
}

struct FeatureInputs {
    power_modes: bool,
    power_mode_options: Vec<u8>,
    fan_control: bool,
    custom_curve_supported: bool,
    gpu_switch: bool,
    keyboard: bool,
    charging: bool,
    keyboard_rgb: bool,
    battery_utility: bool,
    energy_save: bool,
    turbo_fan: bool,
    slow_fan: bool,
    max_q_fan: bool,
    silent_fan: bool,
    cpu_overclock: bool,
    gpu_overclock: bool,
    flexikey: bool,
    per_key_keyboard: bool,
    keyboard_controller_type: u8,
    anti_dust: bool,
    dtt: bool,
    fan_offset_supported: bool,
    low_refresh_rate: bool,
    blue_light_filter: bool,
    battery_firmware: bool,
    flexi_access: bool,
    page_up_down_key: bool,
    control_center_function: bool,
    amd_manual_mode: bool,
}

fn build_feature_flags(input: &FeatureInputs) -> Vec<FeatureFlag> {
    let mut flags = Vec::new();
    let power_detail = if input.power_mode_options.is_empty() {
        "固件未声明可选电源配置".to_string()
    } else {
        let labels = input
            .power_mode_options
            .iter()
            .map(|mode| power_mode_label(*mode))
            .collect::<Vec<_>>()
            .join("、");
        format!("固件支持{labels}；已实现写入与回读")
    };
    push_feature(
        &mut flags,
        "power",
        "电源模式",
        input.power_modes,
        "writable",
        &power_detail,
    );
    push_feature(
        &mut flags,
        "fan",
        "风扇模式",
        input.fan_control,
        "writable",
        "已实现：自动与最大模式写入与回读",
    );
    push_feature(
        &mut flags,
        "fan_offset",
        "风扇偏移",
        input.fan_control && input.fan_offset_supported,
        "writable",
        "已实现：0-100% 曲线偏移写入与回读",
    );
    push_feature(
        &mut flags,
        "fan_curve",
        "风扇曲线",
        input.custom_curve_supported,
        "readOnly",
        "已读取固件温度到占空比映射",
    );
    push_feature(
        &mut flags,
        "gpu_switch",
        "GPU 模式",
        input.gpu_switch,
        "readOnly",
        "可读取当前模式与固件允许项，需要重启",
    );
    push_feature(
        &mut flags,
        "keyboard",
        "键盘设置",
        input.keyboard,
        "writable",
        &format!(
            "控制器类型 {}；已实现 FnLock、Win Key 与 Win/Fn 写入与回读",
            input.keyboard_controller_type
        ),
    );
    push_feature(
        &mut flags,
        "charging",
        "FlexiCharger",
        input.charging,
        "writable",
        "已实现充电开关与启停阈值写入、范围校验和回读",
    );
    push_feature(
        &mut flags,
        "rgb",
        "键盘 RGB",
        input.keyboard_rgb,
        "notImplemented",
        if input.per_key_keyboard {
            "固件声明单键灯效；灯效协议尚未接入"
        } else {
            "固件声明通用键盘灯效；灯效协议尚未接入"
        },
    );
    push_feature(
        &mut flags,
        "battery",
        "电池固件诊断",
        input.battery_utility || input.battery_firmware,
        "readOnly",
        "Windows 电池数据与 DCHU WMI 7 固件状态",
    );
    push_feature(
        &mut flags,
        "battery_refresh",
        "电池校准",
        input.battery_utility || input.battery_firmware,
        "blocked",
        "涉及放电、充电与 AC 控制，未纳入安全白名单",
    );
    push_feature(
        &mut flags,
        "energy_save",
        "节能功能",
        input.energy_save,
        "notImplemented",
        "固件声明能力，尚未完成实机验证",
    );
    push_feature(
        &mut flags,
        "anti_dust",
        "风扇除尘",
        input.anti_dust,
        "readOnly",
        "已读取运行状态与计划，启停写入未开放",
    );
    push_feature(
        &mut flags,
        "dtt",
        "动态散热 DTT",
        input.dtt,
        "readOnly",
        "只读：当前动态散热策略状态",
    );
    push_feature(
        &mut flags,
        "turbo_fan",
        "Turbo Fan",
        input.turbo_fan,
        "readOnly",
        "只读：固件运行标记",
    );
    push_feature(
        &mut flags,
        "slow_fan",
        "Slow Fan",
        input.slow_fan,
        "readOnly",
        "只读：固件运行标记",
    );
    push_feature(
        &mut flags,
        "max_q_fan",
        "Max-Q Fan",
        input.max_q_fan,
        "notImplemented",
        "由风扇配置字识别",
    );
    push_feature(
        &mut flags,
        "silent_fan",
        "Fanless",
        input.silent_fan,
        "notImplemented",
        "静音风扇固件能力",
    );
    push_feature(
        &mut flags,
        "flexikey",
        "Flexikey",
        input.flexikey,
        "notImplemented",
        "固件声明能力，映射协议未接入",
    );
    push_feature(
        &mut flags,
        "flexi_access",
        "FlexiAccess",
        input.flexi_access,
        "notImplemented",
        "依赖厂商 FlexiAccess 组件，当前仅识别能力位",
    );
    push_feature(
        &mut flags,
        "per_key",
        "单键 RGB",
        input.per_key_keyboard,
        "notImplemented",
        "固件能力位状态",
    );
    push_feature(
        &mut flags,
        "page_up_down",
        "PgUp / PgDn 功能键",
        input.page_up_down_key,
        "readOnly",
        "只读：功能键启用状态，写入未开放",
    );
    push_feature(
        &mut flags,
        "cc_function",
        "CC 功能键",
        input.control_center_function,
        "notImplemented",
        "已识别固件能力，功能键事件处理未接入",
    );
    push_feature(
        &mut flags,
        "low_refresh_rate",
        "低刷新率",
        input.low_refresh_rate,
        "writable",
        "已实现低刷新率策略写入、持久化与回读",
    );
    push_feature(
        &mut flags,
        "blue_light",
        "蓝光过滤",
        input.blue_light_filter,
        "writable",
        "已实现 1.0× 至 2.0× 强度调整与持久化",
    );
    push_feature(
        &mut flags,
        "cpu_oc",
        "CPU 超频",
        input.cpu_overclock,
        "blocked",
        "产品安全策略不开放超频写入",
    );
    push_feature(
        &mut flags,
        "gpu_oc",
        "GPU 超频",
        input.gpu_overclock,
        "blocked",
        "产品安全策略不开放超频写入",
    );
    push_feature(
        &mut flags,
        "amd_manual",
        "AMD 手动调频",
        input.amd_manual_mode,
        "blocked",
        "属于性能调校写入，产品安全策略不开放",
    );
    flags
}

fn push_feature(
    flags: &mut Vec<FeatureFlag>,
    id: &str,
    label: &str,
    supported: bool,
    access: &str,
    detail: &str,
) {
    let access = if supported || access == "blocked" {
        access
    } else {
        "unsupported"
    };
    flags.push(FeatureFlag {
        id: id.to_string(),
        label: label.to_string(),
        supported,
        access: access.to_string(),
        detail: if access == "unsupported" {
            "当前固件未声明此能力".to_string()
        } else {
            detail.to_string()
        },
    });
}

fn read_power_mode(api: &Dchu) -> Result<u8, String> {
    api.read_settings(1, 1, 1)
        .map_err(|error| error.to_string())?
        .first()
        .copied()
        .ok_or_else(|| "电源模式应答为空".to_string())
}

fn tach_to_rpm(raw: u16) -> u32 {
    if raw == 0 {
        0
    } else {
        (2_156_250.0 / raw as f64).round() as u32
    }
}

fn raw_duty(raw: u8) -> u8 {
    ((raw as f32 / 255.0) * 100.0).round() as u8
}

fn curve_from_app(
    id: &str,
    label: &str,
    data: &[u8],
    duty_offset: usize,
    temp_offset: usize,
) -> FanCurve {
    FanCurve {
        id: id.to_string(),
        label: label.to_string(),
        points: vec![
            FanCurvePoint {
                temperature_c: data[temp_offset],
                duty_percent: data[duty_offset],
            },
            FanCurvePoint {
                temperature_c: data[temp_offset + 1],
                duty_percent: data[duty_offset + 1],
            },
            FanCurvePoint {
                temperature_c: data[temp_offset + 2],
                duty_percent: data[duty_offset + 2],
            },
            FanCurvePoint {
                temperature_c: 100,
                duty_percent: 100,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blue_light_raw_values_map_to_supported_range() {
        assert_eq!(blue_light_factor(0), 1.0);
        assert_eq!(blue_light_factor(2), 1.2);
        assert_eq!(blue_light_factor(10), 2.0);
        assert_eq!(blue_light_factor(250), 2.0);
    }

    #[test]
    fn system_policy_decodes_firmware_preferences() {
        let preferences = [1, 1, 0, 0, 0, 1, 0, 2, 1];
        let mut fan = vec![0u8; 96];
        fan[8] = 1;
        fan[9] = 0;

        let policy = read_system_policy(&fan, Some(&preferences), true, true, true, true, true);

        assert!(policy.low_refresh_rate_supported);
        assert!(policy.low_refresh_rate_enabled);
        assert!(policy.blue_light_supported);
        assert_eq!(policy.blue_light_factor, 1.2);
        assert!(policy.page_up_down_supported);
        assert!(policy.page_up_down_enabled);
        assert!(policy.turbo_fan_active);
        assert!(!policy.slow_fan_active);
        assert!(policy.dtt_enabled);
        assert!(policy.power_mode_locked);
    }

    #[test]
    fn system_policy_defaults_to_safe_values_without_response() {
        let policy = read_system_policy(&[], None, false, false, false, false, false);

        assert!(!policy.low_refresh_rate_supported);
        assert!(!policy.low_refresh_rate_enabled);
        assert_eq!(policy.blue_light_factor, 1.0);
        assert!(!policy.page_up_down_enabled);
        assert!(!policy.turbo_fan_active);
        assert!(!policy.slow_fan_active);
        assert!(!policy.dtt_enabled);
        assert!(!policy.power_mode_locked);
    }

    #[test]
    fn battery_date_rejects_invalid_encoding() {
        assert_eq!(battery_date(0), None);
        assert_eq!(battery_date(0x5f0d).as_deref(), Some("2027-08-13"));
    }

    #[test]
    fn power_mode_options_follow_feature_package_bits() {
        let mut package = vec![0u8; 18];
        package[17] = 0x05;
        assert_eq!(power_mode_options(&Some(package), true), vec![0, 2]);
        assert!(power_mode_options(&None, false).is_empty());
        assert_eq!(power_mode_options(&None, true), vec![0, 2]);

        let mut empty_package = vec![0u8; 18];
        empty_package[17] = 0;
        assert_eq!(power_mode_options(&Some(empty_package), true), vec![0, 2]);
    }

    #[test]
    #[ignore = "requires the reference CLEVO hardware and signed DCHU service"]
    fn live_snapshot_exposes_verified_power_modes() {
        let snapshot = AppState::new(None).snapshot();

        assert_eq!(snapshot.source, "native");
        assert!(snapshot.capabilities.power_mode_options.contains(&0));
        assert!(snapshot.capabilities.power_mode_options.contains(&2));
    }
}
