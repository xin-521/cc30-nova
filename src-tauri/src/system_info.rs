use crate::models::{MemoryModule, SystemInfo};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use windows_sys::Win32::System::SystemInformation::{
    ComputerNameDnsHostname, GetComputerNameExW, GetSystemFirmwareTable, GlobalMemoryStatusEx,
    MEMORYSTATUSEX,
};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn collect() -> SystemInfo {
    let mut info = SystemInfo::default();
    if let Some(smbios) = smbios() {
        info.bios_vendor = smbios.bios_vendor;
        info.bios_version = smbios.bios_version;
        info.bios_date = smbios.bios_date;
        info.manufacturer = smbios.manufacturer;
        info.product_name = smbios.product_name;
        info.serial_number = smbios.serial_number;
        info.family = smbios.family;
        info.board_manufacturer = smbios.board_manufacturer;
        info.board_product = smbios.board_product;
        info.board_version = smbios.board_version;
        info.cpu_cores = smbios.cpu_cores;
        info.cpu_threads = smbios.cpu_threads;
        info.memory_modules = smbios.memory_modules;
    }

    info.cpu_name = registry_string(
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
        "ProcessorNameString",
    )
    .unwrap_or_else(|| "未知处理器".to_string());

    info.os_name = os_name();
    info.os_build = os_build();
    info.hostname = hostname().unwrap_or_else(|| "未知".to_string());
    let module_total_gb: u64 = info
        .memory_modules
        .iter()
        .map(|module| module.size_gb as u64)
        .sum();
    info.memory_total_gb = if module_total_gb > 0 {
        module_total_gb
    } else {
        (total_memory_bytes() as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u64
    };
    info.gpu_names = gpu_names();
    info.battery = crate::battery::static_info();

    if info.memory_modules.is_empty() && info.memory_total_gb > 0 {
        info.memory_modules.push(MemoryModule {
            locator: "系统内存".to_string(),
            manufacturer: String::new(),
            part_number: String::new(),
            speed_mhz: 0,
            size_gb: info.memory_total_gb as u32,
        });
    }

    info
}

#[derive(Default)]
struct SmbiosInfo {
    bios_vendor: String,
    bios_version: String,
    bios_date: String,
    manufacturer: String,
    product_name: String,
    serial_number: String,
    family: String,
    board_manufacturer: String,
    board_product: String,
    board_version: String,
    cpu_cores: u32,
    cpu_threads: u32,
    memory_modules: Vec<MemoryModule>,
}

fn smbios() -> Option<SmbiosInfo> {
    // GetSystemFirmwareTable uses the registry-style fourcc in big-endian order.
    let signature = u32::from_be_bytes(*b"RSMB");
    let size = unsafe { GetSystemFirmwareTable(signature, 0, null_mut(), 0) };
    if size <= 8 {
        return None;
    }
    let mut table = vec![0u8; size as usize];
    let written =
        unsafe { GetSystemFirmwareTable(signature, 0, table.as_mut_ptr(), table.len() as u32) };
    if written == 0 {
        return None;
    }
    table.truncate(written as usize);
    if table.len() <= 8 {
        return None;
    }

    let mut result = SmbiosInfo::default();
    let mut cursor = 8usize;
    while cursor + 4 <= table.len() {
        let structure_type = table[cursor];
        let formatted_len = table[cursor + 1] as usize;
        if formatted_len < 4 || cursor + formatted_len > table.len() {
            break;
        }
        let record = &table[cursor..cursor + formatted_len];
        let (strings, next) = smbios_strings(&table, cursor + formatted_len);
        match structure_type {
            0 => {
                result.bios_vendor = string_at(record, &strings, 4);
                result.bios_version = string_at(record, &strings, 5);
                result.bios_date = string_at(record, &strings, 8);
            }
            1 => {
                result.manufacturer = string_at(record, &strings, 4);
                result.product_name = string_at(record, &strings, 5);
                result.serial_number = string_at(record, &strings, 7);
                result.family = string_at(record, &strings, 26);
            }
            2 => {
                result.board_manufacturer = string_at(record, &strings, 4);
                result.board_product = string_at(record, &strings, 5);
                result.board_version = string_at(record, &strings, 6);
            }
            4 => {
                result.cpu_cores = byte_at(record, 35).unwrap_or(0) as u32;
                result.cpu_threads = byte_at(record, 37).unwrap_or(0) as u32;
            }
            17 => {
                if let Some(size_gb) = memory_size_gb(record) {
                    if size_gb > 0 {
                        result.memory_modules.push(MemoryModule {
                            locator: string_at(record, &strings, 16),
                            manufacturer: string_at(record, &strings, 23),
                            part_number: string_at(record, &strings, 26),
                            speed_mhz: word_at(record, 21).unwrap_or(0) as u32,
                            size_gb,
                        });
                    }
                }
            }
            _ => {}
        }
        if structure_type == 127 || next <= cursor {
            break;
        }
        cursor = next;
    }
    Some(result)
}

fn smbios_strings(table: &[u8], formatted_end: usize) -> (Vec<String>, usize) {
    let mut strings = Vec::new();
    let mut cursor = formatted_end;
    if cursor >= table.len() {
        return (strings, cursor);
    }

    while cursor < table.len() {
        if table[cursor] == 0 {
            cursor += 1;
            // An empty string terminates the string-set. Consume its second
            // NUL, but leave the next structure's type byte untouched.
            if cursor < table.len() && table[cursor] == 0 {
                cursor += 1;
            }
            break;
        }

        let start = cursor;
        while cursor < table.len() && table[cursor] != 0 {
            cursor += 1;
        }
        strings.push(
            String::from_utf8_lossy(&table[start..cursor])
                .trim()
                .to_string(),
        );
        if cursor < table.len() {
            cursor += 1;
        }
    }
    (strings, cursor)
}

fn byte_at(record: &[u8], offset: usize) -> Option<u8> {
    record.get(offset).copied()
}

fn word_at(record: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
        *record.get(offset)?,
        *record.get(offset + 1)?,
    ]))
}

fn dword_at(record: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *record.get(offset)?,
        *record.get(offset + 1)?,
        *record.get(offset + 2)?,
        *record.get(offset + 3)?,
    ]))
}

fn string_at(record: &[u8], strings: &[String], offset: usize) -> String {
    let Some(index) = byte_at(record, offset).filter(|index| *index > 0) else {
        return String::new();
    };
    strings.get(index as usize - 1).cloned().unwrap_or_default()
}

fn memory_size_gb(record: &[u8]) -> Option<u32> {
    let size = word_at(record, 12)?;
    if size == 0 {
        return None;
    }
    let megabytes = if size == 0x7FFF {
        dword_at(record, 28)? & 0x7FFF_FFFF
    } else if size & 0x8000 != 0 {
        (size & 0x7FFF) as u32 / 1024
    } else {
        size as u32
    };
    Some(megabytes / 1024)
}

fn registry_string(path: &str, name: &str) -> Option<String> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(path)
        .ok()?
        .get_value::<String, _>(name)
        .ok()
        .map(|value| value.trim().to_string())
}

fn os_name() -> String {
    let path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let product = registry_string(path, "ProductName").unwrap_or_else(|| "Windows".to_string());
    let build = registry_string(path, "CurrentBuild")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    if build >= 22000 {
        product.replace("Windows 10", "Windows 11")
    } else {
        product
    }
}

fn os_build() -> String {
    let path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let build = registry_string(path, "CurrentBuild").unwrap_or_default();
    let revision = registry_string(path, "UBR").unwrap_or_default();
    let display = registry_string(path, "DisplayVersion").unwrap_or_default();
    match (build.is_empty(), revision.is_empty(), display.is_empty()) {
        (true, _, _) => "未知".to_string(),
        (false, false, false) => format!("{build}.{revision} · {display}"),
        (false, false, true) => format!("{build}.{revision}"),
        _ => build,
    }
}

fn hostname() -> Option<String> {
    let mut buffer = vec![0u16; 256];
    let mut size = buffer.len() as u32;
    let ok = unsafe { GetComputerNameExW(ComputerNameDnsHostname, buffer.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return None;
    }
    buffer.truncate(size as usize);
    Some(OsString::from_wide(&buffer).to_string_lossy().to_string())
}

fn total_memory_bytes() -> u64 {
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ok == 0 {
        0
    } else {
        status.ullTotalPhys
    }
}

fn gpu_names() -> Vec<String> {
    fn collect_from(root: &RegKey, names: &mut Vec<String>) {
        for key in root.enum_keys().flatten() {
            let Ok(child) = root.open_subkey(&key) else {
                continue;
            };
            for child_name in child
                .enum_keys()
                .flatten()
                .filter(|name| name.starts_with("000"))
            {
                let Ok(adapter) = child.open_subkey(&child_name) else {
                    continue;
                };
                let Ok(description) = adapter.get_value::<String, _>("DriverDesc") else {
                    continue;
                };
                if !description.is_empty()
                    && !description.contains("Microsoft Basic")
                    && !names.iter().any(|name| name == &description)
                {
                    names.push(description);
                }
            }
        }
    }

    let mut names = Vec::new();
    if let Ok(video) =
        RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SYSTEM\\CurrentControlSet\\Control\\Video")
    {
        collect_from(&video, &mut names);
    }
    names
}

#[cfg(test)]
mod tests {
    use super::{memory_size_gb, smbios_strings};

    #[test]
    fn parses_string_set_without_consuming_next_structure() {
        let table = b"Vendor\0BIOS\0Date\0\0\x01\x04\0\0";
        let (strings, next) = smbios_strings(table, 0);
        assert_eq!(strings, ["Vendor", "BIOS", "Date"]);
        assert_eq!(table[next], 1);
    }

    #[test]
    fn parses_empty_string_set() {
        let table = b"\0\0\x01";
        let (strings, next) = smbios_strings(table, 0);
        assert!(strings.is_empty());
        assert_eq!(table[next], 1);
    }

    #[test]
    fn decodes_megabyte_kilobyte_and_extended_memory_sizes() {
        let mut record = vec![0u8; 32];

        record[12..14].copy_from_slice(&8192u16.to_le_bytes());
        assert_eq!(memory_size_gb(&record), Some(8));

        record[12..14].copy_from_slice(&(0x8000u16 | 1024).to_le_bytes());
        assert_eq!(memory_size_gb(&record), Some(0));

        record[12..14].copy_from_slice(&0x7FFFu16.to_le_bytes());
        record[28..32].copy_from_slice(&16384u32.to_le_bytes());
        assert_eq!(memory_size_gb(&record), Some(16));
    }
}
