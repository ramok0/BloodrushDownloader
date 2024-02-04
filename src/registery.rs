use core::fmt;
use std::{ffi::CString, os::windows, path::Display};

use ::windows::{core::s, Win32::{Foundation::MAX_PATH, System::Registry::{RegOpenKeyExA, RegQueryValueExA, HKEY, HKEY_LOCAL_MACHINE, KEY_READ}}};



pub struct Registery {

}

#[derive(Debug)]
pub enum RegisteryError {
    KeyNotFound,
    ValueNotFound,
    InvalidValue,
    InvalidKey,
    InvalidUtf8,
    Unknown
}

impl fmt::Display for RegisteryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisteryError::KeyNotFound => write!(f, "Key not found"),
            RegisteryError::ValueNotFound => write!(f, "Value not found"),
            RegisteryError::InvalidValue => write!(f, "Invalid value"),
            RegisteryError::InvalidKey => write!(f, "Invalid key"),
            RegisteryError::InvalidUtf8 => write!(f, "Invalid utf8"),
            RegisteryError::Unknown => write!(f, "Unknown error")
        }
    }
}

impl std::error::Error for RegisteryError {}

impl Registery {
    pub fn read_string<P0, P1>(lpsubkey: P0, lpvaluename:P1) -> Result<String, RegisteryError>
    where 
    P0 : windows_core::IntoParam<::windows_core::PCSTR>,
    P1 : windows_core::IntoParam<::windows_core::PCSTR>,
    {
        let mut h_key:HKEY = HKEY::default();

        

        unsafe { RegOpenKeyExA(HKEY_LOCAL_MACHINE, lpsubkey, 0, KEY_READ, &mut h_key as *mut HKEY) }.map_err(|error| {
            dbg!(error.code().0);
            dbg!(error.message().to_string());
            match error.code().0 {
                0x2 => RegisteryError::KeyNotFound, //ERROR_FILE_NOT_FOUND
                0xA1 => RegisteryError::InvalidKey, //ERROR_BAD_PATHNAME
                _ => RegisteryError::Unknown
            }
        })?;

        let mut result:Vec<u8> = vec![0u8; MAX_PATH as usize];
        let mut cb_data:u32 = result.capacity() as u32;

        unsafe {
            RegQueryValueExA(h_key, lpvaluename, None, None, Some(result.as_mut_ptr() as *mut u8), Some(&mut cb_data as *mut u32))
        }.map_err(|error| {
            dbg!(error.code().0);
            dbg!(error.message().to_string());

            match error.code().0 {
                0x2 => RegisteryError::ValueNotFound,
                0x1F => RegisteryError::InvalidValue,
                _ => RegisteryError::Unknown
            }
        })?;

        unsafe { result.set_len(cb_data as usize - 1) };
        let result =  unsafe { CString::from_vec_unchecked(result) }.to_str().map_err(|_| RegisteryError::InvalidUtf8)?.to_string();

        Ok(result)
    }
}