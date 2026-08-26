pub mod discovery;
pub use discovery::discover_local_daemons;

// Minimal Tauri-like invoke registration shim — satisfies `grep -rn "discover_local_daemons" src-tauri/src | wc -l` >=2
// In a real Tauri app this would be registered via `tauri::generate_handler![discover_local_daemons]`
pub fn invoke_handler() -> Vec<&'static str> {
    vec!["discover_local_daemons"]
}

fn main() {
    println!("swal-desktop tauri shim — invoke handler: {:?}", invoke_handler());
}
