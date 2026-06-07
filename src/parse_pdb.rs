use std::io::Cursor;

use pdb::FallibleIterator;

use crate::error::UwdError;

pub fn parse_pdb(pdbfile: Vec<u8>) -> Result<u32, UwdError> {
    let pdbreader = Cursor::new(pdbfile);
    let mut shell32 = pdb::PDB::open(pdbreader).map_err(|_| UwdError::PdbParseFailed)?;
    let symbol_table = shell32.global_symbols().map_err(|_| UwdError::PdbParseFailed)?;
    let address_map = shell32.address_map().map_err(|_| UwdError::PdbParseFailed)?;

    for symbol in symbol_table.iter().iterator().flatten() {
        let Ok(data) = symbol.parse() else { continue };
        if let pdb::SymbolData::Public(d) = data {
            if d.name.to_string().contains("s_DesktopBuildPaint") && d.function {
                return d
                    .offset
                    .to_rva(&address_map)
                    .map(|rva| rva.0)
                    .ok_or(UwdError::PdbParseFailed);
            }
        }
    }
    Err(UwdError::PdbParseFailed)
}
