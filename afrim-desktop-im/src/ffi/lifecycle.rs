#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

use super::{AfrimDesktopEngine, ffi_mut, init_logging};

/// Create a new AfrimDesktop engine instance
/// Returns a pointer to the engine, or null on failure
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_new() -> *mut AfrimDesktopEngine {
    init_logging();
    let engine = Box::new(AfrimDesktopEngine::new());
    Box::into_raw(engine)
}

/// Load the afrim configuration selected by the user settings
/// (`$AFRIM_CONFIG`, `[afrim] config_path`, or the built-in sample).
/// Returns 0 on success, -1 on failure (the previous dataset stays active).
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_init(engine: *mut AfrimDesktopEngine) -> c_int {
    let engine = ffi_mut!(engine, -1);

    match engine.engine.init_afrim(&engine.settings) {
        Ok(()) => 0,
        Err(e) => {
            tracing::error!("Failed to load afrim config: {:#}", e);
            -1
        }
    }
}

/// Destroy a AfrimDesktop engine instance
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_free(engine: *mut AfrimDesktopEngine) {
    if !engine.is_null() {
        // SAFETY: Pointer is non-null (checked above) and was created by Box::into_raw in afrim_desktop_engine_new
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}
