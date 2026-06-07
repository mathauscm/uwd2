use std::ffi::c_void;

use windows::core::imp::CloseHandle;
use windows::core::s;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowA, GetWindow, GetWindowInfo, SendMessageA, GW_CHILD, WINDOWINFO, WM_COMMAND,
    WS_VISIBLE,
};

use crate::constants::*;
use crate::error::UwdError;
use crate::explorer_modinfo::{get_explorer_handle, get_shell32_offset};

pub unsafe fn inject(rva: u32) -> Result<(), UwdError> {
    println!("Getting shell32 offset...");
    let offset = get_shell32_offset()?;
    println!("Offset of shell32 inside explorer.exe is {offset:#x}");
    let handle = get_explorer_handle()?;
    let target = (offset + rva as u64) as *const c_void;

    // check if patch is already applied before writing
    let mut current = [0u8; RET.len()];
    ReadProcessMemory(
        handle,
        target,
        current.as_mut_ptr() as *mut c_void,
        RET.len(),
        None,
    )
    .ok();
    if current == RET {
        CloseHandle(handle.0);
        return Err(UwdError::PatchAlreadyApplied);
    }

    println!("Injecting ret...");
    WriteProcessMemory(handle, target, &RET as *const u8 as *const c_void, RET.len(), None)
        .map_err(UwdError::InjectionFailed)?;
    println!("Injected!");
    CloseHandle(handle.0);
    Ok(())
}

pub unsafe fn refresh() {
    println!("Refreshing desktop...");
    let h_wnd = GetWindow(FindWindowA(s!("Progman"), s!("Program Manager")), GW_CHILD);

    let h_wnd2 = GetWindow(h_wnd, GW_CHILD);
    let mut wi = WINDOWINFO::default();
    wi.cbSize = std::mem::size_of::<WINDOWINFO>() as u32;
    if GetWindowInfo(h_wnd2, &mut wi as *mut _).is_err() {
        return;
    }
    let visible = wi.dwStyle & WS_VISIBLE == WS_VISIBLE;

    if visible {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    } else {
        SendMessageA(h_wnd, WM_COMMAND, WPARAM(0x7402), LPARAM::default());
        SendMessageA(h_wnd, WM_COMMAND, WPARAM(0x7402), LPARAM::default());
    }
    println!("Refreshed!");
}
