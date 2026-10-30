#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int, c_uint};

use crate::core::keycode::{KeyEvent, KeyModifiers, Keysym};

use super::{AfrimDesktopEngine, ffi_mut};

/// Process a key event
/// Returns 1 if the key was consumed, 0 if not
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_process_key(
    engine: *mut AfrimDesktopEngine,
    keysym: c_uint,
    state: c_uint,
    is_release: c_int,
) -> c_int {
    let engine = ffi_mut!(engine, 0);
    engine.clear_flags();

    // Convert modifier state
    let modifiers = KeyModifiers::from_modifier_state(state);

    let key_event = KeyEvent::new(Keysym(keysym), modifiers, is_release == 0);
    let start = std::time::Instant::now();
    let result = engine.engine.process_key(&key_event);
    engine.last_process_key_ms = start.elapsed().as_millis() as u64;

    engine.apply_actions(result.actions);

    if result.consumed { 1 } else { 0 }
}

/// Reset the engine state
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_reset(engine: *mut AfrimDesktopEngine) {
    let engine = ffi_mut!(engine);
    engine.engine.reset();
    engine.preedit = super::PreeditCache::default();
    engine.candidates = super::CandidateCache::default();
    engine.commit = super::CommitCache::default();
    engine.aux = super::AuxCache::default();
}

/// Commit the candidate at `index` (0-based) on the current candidate page.
/// Returns 1 if a candidate was committed, 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_select_candidate(
    engine: *mut AfrimDesktopEngine,
    index: c_uint,
) -> c_int {
    let engine = ffi_mut!(engine, 0);
    engine.clear_flags();

    let result = engine.engine.select_candidate(index as usize);
    engine.apply_actions(result.actions);

    if result.consumed { 1 } else { 0 }
}

/// Set the surrounding text context from the editor.
///
/// Kept for ABI compatibility: afrim does not use surrounding text.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_engine_set_surrounding_text(
    _engine: *mut AfrimDesktopEngine,
    _text: *const c_char,
    _cursor_pos: c_uint,
) {
}
