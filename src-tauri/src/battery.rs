use crate::models::{BatteryInfo, BatteryRuntime};
use serde::Deserialize;
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BatteryStaticData {
    designed_capacity: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BatteryFullChargedCapacity {
    full_charged_capacity: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BatteryCycleCount {
    cycle_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32Battery {
    name: Option<String>,
}

pub fn static_info() -> BatteryInfo {
    let mut info = BatteryInfo::default();

    let mut designed = 0;
    let mut full = 0;
    let mut cycles = 0;

    if let Ok(com) = COMLibrary::new() {
        if let Ok(connection) = WMIConnection::with_namespace_path("ROOT\\WMI", com) {
            designed = connection
                .raw_query::<BatteryStaticData>("SELECT DesignedCapacity FROM BatteryStaticData")
                .ok()
                .and_then(|rows| rows.into_iter().find_map(|row| row.designed_capacity))
                .unwrap_or(0);
            full = connection
                .raw_query::<BatteryFullChargedCapacity>(
                    "SELECT FullChargedCapacity FROM BatteryFullChargedCapacity",
                )
                .ok()
                .and_then(|rows| rows.into_iter().find_map(|row| row.full_charged_capacity))
                .unwrap_or(0);
            cycles = connection
                .raw_query::<BatteryCycleCount>("SELECT CycleCount FROM BatteryCycleCount")
                .ok()
                .and_then(|rows| rows.into_iter().find_map(|row| row.cycle_count))
                .unwrap_or(0);
        }
    }

    let model = COMLibrary::new()
        .ok()
        .and_then(|com| WMIConnection::new(com).ok())
        .and_then(|connection| {
            connection
                .raw_query::<Win32Battery>("SELECT Name FROM Win32_Battery")
                .ok()
                .and_then(|rows| rows.into_iter().find_map(|row| row.name))
        })
        .unwrap_or_default();

    info.present = designed > 0 || full > 0;
    info.model = model;
    info.designed_capacity_mwh = designed;
    info.full_charged_capacity_mwh = full;
    info.cycle_count = cycles;
    info.health_percent = if designed > 0 && full > 0 {
        ((full as f64 / designed as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8
    } else {
        0
    };
    info
}

pub fn runtime() -> BatteryRuntime {
    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
        return BatteryRuntime::default();
    }

    let no_battery = status.BatteryFlag & 0x80 != 0;
    BatteryRuntime {
        present: !no_battery && status.BatteryLifePercent != 255,
        power_online: status.ACLineStatus == 1,
        charging: status.BatteryFlag & 0x08 != 0,
        discharging: status.ACLineStatus == 0 && status.BatteryLifePercent != 255,
        charge_percent: if status.BatteryLifePercent == 255 {
            0
        } else {
            status.BatteryLifePercent
        },
    }
}
