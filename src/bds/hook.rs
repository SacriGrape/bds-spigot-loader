mod bds_detour;

use std::sync::Mutex;
use std::{mem, ptr};
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use detour::RawDetour;
use json::JsonValue;
use lazy_static::lazy_static;
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use spigot_loader_macros::initialize_hook;

lazy_static! {
    pub static ref SYMBOL_MAP: Mutex<JsonValue> = {
        let symbol_file = include_str!("../symbol_cache.json");
        Mutex::new(json::parse(symbol_file).expect("Failed to read symbol map json"))
    };
}

lazy_static! {
    pub static ref HOOK_MAP: Mutex<HashMap<String, ManuallyDrop<RawDetour>>> = Mutex::new(HashMap::new());
}

#[macro_export]
macro_rules! bds_func {
    ($class:ident :: $method:ident ( $( $param_type:ty ),* ) -> $return_type:ty) => {{
        // Getting symbol_map
        let mut symbol_map = crate::bds::hook::SYMBOL_MAP.lock().expect("Failed to get symbol map lock");

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

pub fn init() {
    println!("[Spigot Loader INFO] Initializing hooks...");
    initialize_hook!(Player::canDestroy(*const (), *const ()) -> bool => player_can_destroy_detour);
}