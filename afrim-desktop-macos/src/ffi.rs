//! C FFI interface for macOS IMKit integration (Swift → Rust)
//!
//! This module provides C-compatible functions that can be called from
//! the Swift IMKInputController wrapper.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CString, c_char, c_int, c_uint};
use std::ptr;
use std::sync::Once;

use afrim_desktop_im::core::engine::EngineAction;

use crate::engine_bridge::EngineBridge;

static INIT_LOGGING: Once = Once::new();

fn init_logging() {
    INIT_LOGGING.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
            )
            .with_writer(std::io::stderr)
            .init();
    });
}

/// Null-check + deref for `*const` FFI pointers. Returns `$default` if null.
macro_rules! ffi_ref {
    ($ptr:expr, $default:expr) => {{
        if $ptr.is_null() {
            return $default;
        }
        unsafe { &*$ptr }
    }};
}

/// Null-check + deref for `*mut` FFI pointers. Returns `$default` if null.
/// Use without default for void functions.
macro_rules! ffi_mut {
    ($ptr:expr) => {{
        if $ptr.is_null() {
            return;
        }
        unsafe { &mut *$ptr }
    }};
    ($ptr:expr, $default:expr) => {{
        if $ptr.is_null() {
            return $default;
        }
        unsafe { &mut *$ptr }
    }};
}

// --- Cached state for FFI consumption ---

#[derive(Default)]
struct PreeditCache {
    text: CString,
    caret_bytes: u32,
    dirty: bool,
}

#[derive(Default)]
struct CandidateCache {
    texts: Vec<CString>,
    annotations: Vec<CString>,
    count: usize,
    cursor: usize,
    dirty: bool,
    hide: bool,
}

#[derive(Default)]
struct CommitCache {
    text: CString,
    dirty: bool,
}

#[derive(Default)]
struct AuxCache {
    text: CString,
    dirty: bool,
}

/// Opaque handle to a macOS IME engine instance
pub struct AfrimDesktopMacEngine {
    bridge: EngineBridge,
    preedit: PreeditCache,
    candidates: CandidateCache,
    commit: CommitCache,
    aux: AuxCache,
}

impl AfrimDesktopMacEngine {
    fn new() -> Self {
        Self {
            bridge: EngineBridge::new(),
            preedit: PreeditCache::default(),
            candidates: CandidateCache::default(),
            commit: CommitCache::default(),
            aux: AuxCache::default(),
        }
    }

    fn clear_flags(&mut self) {
        self.preedit.dirty = false;
        self.candidates.dirty = false;
        self.candidates.hide = false;
        self.commit.dirty = false;
        self.aux.dirty = false;
    }

    fn apply_actions(&mut self, actions: Vec<EngineAction>) {
        for action in actions {
            match action {
                EngineAction::UpdatePreedit(preedit) => {
                    let caret_chars = preedit.caret();
                    let caret_bytes = preedit
                        .text()
                        .char_indices()
                        .nth(caret_chars)
                        .map(|(i, _)| i)
                        .unwrap_or(preedit.text().len());
                    self.preedit.caret_bytes = caret_bytes as u32;
                    self.preedit.text = CString::new(preedit.text()).unwrap_or_default();
                    self.preedit.dirty = true;
                }
                EngineAction::ShowCandidates(candidates) => {
                    let page = candidates.page_candidates();
                    self.candidates.texts = page
                        .iter()
                        .filter_map(|c| CString::new(c.text.as_str()).ok())
                        .collect();
                    self.candidates.annotations = page
                        .iter()
                        .map(|c| {
                            let ann = c.annotation.as_deref().unwrap_or("");
                            CString::new(ann).unwrap_or_default()
                        })
                        .collect();
                    self.candidates.count = self.candidates.texts.len();
                    self.candidates.cursor = candidates.page_cursor();
                    self.candidates.dirty = true;
                    self.candidates.hide = false;
                }
                EngineAction::HideCandidates => {
                    self.candidates.hide = true;
                    self.candidates.dirty = true;
                }
                EngineAction::Commit(text) => {
                    self.commit.text = CString::new(text).unwrap_or_default();
                    self.commit.dirty = true;
                }
                EngineAction::UpdateAuxText(text) => {
                    self.aux.text = CString::new(text).unwrap_or_default();
                    self.aux.dirty = true;
                }
                EngineAction::HideAuxText => {
                    self.aux.text = CString::default();
                    self.aux.dirty = true;
                }
            }
        }
    }
}

// =====================
// Lifecycle functions
// =====================

/// Create a new AfrimDesktop macOS engine instance.
/// Returns a pointer to the engine, or null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_engine_new() -> *mut AfrimDesktopMacEngine {
    init_logging();
    let engine = Box::new(AfrimDesktopMacEngine::new());
    Box::into_raw(engine)
}

/// Load the afrim configuration selected by the user settings.
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_engine_init(engine: *mut AfrimDesktopMacEngine) -> c_int {
    let engine = ffi_mut!(engine, -1);
    match engine.bridge.initialize() {
        Ok(()) => 0,
        Err(e) => {
            tracing::error!("afrim_desktop_macos_engine_init failed: {}", e);
            -1
        }
    }
}

/// Destroy a AfrimDesktop macOS engine instance.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_engine_free(engine: *mut AfrimDesktopMacEngine) {
    if !engine.is_null() {
        // SAFETY: Pointer is non-null and was created by Box::into_raw in afrim_desktop_macos_engine_new
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

// =====================
// Input functions
// =====================

/// Process a macOS key event.
///
/// - `keycode`: Carbon key code from NSEvent.keyCode
/// - `unicode_char`: first character from NSEvent.characters (0 if none)
/// - `has_unicode`: 1 if unicode_char is valid, 0 if not
/// - `shift`, `ctrl`, `opt`, `cmd`: modifier flags
/// - `is_press`: 1 for key down, 0 for key up
///
/// Returns 1 if the key was consumed, 0 if not.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_process_key(
    engine: *mut AfrimDesktopMacEngine,
    keycode: u16,
    unicode_char: u32,
    has_unicode: c_int,
    shift: c_int,
    ctrl: c_int,
    opt: c_int,
    cmd: c_int,
    is_press: c_int,
) -> c_int {
    let engine = ffi_mut!(engine, 0);
    engine.clear_flags();

    let uc = if has_unicode != 0 {
        char::from_u32(unicode_char)
    } else {
        None
    };

    let result = engine.bridge.process_key(
        keycode,
        uc,
        shift != 0,
        ctrl != 0,
        opt != 0,
        cmd != 0,
        is_press != 0,
    );

    engine.apply_actions(result.actions);

    if result.consumed { 1 } else { 0 }
}

/// Reset the engine state.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_reset(engine: *mut AfrimDesktopMacEngine) {
    let engine = ffi_mut!(engine);
    engine.bridge.reset();
    engine.preedit = PreeditCache::default();
    engine.candidates = CandidateCache::default();
    engine.commit = CommitCache::default();
    engine.aux = AuxCache::default();
}

/// Commit any pending input text.
/// Returns 1 if text was committed, 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_commit(engine: *mut AfrimDesktopMacEngine) -> c_int {
    let engine = ffi_mut!(engine, 0);
    let text = engine.bridge.commit();

    if text.is_empty() {
        return 0;
    }

    engine.commit.text = CString::new(text).unwrap_or_default();
    engine.commit.dirty = true;
    1
}

/// Commit the candidate at `index` (0-based) on the current candidate page.
/// Returns 1 if a candidate was committed, 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_select_candidate(
    engine: *mut AfrimDesktopMacEngine,
    index: c_uint,
) -> c_int {
    let engine = ffi_mut!(engine, 0);
    engine.clear_flags();

    let result = engine.bridge.select_candidate(index as usize);
    engine.apply_actions(result.actions);

    if result.consumed { 1 } else { 0 }
}

/// Save the learning cache to disk.
///
/// Kept for ABI compatibility: the afrim engine has no learning cache.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_save_learning(_engine: *mut AfrimDesktopMacEngine) {}

/// Set the surrounding text context from the editor.
///
/// Kept for ABI compatibility: afrim does not use surrounding text.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_set_surrounding_text(
    _engine: *mut AfrimDesktopMacEngine,
    _text: *const c_char,
    _cursor_pos: c_uint,
) {
}

// =====================
// Query functions
// =====================

/// Check if there's a preedit update pending.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_has_preedit(engine: *const AfrimDesktopMacEngine) -> c_int {
    let engine = ffi_ref!(engine, 0);
    if engine.preedit.dirty { 1 } else { 0 }
}

/// Get the current preedit text (null-terminated UTF-8).
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_preedit(
    engine: *const AfrimDesktopMacEngine,
) -> *const c_char {
    let engine = ffi_ref!(engine, ptr::null());
    engine.preedit.text.as_ptr()
}

/// Get the preedit length in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_preedit_len(
    engine: *const AfrimDesktopMacEngine,
) -> c_uint {
    let engine = ffi_ref!(engine, 0);
    engine.preedit.text.as_bytes().len() as c_uint
}

/// Get the preedit caret position in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_preedit_caret(
    engine: *const AfrimDesktopMacEngine,
) -> c_uint {
    let engine = ffi_ref!(engine, 0);
    engine.preedit.caret_bytes as c_uint
}

/// Check if there's a commit pending.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_has_commit(engine: *const AfrimDesktopMacEngine) -> c_int {
    let engine = ffi_ref!(engine, 0);
    if engine.commit.dirty { 1 } else { 0 }
}

/// Get the commit text (null-terminated UTF-8).
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_commit(
    engine: *const AfrimDesktopMacEngine,
) -> *const c_char {
    let engine = ffi_ref!(engine, ptr::null());
    engine.commit.text.as_ptr()
}

/// Get the commit text length in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_commit_len(
    engine: *const AfrimDesktopMacEngine,
) -> c_uint {
    let engine = ffi_ref!(engine, 0);
    engine.commit.text.as_bytes().len() as c_uint
}

/// Check if there's a candidates update pending.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_has_candidates(
    engine: *const AfrimDesktopMacEngine,
) -> c_int {
    let engine = ffi_ref!(engine, 0);
    if engine.candidates.dirty { 1 } else { 0 }
}

/// Check if candidates should be hidden.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_should_hide_candidates(
    engine: *const AfrimDesktopMacEngine,
) -> c_int {
    let engine = ffi_ref!(engine, 0);
    if engine.candidates.hide { 1 } else { 0 }
}

/// Get the number of candidates on the current page.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_candidate_count(
    engine: *const AfrimDesktopMacEngine,
) -> c_uint {
    let engine = ffi_ref!(engine, 0);
    engine.candidates.count as c_uint
}

/// Get a candidate text by index on the current page.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_candidate(
    engine: *const AfrimDesktopMacEngine,
    index: c_uint,
) -> *const c_char {
    let engine = ffi_ref!(engine, ptr::null());
    engine
        .candidates
        .texts
        .get(index as usize)
        .map(|c| c.as_ptr())
        .unwrap_or(ptr::null())
}

/// Get a candidate annotation by index on the current page.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_candidate_annotation(
    engine: *const AfrimDesktopMacEngine,
    index: c_uint,
) -> *const c_char {
    let engine = ffi_ref!(engine, ptr::null());
    engine
        .candidates
        .annotations
        .get(index as usize)
        .map(|c| c.as_ptr())
        .unwrap_or(ptr::null())
}

/// Get the current candidate cursor position within the page.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_candidate_cursor(
    engine: *const AfrimDesktopMacEngine,
) -> c_uint {
    let engine = ffi_ref!(engine, 0);
    engine.candidates.cursor as c_uint
}

/// Check if there's an aux text update pending.
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_has_aux(engine: *const AfrimDesktopMacEngine) -> c_int {
    let engine = ffi_ref!(engine, 0);
    if engine.aux.dirty { 1 } else { 0 }
}

/// Get the aux text (null-terminated UTF-8).
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_get_aux(
    engine: *const AfrimDesktopMacEngine,
) -> *const c_char {
    let engine = ffi_ref!(engine, ptr::null());
    engine.aux.text.as_ptr()
}

/// Get the current input mode as an integer.
/// 0 = Afrim, 1 = Direct input
#[unsafe(no_mangle)]
pub extern "C" fn afrim_desktop_macos_input_mode(engine: *const AfrimDesktopMacEngine) -> c_int {
    let engine = ffi_ref!(engine, 0);
    match engine.bridge.input_mode() {
        afrim_desktop_im::InputMode::Afrim => 0,
        afrim_desktop_im::InputMode::Direct => 1,
    }
}
