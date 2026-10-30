use libloading::Library;
use std::ffi::c_int;
use std::path::{Path, PathBuf};
use thiserror::Error;

type GetInteger = unsafe extern "system" fn(c_int, *mut c_int) -> c_int;
type GetBuffer = unsafe extern "system" fn(c_int, *mut u8) -> c_int;
type SetData = unsafe extern "system" fn(c_int, *const u8, c_int) -> c_int;
type SetDataEx = unsafe extern "system" fn(c_int, *const u8, c_int, *mut u8) -> c_int;
type ReadSettings = unsafe extern "system" fn(c_int, c_int, c_int, *mut u8) -> c_int;
type WriteSettings = unsafe extern "system" fn(c_int, c_int, c_int, *mut u8) -> c_int;

#[derive(Debug, Error)]
pub enum DchuError {
    #[error("未找到 InsydeDCHU.dll")]
    NotFound,
    #[error("无法加载原生桥接：{0}")]
    Load(String),
    #[error("原生 API 拒绝了 {0}")]
    Api(String),
}

pub struct Dchu {
    _library: Library,
    get_integer: GetInteger,
    get_buffer: GetBuffer,
    set_data: SetData,
    set_data_ex: SetDataEx,
    read_settings: ReadSettings,
    write_settings: WriteSettings,
}

impl Dchu {
    pub unsafe fn load(path: &Path) -> Result<Self, DchuError> {
        let library =
            unsafe { Library::new(path) }.map_err(|error| DchuError::Load(error.to_string()))?;

        let get_integer = unsafe {
            *library
                .get::<GetInteger>(b"GetDCHU_Data_Integer\0")
                .map_err(|error| DchuError::Load(error.to_string()))?
        };
        let get_buffer = unsafe {
            *library
                .get::<GetBuffer>(b"GetDCHU_Data_Buffer\0")
                .map_err(|error| DchuError::Load(error.to_string()))?
        };
        let set_data = unsafe {
            *library
                .get::<SetData>(b"SetDCHU_Data\0")
                .map_err(|error| DchuError::Load(error.to_string()))?
        };
        let set_data_ex = unsafe {
            *library
                .get::<SetDataEx>(b"SetDCHU_DataEx\0")
                .map_err(|error| DchuError::Load(error.to_string()))?
        };
        let read_settings = unsafe {
            *library
                .get::<ReadSettings>(b"ReadAppSettings\0")
                .map_err(|error| DchuError::Load(error.to_string()))?
        };
        let write_settings = unsafe {
            *library
                .get::<WriteSettings>(b"WriteAppSettings\0")
                .map_err(|error| DchuError::Load(error.to_string()))?
        };

        Ok(Self {
            _library: library,
            get_integer,
            get_buffer,
            set_data,
            set_data_ex,
            read_settings,
            write_settings,
        })
    }

    pub fn get_integer(&self, command: u32) -> u32 {
        let mut value: c_int = 0;
        unsafe {
            (self.get_integer)(command as c_int, &mut value);
        }
        value as u32
    }

    pub fn get_buffer(&self, command: u32) -> Vec<u8> {
        let mut buffer = vec![0u8; 256];
        unsafe {
            (self.get_buffer)(command as c_int, buffer.as_mut_ptr());
        }
        buffer
    }

    pub fn read_settings(
        &self,
        page: u32,
        offset: u32,
        length: usize,
    ) -> Result<Vec<u8>, DchuError> {
        let mut buffer = vec![0u8; length];
        let result = unsafe {
            (self.read_settings)(
                page as c_int,
                offset as c_int,
                length as c_int,
                buffer.as_mut_ptr(),
            )
        };
        if result < 0 || buffer.len() != length {
            return Err(DchuError::Api(format!(
                "ReadAppSettings({page}, {offset}, {length})"
            )));
        }
        Ok(buffer)
    }

    pub fn write_settings(&self, page: u32, offset: u32, value: &[u8]) -> Result<(), DchuError> {
        let mut buffer = value.to_vec();
        let result = unsafe {
            (self.write_settings)(
                page as c_int,
                offset as c_int,
                buffer.len() as c_int,
                buffer.as_mut_ptr(),
            )
        };
        if result < 0 {
            return Err(DchuError::Api(format!(
                "WriteAppSettings({page}, {offset})"
            )));
        }
        Ok(())
    }

    pub fn set_command(&self, command: u32, sub_command: u8, value: u32) -> Result<(), DchuError> {
        let mut buffer = value.to_le_bytes();
        buffer[3] = sub_command;
        let result =
            unsafe { (self.set_data)(command as c_int, buffer.as_ptr(), buffer.len() as c_int) };
        if result < 0 {
            return Err(DchuError::Api(format!(
                "SetDCHU_Data({command}, sub={sub_command})"
            )));
        }
        Ok(())
    }

    pub fn feature_package(&self) -> Result<Vec<u8>, DchuError> {
        self.query_ex(&[8])
    }

    pub fn gpu_switch(&self) -> Result<Vec<u8>, DchuError> {
        self.query_ex(&[21])
    }

    pub fn keyboard_settings(&self) -> Result<Vec<u8>, DchuError> {
        self.query_ex(&[24])
    }

    pub fn apply_keyboard_settings(
        &self,
        fn_lock_mode: u8,
        windows_key_mode: u8,
        win_fn_mode: u8,
    ) -> Result<Vec<u8>, DchuError> {
        if !matches!(fn_lock_mode, 1 | 2)
            || !matches!(windows_key_mode, 1 | 2)
            || !matches!(win_fn_mode, 1 | 2)
        {
            return Err(DchuError::Api("键盘设置值超出固定枚举".to_string()));
        }
        self.query_ex(&[25, fn_lock_mode, windows_key_mode, win_fn_mode])
    }

    pub fn charger_settings(&self) -> Result<Vec<u8>, DchuError> {
        self.query_ex(&[30])
    }

    pub fn apply_charger_threshold(
        &self,
        enabled: bool,
        start_percent: u8,
        stop_percent: u8,
    ) -> Result<Vec<u8>, DchuError> {
        if start_percent == 0 || stop_percent > 100 || start_percent >= stop_percent {
            return Err(DchuError::Api("充电阈值范围无效".to_string()));
        }
        self.query_ex(&[31, u8::from(enabled), start_percent, stop_percent])
    }

    pub fn battery_package(&self) -> Vec<u8> {
        self.get_buffer(7)
    }

    pub fn anti_dust_status(&self) -> Result<u8, DchuError> {
        self.query_ex(&[1, 163, 0, 0, 0, 0, 184])
            .map(|response| response.get(1).copied().unwrap_or(0))
    }

    pub fn ec_version(&self) -> Result<String, DchuError> {
        let mut raw = Vec::with_capacity(18);
        let mut pages = Vec::with_capacity(3);
        for page in 1..=3u8 {
            pages.push(self.query_ex(&[1, page, 0, 0, 0, 0, 222])?);
        }

        for page in &pages {
            raw.extend_from_slice(&page[1..6]);
        }
        let value = format!("1.{}", String::from_utf8_lossy(&raw));
        Ok(value
            .split('$')
            .next()
            .unwrap_or_default()
            .trim_matches(char::from(0))
            .trim()
            .to_string())
    }

    fn query_ex(&self, payload: &[u8]) -> Result<Vec<u8>, DchuError> {
        let mut request = [0u8; 256];
        request[..payload.len()].copy_from_slice(payload);
        let mut response = [0u8; 256];
        let result = unsafe {
            (self.set_data_ex)(
                4,
                request.as_ptr(),
                request.len() as c_int,
                response.as_mut_ptr(),
            )
        };
        if result <= 0 {
            return Err(DchuError::Api(format!(
                "SetDCHU_DataEx(4, sub={})",
                payload.first().copied().unwrap_or(0)
            )));
        }
        Ok(response.to_vec())
    }
}

pub struct DchuRuntime {
    pub api: Option<Dchu>,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

impl DchuRuntime {
    pub fn discover(resource_dir: Option<&Path>) -> Self {
        let path = find_dll(resource_dir);
        let Some(path) = path else {
            return Self {
                api: None,
                path: None,
                error: Some(DchuError::NotFound.to_string()),
            };
        };

        match unsafe { Dchu::load(&path) } {
            Ok(api) => Self {
                api: Some(api),
                path: Some(path),
                error: None,
            },
            Err(error) => Self {
                api: None,
                path: Some(path),
                error: Some(error.to_string()),
            },
        }
    }
}

fn find_dll(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("CC30_DCHU_PATH") {
        candidates.push(PathBuf::from(path));
    }

    if let Some(resource_dir) = resource_dir {
        candidates.push(resource_dir.join("native/InsydeDCHU.dll"));
        candidates.push(resource_dir.join("resources/native/InsydeDCHU.dll"));
    }

    if let Ok(executable) = std::env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.push(directory.join("InsydeDCHU.dll"));
            candidates.push(directory.join("native/InsydeDCHU.dll"));
            candidates.push(directory.join("resources/native/InsydeDCHU.dll"));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        for ancestor in current_dir.ancestors().take(5) {
            candidates.push(ancestor.join(".tools/cc30-x64/ControlCenter30/InsydeDCHU.dll"));
            candidates.push(ancestor.join("cc30-next/src-tauri/resources/native/InsydeDCHU.dll"));
        }
    }

    for program_files in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Ok(base) = std::env::var(program_files) {
            let base = PathBuf::from(base);
            candidates.push(base.join("ControlCenter3.0/ControlCenter30/InsydeDCHU.dll"));
            candidates.push(base.join("CLEVO/ControlCenter3.0/ControlCenter30/InsydeDCHU.dll"));
        }
    }

    candidates.into_iter().find(|path| path.is_file())
}
