use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;

use crate::error::UwdError;

pub fn install_path() -> PathBuf {
    BaseDirs::new()
        .expect("could not resolve home directory")
        .data_local_dir()
        .join("UWD2")
        .join("uwd2.exe")
}

pub fn copy_to_local_appdata() -> Result<PathBuf, UwdError> {
    let dest = install_path();
    fs::create_dir_all(dest.parent().unwrap()).map_err(UwdError::InstallPathFailed)?;
    let src = std::env::current_exe().unwrap();
    if src != dest {
        fs::copy(&src, &dest).map_err(UwdError::InstallPathFailed)?;
    }
    Ok(dest)
}

pub fn remove_from_local_appdata() -> Result<(), UwdError> {
    let dir = install_path()
        .parent()
        .expect("install path has no parent")
        .to_path_buf();
    if dir.exists() {
        fs::remove_dir_all(dir).map_err(UwdError::InstallPathFailed)?;
    }
    Ok(())
}
