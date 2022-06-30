mod bds;
mod interop;

extern crate core;

#[no_mangle]
pub extern "C" fn rDllMain(_: *const u8, t: u32, _: *const u8) -> u32 {
    // Checking start reason to make sure DLL isn't being unloaded/changed
    if t != 1 {
        if t == 0 {
            println!("[Spigot Loader ERROR] DLL IS BEING UNLOADED");
        }
        return 1;
    }

    // Initializing the server
    bds::init();
    interop::init();

    // Printing final setup message and returning
    println!("[Spigot Loader INFO] Initialization complete");
    1
}