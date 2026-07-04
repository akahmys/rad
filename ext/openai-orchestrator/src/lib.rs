#![deny(clippy::pedantic)]

// Import host RPC function
unsafe extern "C" {
    fn rad_host_rpc(ptr: *const u8, len: usize) -> u64;
}

/// Allocates memory on the guest side for the host to write data.
///
/// # Panics
/// This function does not panic under normal circumstances.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let size = match usize::try_from(size) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Deallocates memory previously allocated via `alloc`.
///
/// # Safety
/// This function is unsafe because it reconstructs a Vec from a raw pointer and capacity/length.
/// The caller must ensure that the pointer and size match a previous call to `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
    let size = match usize::try_from(size) {
        Ok(s) => s,
        Err(_) => return,
    };
    if !ptr.is_null() && size > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, size, size);
        }
    }
}

/// Receives an event from the host Core.
#[unsafe(no_mangle)]
pub extern "C" fn rad_on_event(_ptr: *const u8, _len: i32) -> u64 {
    // Return 0 to indicate success
    0
}
