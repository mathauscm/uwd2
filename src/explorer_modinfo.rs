use std::mem::size_of;
use std::thread;
use std::time::{Duration, Instant};

use windows::core::imp::CloseHandle;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, FALSE, HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::Debug::{
    SymGetModuleInfo64, SymInitialize, SymLoadModuleEx, SymSetOptions, IMAGEHLP_MODULE64,
    SYMOPT_UNDNAME, SYM_LOAD_FLAGS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleExA;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE,
};

use crate::constants::shell32_path;
use crate::error::UwdError;

const EXPLORER_TIMEOUT: Duration = Duration::from_secs(60);
const EXPLORER_RETRY_INTERVAL: Duration = Duration::from_secs(2);

fn find_explorer_pid() -> Option<u32> {
    sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::everything()),
    )
    .processes()
    .values()
    .find(|proc| proc.name().eq_ignore_ascii_case("explorer.exe"))
    .map(|proc| proc.pid().as_u32())
}

pub unsafe fn get_explorer_handle() -> Result<HANDLE, UwdError> {
    let access = PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
    let deadline = Instant::now() + EXPLORER_TIMEOUT;

    loop {
        if let Some(pid) = find_explorer_pid() {
            return OpenProcess(access, FALSE, pid).map_err(UwdError::ExplorerOpenFailed);
        }
        if Instant::now() >= deadline {
            return Err(UwdError::ExplorerNotFound);
        }
        println!("Waiting for explorer.exe...");
        thread::sleep(EXPLORER_RETRY_INTERVAL);
    }
}

pub unsafe fn get_guid() -> Result<String, UwdError> {
    let modinfo = get_shell32_modinfo()?;
    let sig = modinfo.PdbSig70.to_u128();
    let age = modinfo.PdbAge;
    Ok(format!("{sig:032X}{age:X}"))
}

pub unsafe fn get_shell32_offset() -> Result<u64, UwdError> {
    let modinfo = get_shell32_modinfo()?;
    Ok(modinfo.BaseOfImage)
}

pub unsafe fn get_shell32_modinfo() -> Result<IMAGEHLP_MODULE64, UwdError> {
    let explorerhandle = get_explorer_handle()?;

    SymInitialize(explorerhandle, PCSTR::null(), true)
        .map_err(UwdError::ExplorerOpenFailed)?;
    SymSetOptions(SYMOPT_UNDNAME);

    let path_str = shell32_path();
    let nullterminatedpath = format!("{path_str}\0");
    let name = PCSTR::from_raw(nullterminatedpath.as_ptr());

    let mut module = HMODULE::default();
    GetModuleHandleExA(0, name, &mut module as *mut HMODULE)
        .map_err(UwdError::ExplorerOpenFailed)?;

    let r = SymLoadModuleEx(
        explorerhandle,
        HANDLE::default(),
        name,
        PCSTR::null(),
        module.0 as u64,
        0,
        None,
        SYM_LOAD_FLAGS::default(),
    );
    if r == 0 {
        GetLastError();
    }

    let mut modinfo = IMAGEHLP_MODULE64 {
        SizeOfStruct: size_of::<IMAGEHLP_MODULE64>() as u32,
        ..Default::default()
    };
    SymGetModuleInfo64(explorerhandle, module.0 as u64, &mut modinfo as *mut IMAGEHLP_MODULE64)
        .map_err(UwdError::ExplorerOpenFailed)?;

    CloseHandle(explorerhandle.0);
    Ok(modinfo)
}
