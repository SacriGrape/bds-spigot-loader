use std::fs;
use std::fs::File;
use json::{JsonValue, object, stringify};
use pdb::{FallibleIterator, PDB, SymbolData};
use regex::Regex;
use windows::Win32::System::Diagnostics::Debug::UnDecorateSymbolName;

fn main() {
    // Checking if the file exists
    if !std::path::Path::new("src/symbol_cache.json").exists() {
        // BDS Directory
        let bds_directory =
            String::from("C:\\Users\\evan6\\Downloads\\bedrock-server-1.19.1.01 (2)");
        let pdb_path = String::from("\\bedrock_server.pdb") + bds_directory.as_str();

        // Getting PDB file
        let file = File::open(pdb_path).expect("Failed to get PDB file");
        let mut pdb = PDB::open(file).expect("Failed to create PDB<File> from File");

        // Get symbols and Addresses from PDB
        let symbols = pdb
            .global_symbols()
            .expect("Failed to get symbols from PDB");
        let addresses = pdb.address_map().expect("Failed to get addresses from PDB");

        // Taking symbol information and creating a string to write to symbol_cache.l2
        let mut symbols_iterator = symbols.iter();

        let mut symbol_json = object::Object::new();
        while let Some(symbol) = symbols_iterator.next().expect("Failed to get next symbol") {
            match symbol.parse() {
                Ok(SymbolData::Public(data)) if data.function => {
                    // Grabbing values to store into Symbol Map
                    let offset =
                        u32::from(data.offset.to_rva(&addresses).unwrap_or_default()) as isize;
                    let dirty_symbol = &data.name.to_string();
                    let clean_symbol = undecorate_symbol(dirty_symbol, 0);
                    let clean_name = undecorate_symbol(dirty_symbol, 4096);

                    // Use regex to grab params
                    let re = Regex::new(r"(.+)\((.*)\)").expect("Failed to create Regex");
                    let cap = re.captures(clean_symbol.as_str());
                    let params = match cap {
                        Some(cap) => cap.get(2).expect("Failed to get capture group").as_str(),
                        None => "()",
                    };

                    let param_offset_json = symbol_json.get_mut(clean_name.as_str());
                    match param_offset_json {
                        Some(value) => value
                            .insert(params, offset)
                            .expect("Failed to insert param into clean_name"),
                        _ => {
                            symbol_json.insert(
                                clean_name.as_str(),
                                JsonValue::Object(object::Object::new()),
                            );
                            let param_offset_json = symbol_json
                                .get_mut(clean_name.as_str())
                                .expect("Failed to get param offset JSON");
                            param_offset_json
                                .insert(params, offset)
                                .expect("Failed to insert param offset");
                        }
                    }
                }
                _ => {}
            }
        }

        // Writing the string to the file
        fs::write(format!("{}\\{}", bds_directory, "symbol_cache.json"), stringify(symbol_json))
            .expect("Failed to write Symbol File");
    }
}

fn undecorate_symbol(symbol: &str, flags: u32) -> String {
    let mut undec_symbol = [0; 2048];
    let chars_written = unsafe { UnDecorateSymbolName(symbol, &mut undec_symbol, flags) };

    let undec_symbol = &undec_symbol[..chars_written as usize];
    let result =
        std::str::from_utf8(undec_symbol).expect("Failed to convert string array into key");
    String::from(result)
}
