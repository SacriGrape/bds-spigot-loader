mod hook;

pub fn init() {
    println!("[Spigot Loader INFO] Grabbing data from symbol file...");
    hook::init();
}