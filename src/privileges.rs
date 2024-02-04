use std::os::raw::c_void;

use windows::Win32::{Foundation::{CloseHandle, HANDLE}, Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY}, System::Threading::{GetCurrentProcess, OpenProcessToken}};



pub fn is_privileged() -> Option<bool> {
    let mut handle:HANDLE = HANDLE::default();

    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle as *mut HANDLE)
    }.ok()?;

    let mut elevation:TOKEN_ELEVATION = TOKEN_ELEVATION::default();
    let mut size:u32 = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

    unsafe {
        GetTokenInformation(handle, TokenElevation, Some(&mut elevation as *mut TOKEN_ELEVATION as *mut c_void), size, &mut size as *mut u32)
    }.ok()?;

    unsafe {
        let _ = CloseHandle(handle);
    }

    Some(elevation.TokenIsElevated == 1)
}   