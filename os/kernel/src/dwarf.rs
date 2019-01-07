extern crate gimli;
use std::slice;
use console::kprintln;

extern "C" {
    fn __debug_info_start();
    fn __debug_info_end();
    fn __debug_abbrev_start();
    fn __debug_abbrev_end();
    fn __debug_str_start();
    fn __debug_str_end();
}

pub unsafe fn get_function_from_pc(pc: usize) -> Option<String> {
    let endian = gimli::LittleEndian;
    let debug_info_start = __debug_info_start as usize;
    let debug_info_end = __debug_info_end as usize;
    let debug_info = gimli::DebugInfo::new(slice::from_raw_parts(debug_info_start as *const u8, debug_info_end - debug_info_start), endian);
    // the positions in debug_info are absolute
    let debug_abbrev_end = __debug_abbrev_end as usize;
    let debug_abbrev = gimli::DebugAbbrev::new(slice::from_raw_parts(0 as *const u8, debug_abbrev_end), endian);
    let debug_str_end = __debug_str_end as usize;
    let debug_str = gimli::DebugStr::new(slice::from_raw_parts(0 as *const u8, debug_str_end), endian);
    match get_function_from_pc_gimli(debug_info, debug_abbrev, debug_str, pc as u64) {
        Ok(res) => Some(res),
        Err(err) => {
            None
        }
    }
}

fn get_function_from_pc_gimli<R : gimli::Reader>(debug_info: gimli::DebugInfo<R>, debug_abbrev: gimli::DebugAbbrev<R>,
                    debug_str: gimli::DebugStr<R>, pc: u64) -> Result<String, gimli::Error>{
    let mut iter = debug_info.units();
    while let Some(unit) = iter.next()? {
        let abbrevs = unit.abbreviations(&debug_abbrev)?;

        let mut entries = unit.entries(&abbrevs);
        while let Some((_, entry)) = entries.next_dfs()? {
            if entry.tag() == gimli::DW_TAG_subprogram {
                // a function
                if let Some(gimli::AttributeValue::Addr(low_pc)) = entry.attr_value(gimli::DW_AT_low_pc)? {
                    if let Some(gimli::AttributeValue::Udata(high_pc)) = entry.attr_value(gimli::DW_AT_high_pc)? {
                        // address: [low_pc, low_pc + high_pc)
                        if low_pc <= pc && low_pc + high_pc > pc {
                            if let Some(gimli::AttributeValue::DebugStrRef(str_ref)) = entry.attr_value(gimli::DW_AT_name)? {
                                return Ok(debug_str.get_str(str_ref)?.to_string()?.to_string())
                            }
                            return Err(gimli::Error::NoEntryAtGivenOffset)
                        } else if low_pc > pc {
                            break
                        }
                    }
                }
            }
        }
    }
    Err(gimli::Error::NoEntryAtGivenOffset)
}