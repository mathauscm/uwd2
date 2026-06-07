use std::io::Read;

use crate::error::UwdError;

pub fn build_url(guid: &str) -> String {
    format!("http://msdl.microsoft.com/download/symbols/shell32.pdb/{guid}/shell32.pdb")
}

pub fn fetch(url: &str) -> Result<Vec<u8>, UwdError> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| UwdError::PdbDownloadFailed(format!("{url}: {e}")))?;

    let len: usize = resp
        .header("Content-Length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(15_000_000);

    let mut buf: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader()
        .take(u64::MAX)
        .read_to_end(&mut buf)
        .map_err(|e| UwdError::PdbDownloadFailed(format!("read error: {e}")))?;
    Ok(buf)
}
