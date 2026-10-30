use std::ffi::c_void;
use std::ptr;

#[repr(C)]
struct GammaRamp {
    red: [u16; 256],
    green: [u16; 256],
    blue: [u16; 256],
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetDC(window: *mut c_void) -> *mut c_void;
    fn ReleaseDC(window: *mut c_void, device_context: *mut c_void) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetDeviceGammaRamp(device_context: *mut c_void, ramp: *mut GammaRamp) -> i32;
    fn SetDeviceGammaRamp(device_context: *mut c_void, ramp: *const GammaRamp) -> i32;
}

pub fn set_blue_light_factor(factor: f32) -> Result<(), String> {
    if !(1.0..=2.0).contains(&factor) {
        return Err("蓝光过滤强度必须介于 1.0 和 2.0 之间".to_string());
    }

    let mut ramp = GammaRamp {
        red: [0; 256],
        green: [0; 256],
        blue: [0; 256],
    };

    unsafe {
        let device_context = GetDC(ptr::null_mut());
        if device_context.is_null() {
            return Err("无法获取显示设备上下文".to_string());
        }

        if GetDeviceGammaRamp(device_context, &mut ramp) == 0 {
            ReleaseDC(ptr::null_mut(), device_context);
            return Err("当前显示模式不允许调整伽马值".to_string());
        }

        for index in 0..256usize {
            let base = 256.0 * index as f64;
            ramp.red[index] = base as u16;
            ramp.green[index] = base as u16;
            ramp.blue[index] = (base / f64::from(factor)) as u16;
        }

        let result = SetDeviceGammaRamp(device_context, &ramp);
        ReleaseDC(ptr::null_mut(), device_context);
        if result == 0 {
            return Err("显示驱动拒绝了蓝光过滤设置".to_string());
        }
    }

    Ok(())
}
