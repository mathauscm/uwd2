use std::fs;

use crate::constants::data_dir;
use crate::error::UwdError;
use crate::fetch_pdb;
use crate::parse_pdb::parse_pdb;

pub fn get_rva(guid: &str) -> Result<u32, UwdError> {
    let dir = data_dir();
    let pdbpath = dir.join(format!("{guid}.rva"));

    if pdbpath.exists() {
        let file = fs::read(&pdbpath).map_err(UwdError::PdbCacheFailed)?;
        if let Ok(bytes) = <[u8; 4]>::try_from(file.as_slice()) {
            println!("PDB cached. Reading...");
            return Ok(u32::from_be_bytes(bytes));
        }
        // cache corrupted — delete and re-fetch
        println!("Cache corrupted. Re-fetching...");
        let _ = fs::remove_file(&pdbpath);
    }

    println!("PDB not cached. Fetching...");
    let url = fetch_pdb::build_url(guid);
    let pdbfile = fetch_pdb::fetch(&url)?;
    println!("Fetched! Parsing...");
    let rva = parse_pdb(pdbfile)?;
    println!("Parsed! Caching...");

    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(UwdError::PdbCacheFailed)?;
    }
    fs::create_dir_all(&dir).map_err(UwdError::PdbCacheFailed)?;
    fs::write(pdbpath, rva.to_be_bytes()).map_err(UwdError::PdbCacheFailed)?;
    println!("Cached!");
    Ok(rva)
}
