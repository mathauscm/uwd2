use std::env;

use windows::core::s;
use windows::Win32::Foundation::{BOOL, ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexA;

use crate::cache_pdb::get_rva;
use crate::error::UwdError;
use crate::explorer_modinfo::get_guid;

mod cache_pdb;
mod constants;
mod error;
mod explorer_modinfo;
mod fetch_pdb;
mod inject;
mod parse_pdb;
mod portable;
mod startup;

fn prog() -> String {
    env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
}

fn help() {
    println!(include_str!("../help.txt"), env!("CARGO_PKG_VERSION"), prog())
}

fn do_patch() -> Result<(), UwdError> {
    let guid = unsafe { get_guid()? };
    let rva = get_rva(&guid)?;
    println!("RVA is {rva:#x}");
    unsafe {
        inject::inject(rva)?;
        inject::refresh();
    }
    Ok(())
}

fn patch() {
    match do_patch() {
        Ok(()) => {}
        Err(UwdError::PatchAlreadyApplied) => println!("Patch already applied."),
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn do_install() -> Result<(), UwdError> {
    println!("Copying UWD2 to AppData...");
    let exe_path = portable::copy_to_local_appdata()?;
    println!("Copied to: {}", exe_path.display());

    println!("Creating scheduled task...");
    startup::install(exe_path.to_str().unwrap_or_default())?;

    println!("Applying patch...");
    match do_patch() {
        Ok(()) | Err(UwdError::PatchAlreadyApplied) => {}
        Err(e) => eprintln!("Warning: could not apply patch now: {e}"),
    }
    Ok(())
}

fn install() {
    match do_install() {
        Ok(()) => {
            println!("Installation complete. UWD2 will run automatically at logon.")
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn uninstall() {
    let task_ok = startup::uninstall().map_err(|e| eprintln!("Error removing task: {e}")).is_ok();
    let files_ok = portable::remove_from_local_appdata()
        .map_err(|e| eprintln!("Error removing files: {e}"))
        .is_ok();
    if task_ok && files_ok {
        println!("UWD2 uninstalled. The watermark will return after the next restart.");
    }
}

fn do_repair() -> Result<(), UwdError> {
    println!("Verifying installation...");
    let exe_path = portable::copy_to_local_appdata()?;
    startup::install(exe_path.to_str().unwrap_or_default())?;
    match do_patch() {
        Ok(()) | Err(UwdError::PatchAlreadyApplied) => {}
        Err(e) => return Err(e),
    }
    Ok(())
}

fn repair() {
    match do_repair() {
        Ok(()) => println!("Repair complete."),
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn main() {
    let _mutex = unsafe {
        CreateMutexA(None, BOOL(0), s!("Global\\UWD2WatermarkPatch"))
    };
    if _mutex.is_ok() && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        eprintln!("UWD2 is already running.");
        return;
    }

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        None | Some("patch") | Some("inject") => patch(),
        Some("install") => install(),
        Some("uninstall") => uninstall(),
        Some("status") => startup::status(),
        Some("repair") => repair(),
        Some("help") => help(),
        Some("about") => println!(include_str!("../about.txt"), env!("CARGO_PKG_VERSION")),
        Some(err) => eprintln!(
            "Invalid argument `{err}`. Run `{} help` to see all commands.",
            prog()
        ),
    }
}
