use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::EventLog::{
    CloseEventLog, DeregisterEventSource, OpenEventLogW, RegisterEventSourceW, ReportEventW,
    EVENTLOG_INFORMATION_TYPE,
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

pub fn write_out_log(message: &str) -> Result<(), String> {
    let source = wide("PowerBIOSServer_Out");
    let message = wide(message);
    let strings = [message.as_ptr()];

    unsafe {
        let handle = RegisterEventSourceW(null(), source.as_ptr());
        if handle.is_null() {
            return Err("无法打开 PowerBIOSServer_Out 事件源".to_string());
        }
        let result = ReportEventW(
            handle,
            EVENTLOG_INFORMATION_TYPE,
            0,
            0,
            null_mut(),
            1,
            0,
            strings.as_ptr(),
            null(),
        );
        DeregisterEventSource(handle);
        if result == 0 {
            return Err("ReportEventW 写入事件失败".to_string());
        }
    }
    Ok(())
}

pub fn log_exists(name: &str) -> bool {
    let name = wide(name);
    unsafe {
        let handle: HANDLE = OpenEventLogW(null(), name.as_ptr());
        if handle.is_null() {
            return false;
        }
        CloseEventLog(handle);
    }
    true
}
