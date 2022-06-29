extern crate core;

use detour::RawDetour;
use json::{JsonValue};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{mem, ptr, u32};
use std::fs::read_to_string;
use std::mem::ManuallyDrop;
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;

lazy_static! {
    pub static ref SYMBOL_MAP: Mutex<JsonValue> = Mutex::new(JsonValue::new_object());
}

macro_rules! bds_func {
    ($class:ident :: $method:ident ( $( $param_type:ty ),* ) -> $return_type:ty) => {{
        // Getting symbol_map
        let mut symbol_map: &JsonValue = &JsonValue::new_object();
        let static_symbol_map = $crate::SYMBOL_MAP.lock().expect("Failed to get symbol map lock");
        let symbol_map_entries = static_symbol_map.entries();
        for (_, symbol_map_value) in symbol_map_entries {
            symbol_map = symbol_map_value;
        }
        if symbol_map.is_empty() {
            panic!("Failed to get symbol map!");
        }

        // Getting values needed for function creation
        let clean_symbol = concat!(stringify!($class), "::", stringify!($method));
        let params = vec![$(std::stringify!($param_type) ),*];
            let mut param_offset_object = &JsonValue::new_object();
        let entries = symbol_map.entries();
        for (clean_name_entry, param_object) in entries {
            if clean_symbol == clean_name_entry {
                param_offset_object = param_object;
            }
        }
        if param_offset_object.is_empty() {
            panic!("Failed to find symbol name!");
        }

        let param_offset_entires = param_offset_object.entries();
        let mut offset = 0;
        for (param_entry, offset_entry) in param_offset_entires {
            let mut matched = true;
            for param in &params {
                if !param_entry.contains(param) && param != &"*const ()" {
                    matched = false;
                    break;
                }
            }

            if !matched {
                continue;
            }

            if offset == 0 {
                offset = offset_entry.as_isize().expect("Failed to get entry as i64");
            } else {
                panic!("Duplicate param_keys!!! Are you sure you provided enough information?");
            }
        }

        if offset == 0 {
            panic!("Couldn't find offset of function!");
        }

        let module_handle = unsafe {
            GetModuleHandleA(PCSTR(ptr::null())).expect("Failed to get Module Handle")
        };

        let function_offset = module_handle.0 + offset;

        let func: extern "C" fn($($param_type),*) -> $return_type =
            unsafe { mem::transmute(function_offset) };

        func
    }}
}

fn can_destroy_hook(_player: *const (), _block: *const ()) -> bool {
    println!("Working");
    false
}

#[no_mangle]
pub extern "C" fn DllMain(_: *const u8, t: u32, _: *const u8) -> u32 {
    if t != 1 {
        if t == 0 {
            println!("[Spigot Loader ERROR] DLL IS BEING UNLOADED");
        }
        return 1;
    }

    println!("[Spigot Loader INFO] Grabbing data from symbol file...");
    // Get symbol file

    let symbol_file = read_to_string("./symbol_cache.json").expect("Failed to read file as string");

    // Creating symbol_map
    let symbol_map = json::parse(symbol_file.as_str()).expect("Failed to read symbol map json");

    SYMBOL_MAP.lock().expect("Failed to get symbol map lock").insert("symbol_map", symbol_map).expect("Failed to insert value");
    println!("[Spigot Loader INFO] Finished grabbing info from symbol file!");
    let can_destroy_func = bds_func!(Player::canDestroy(*const (), *const ()) -> bool);

    let hook: RawDetour = unsafe { RawDetour::new(can_destroy_func as *const (), can_destroy_hook as *const ()).expect("Failed to create Detour")};
    unsafe { hook.enable().expect("Failed to enable hook!"); }
    let _hook = ManuallyDrop::new(hook);

    println!("[Spigot Loader INFO] Initialization complete");

    1
}
