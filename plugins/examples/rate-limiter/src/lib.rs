// Example: Rate Limiter Plugin
//
// A more advanced plugin that implements rate limiting.
// Demonstrates:
// - Plugin state management
// - Request inspection
// - Response modification
// - Using host functions

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU64, Ordering};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Memory for communication
static mut MEMORY: [u8; 131072] = [0; 131072];

// Rate limit state (simple counter)
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static mut RATE_LIMIT: u64 = 100; // 100 requests per second

#[no_mangle]
pub extern "C" fn plugin_init(_config_ptr: i32, _config_len: i32) -> i32 {
    // In real implementation, would parse config and set RATE_LIMIT
    REQUEST_COUNT.store(0, Ordering::Relaxed);
    0
}

#[no_mangle]
pub extern "C" fn plugin_on_request(_req_ptr: i32, _req_len: i32, result_ptr: i32) -> i32 {
    // Check rate limit
    let count = REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    unsafe {
        if count > RATE_LIMIT {
            // Rate limit exceeded - return 429 error
            let action = br#"{"Intercept":{"status":429,"headers":{"X-RateLimit":"exceeded"},"body":"eyJtZXNzYWdlIjoiUmF0ZSBsaW1pdCBleGNlZWRlZCJ9"}}"#;

            let offset = result_ptr as usize;
            MEMORY[offset..offset+action.len()].copy_from_slice(action);
            MEMORY[offset+action.len()] = 0;

            0
        } else {
            // Continue
            let msg = b"Continue";
            let offset = result_ptr as usize;
            MEMORY[offset..offset+msg.len()].copy_from_slice(msg);
            MEMORY[offset+msg.len()] = 0;

            0
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_on_response(_res_ptr: i32, _res_len: i32, result_ptr: i32) -> i32 {
    // Add rate limit header to response
    let count = REQUEST_COUNT.load(Ordering::Relaxed);

    unsafe {
        // For simplicity, just continue
        // In real implementation, would modify response headers
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

#[no_mangle]
pub extern "C" fn memory() -> *mut u8 {
    unsafe { MEMORY.as_mut_ptr() }
}
