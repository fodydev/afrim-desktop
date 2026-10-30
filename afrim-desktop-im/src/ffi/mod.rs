//! C FFI interface for fcitx5 integration
//!
//! This module provides C-compatible functions that can be called from
//! the fcitx5 C++ addon wrapper.

use std::ffi::CString;
use std::sync::Once;

mod input;
mod lifecycle;
mod query;

#[cfg(test)]
mod tests;

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

// Make macros available to submodules
pub(crate) use ffi_mut;
pub(crate) use ffi_ref;

use crate::config::Settings;
use crate::core::engine::{EngineAction, InputMethodEngine};

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

/// Cached preedit text and caret position for FFI consumption.
#[derive(Default)]
struct PreeditCache {
    text: CString,
    caret_bytes: u32,
    dirty: bool,
}

/// Cached candidate list for FFI consumption.
#[derive(Default)]
struct CandidateCache {
    texts: Vec<CString>,
    annotations: Vec<CString>,
    count: usize,
    cursor: usize,
    dirty: bool,
    hide: bool,
}

/// Cached commit text for FFI consumption.
#[derive(Default)]
struct CommitCache {
    text: CString,
    dirty: bool,
}

/// Cached aux text for FFI consumption.
#[derive(Default)]
struct AuxCache {
    text: CString,
    dirty: bool,
}

/// Opaque handle to an IME engine instance
pub struct AfrimDesktopEngine {
    engine: InputMethodEngine,
    settings: Settings,
    preedit: PreeditCache,
    candidates: CandidateCache,
    commit: CommitCache,
    aux: AuxCache,
    /// Last process_key time in milliseconds (input to result, end-to-end)
    last_process_key_ms: u64,
}

impl AfrimDesktopEngine {
    fn new() -> Self {
        // Load user settings from config.toml, fall back to defaults
        let settings = Settings::load().unwrap_or_default();
        Self {
            engine: InputMethodEngine::new(),
            settings,
            preedit: PreeditCache::default(),
            candidates: CandidateCache::default(),
            commit: CommitCache::default(),
            aux: AuxCache::default(),
            last_process_key_ms: 0,
        }
    }

    fn clear_flags(&mut self) {
        self.preedit.dirty = false;
        self.candidates.dirty = false;
        self.candidates.hide = false;
        self.commit.dirty = false;
        self.aux.dirty = false;
    }

    /// Process engine actions and cache results for FFI consumption.
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
