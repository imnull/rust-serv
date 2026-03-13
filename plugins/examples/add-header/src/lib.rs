// Example Wasm plugin: Add custom headers
//
// This is a simple plugin that adds custom headers to responses.
// Compile with: cargo build --target wasm32-unknown-unknown --release

#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Memory for communication
static mut MEMORY: [u8; 131072] = [0; 131072]; // 128KB

// Plugin exports
#[no_mangle]
pub extern "C" fn plugin_init(_config_ptr: i32, _config_len: i32) -> i32 {
    // Initialization successful
    0
}

#[no_mangle]
pub extern "C" fn plugin_on_request(_req_ptr: i32, _req_len: i32, _result_ptr: i32) -> i32 {
    // Continue to next plugin
    // Write "Continue" to result
    unsafe {
        let msg = b"Continue";
        MEMORY[65536..65536+msg.len()].copy_from_slice(msg);
        MEMORY[65536+msg.len()] = 0; // null terminator
    }
    0
}

#[no_mangle]
pub extern "C" fn plugin_on_response(_res_ptr: i32, _res_len: i32, result_ptr: i32) -> i32 {
    // Add custom header to response
    // In real implementation, would deserialize response, modify, and serialize action

    unsafe {
        // For now, just return Continue
        let msg = b"Continue";
        let offset = result_ptr as usize;
        MEMORY[offset..offset+msg.len()].copy_from_slice(msg);
        MEMORY[offset+msg.len()] = 0;
    }

    0
}

#[no_mangle]
pub extern "C" fn plugin_unload() -> i32 {
    0
}

// Memory export
#[no_mangle]
pub extern "C" fn memory() -> *mut u8 {
    unsafe { MEMORY.as_mut_ptr() }
}
