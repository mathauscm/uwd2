use std::fmt;

#[derive(Debug)]
pub enum UwdError {
    ExplorerNotFound,
    ExplorerOpenFailed(windows::core::Error),
    PdbDownloadFailed(String),
    PdbParseFailed,
    PdbCacheFailed(std::io::Error),
    InjectionFailed(windows::core::Error),
    PatchAlreadyApplied,
    TaskCreateFailed(windows::core::Error),
    TaskDeleteFailed(windows::core::Error),
    InstallPathFailed(std::io::Error),
}

impl fmt::Display for UwdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UwdError::ExplorerNotFound => write!(f, "explorer.exe not found after 60 seconds"),
            UwdError::ExplorerOpenFailed(e) => write!(f, "failed to open explorer.exe: {e}"),
            UwdError::PdbDownloadFailed(msg) => write!(f, "failed to download PDB: {msg}"),
            UwdError::PdbParseFailed => write!(f, "failed to parse PDB or find target symbol"),
            UwdError::PdbCacheFailed(e) => write!(f, "PDB cache error: {e}"),
            UwdError::InjectionFailed(e) => write!(f, "memory injection failed: {e}"),
            UwdError::PatchAlreadyApplied => write!(f, "patch already applied"),
            UwdError::TaskCreateFailed(e) => write!(f, "failed to create scheduled task: {e}"),
            UwdError::TaskDeleteFailed(e) => write!(f, "failed to delete scheduled task: {e}"),
            UwdError::InstallPathFailed(e) => write!(f, "install path error: {e}"),
        }
    }
}
