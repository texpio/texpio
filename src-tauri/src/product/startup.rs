//! Only this application's HKCU Run value is modified, on explicit preference changes.
use super::effects::ProductError;
use windows::{
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::*,
    },
    core::{PCWSTR, w},
};
const RUN: &windows::core::PCWSTR = &w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}
fn key(write: bool) -> Result<Option<Key>, ProductError> {
    let mut key = HKEY::default();
    let error = unsafe {
        if write {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                *RUN,
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
        } else {
            RegOpenKeyExW(HKEY_CURRENT_USER, *RUN, None, KEY_QUERY_VALUE, &mut key)
        }
    };
    if error == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if error != ERROR_SUCCESS {
        return Err(ProductError::Unavailable);
    }
    Ok(Some(Key(key)))
}
#[derive(Clone)]
pub struct StartupValue(Option<(REG_VALUE_TYPE, Vec<u8>)>);
pub fn snapshot(name: &str) -> Result<StartupValue, ProductError> {
    let Some(key) = key(false)? else {
        return Ok(StartupValue(None));
    };
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut kind = REG_VALUE_TYPE::default();
    let mut size = 0u32;
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            None,
            Some(&mut size),
        )
    };
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(StartupValue(None));
    }
    if result != ERROR_SUCCESS || size > 65536 {
        return Err(ProductError::Unavailable);
    }
    let mut bytes = vec![0; size as usize];
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            Some(bytes.as_mut_ptr()),
            Some(&mut size),
        )
    };
    if result != ERROR_SUCCESS {
        return Err(ProductError::Unavailable);
    }
    bytes.truncate(size as usize);
    Ok(StartupValue(Some((kind, bytes))))
}
pub fn restore(name: &str, value: StartupValue) -> Result<(), ProductError> {
    let key = key(true)?.ok_or(ProductError::Unavailable)?;
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        match value.0 {
            Some((kind, bytes)) => {
                RegSetValueExW(key.0, PCWSTR(name.as_ptr()), None, kind, Some(&bytes))
            }
            None => RegDeleteValueW(key.0, PCWSTR(name.as_ptr())),
        }
    };
    if result == ERROR_SUCCESS || result == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(ProductError::Unavailable)
    }
}
pub fn set(name: &str, enabled: bool) -> Result<(), ProductError> {
    let value = if enabled {
        let executable = std::env::current_exe().map_err(|_| ProductError::Unavailable)?;
        let command =
            crate::lifecycle::startup_command(&executable).ok_or(ProductError::Unavailable)?;
        let bytes: Vec<u8> = command
            .encode_utf16()
            .chain(Some(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        StartupValue(Some((REG_SZ, bytes)))
    } else {
        StartupValue(None)
    };
    restore(name, value)
}
