use super::*;
use input::*;
use lifecycle::*;
use query::*;
use std::ffi::{CStr, c_char};
use std::ptr;

// XKB keysyms for common keys
const XKB_KEY_SPACE: u32 = 0x20;
const XKB_KEY_RETURN: u32 = 0xff0d;
const XKB_KEY_ESCAPE: u32 = 0xff1b;
const XKB_KEY_BACKSPACE: u32 = 0xff08;
const XKB_KEY_SHIFT_L: u32 = 0xffe1;
const SHIFT_MASK: u32 = crate::core::keycode::KeyModifiers::SHIFT_MASK;

/// RAII wrapper around a raw `AfrimDesktopEngine` pointer.
/// Automatically frees the engine on drop, preventing leaks in tests.
struct TestEngine(*mut AfrimDesktopEngine);

impl TestEngine {
    fn new() -> Self {
        let ptr = afrim_desktop_engine_new();
        assert!(!ptr.is_null());
        Self(ptr)
    }

    fn ptr(&self) -> *mut AfrimDesktopEngine {
        self.0
    }

    /// Send a key press event. Returns true if consumed.
    fn press(&self, keysym: u32) -> bool {
        afrim_desktop_engine_process_key(self.0, keysym, 0, 0) == 1
    }

    /// Send a key press event with modifier state. Returns true if consumed.
    fn press_with(&self, keysym: u32, state: u32) -> bool {
        afrim_desktop_engine_process_key(self.0, keysym, state, 0) == 1
    }

    /// Send a key release event. Returns true if consumed.
    fn release(&self, keysym: u32) -> bool {
        afrim_desktop_engine_process_key(self.0, keysym, 0, 1) == 1
    }

    /// Type an ASCII string.
    fn type_str(&self, input: &str) {
        for ch in input.chars() {
            assert!(self.press(ch as u32), "'{ch}' should be consumed");
        }
    }

    fn cstr(ptr: *const c_char) -> &'static str {
        if ptr.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(ptr) }.to_str().unwrap()
    }

    fn preedit(&self) -> &str {
        Self::cstr(afrim_desktop_engine_get_preedit(self.0))
    }

    fn commit_text(&self) -> &str {
        Self::cstr(afrim_desktop_engine_get_commit(self.0))
    }

    fn aux(&self) -> &str {
        Self::cstr(afrim_desktop_engine_get_aux(self.0))
    }

    fn candidate(&self, index: u32) -> &str {
        Self::cstr(afrim_desktop_engine_get_candidate(self.0, index))
    }

    fn candidate_annotation(&self, index: u32) -> &str {
        Self::cstr(afrim_desktop_engine_get_candidate_annotation(self.0, index))
    }

    fn candidates(&self) -> Vec<&str> {
        (0..afrim_desktop_engine_get_candidate_count(self.0))
            .map(|i| self.candidate(i))
            .collect()
    }
}

impl Drop for TestEngine {
    fn drop(&mut self) {
        afrim_desktop_engine_free(self.0);
    }
}

#[test]
fn test_engine_lifecycle() {
    let engine = TestEngine::new();
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 1);
}

#[test]
fn test_null_pointers_are_safe() {
    let null = ptr::null_mut();
    assert_eq!(afrim_desktop_engine_init(null), -1);
    assert_eq!(afrim_desktop_engine_process_key(null, 0x61, 0, 0), 0);
    assert_eq!(afrim_desktop_engine_select_candidate(null, 0), 0);
    assert_eq!(afrim_desktop_engine_has_preedit(null), 0);
    assert_eq!(afrim_desktop_engine_has_commit(null), 0);
    assert_eq!(afrim_desktop_engine_commit(null), 0);
    assert!(afrim_desktop_engine_get_preedit(null).is_null());
    assert!(afrim_desktop_engine_get_candidate(null, 0).is_null());
    afrim_desktop_engine_reset(null);
    afrim_desktop_engine_save_learning(null);
    afrim_desktop_engine_set_surrounding_text(null, ptr::null(), 0);
    afrim_desktop_engine_free(null);
}

#[test]
fn test_compose_and_preedit() {
    let engine = TestEngine::new();
    engine.type_str("uu");
    assert_eq!(afrim_desktop_engine_has_preedit(engine.ptr()), 1);
    assert_eq!(engine.preedit(), "ʉ");
    assert_eq!(
        afrim_desktop_engine_get_preedit_len(engine.ptr()) as usize,
        "ʉ".len()
    );
    assert_eq!(
        afrim_desktop_engine_get_preedit_caret(engine.ptr()) as usize,
        "ʉ".len()
    );
    assert_eq!(engine.aux(), "uu");
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 0);
}

#[test]
fn test_space_commits() {
    let engine = TestEngine::new();
    engine.type_str("uuaf3");
    assert!(engine.press(XKB_KEY_SPACE));
    assert_eq!(afrim_desktop_engine_has_commit(engine.ptr()), 1);
    assert_eq!(engine.commit_text(), "ʉ̄ɑ̄ ");
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 1);
}

#[test]
fn test_flags_cleared_between_keys() {
    let engine = TestEngine::new();
    engine.type_str("uu");
    engine.press(XKB_KEY_RETURN);
    assert_eq!(afrim_desktop_engine_has_commit(engine.ptr()), 1);
    // Next key must not report the stale commit
    engine.type_str("a");
    assert_eq!(afrim_desktop_engine_has_commit(engine.ptr()), 0);
}

#[test]
fn test_shift_produces_capitalized_code() {
    let engine = TestEngine::new();
    // Shift press alone passes through
    assert!(!engine.press(XKB_KEY_SHIFT_L));
    assert!(engine.press_with('A' as u32, SHIFT_MASK));
    engine.type_str("f");
    assert_eq!(engine.preedit(), "Ɑ");
}

#[test]
fn test_release_not_consumed() {
    let engine = TestEngine::new();
    assert!(!engine.release('a' as u32));
}

#[test]
fn test_backspace_and_escape() {
    let engine = TestEngine::new();
    engine.type_str("uu3");
    assert!(engine.press(XKB_KEY_BACKSPACE));
    assert_eq!(engine.preedit(), "ʉ");

    assert!(engine.press(XKB_KEY_ESCAPE));
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 1);
    assert_eq!(afrim_desktop_engine_has_commit(engine.ptr()), 0);
}

#[test]
fn test_candidates_and_select() {
    let engine = TestEngine::new();
    engine.type_str("ha");
    assert_eq!(afrim_desktop_engine_has_candidates(engine.ptr()), 1);
    assert_eq!(afrim_desktop_engine_should_hide_candidates(engine.ptr()), 0);

    let candidates = engine.candidates();
    let index = candidates.iter().position(|c| *c == "hello").unwrap() as u32;
    assert_eq!(engine.candidate_annotation(index), "~lo");

    assert_eq!(
        afrim_desktop_engine_select_candidate(engine.ptr(), index),
        1
    );
    assert_eq!(engine.commit_text(), "hello");
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 1);
}

#[test]
fn test_candidates_hidden_without_predictions() {
    let engine = TestEngine::new();
    engine.type_str("uu");
    assert_eq!(afrim_desktop_engine_has_candidates(engine.ptr()), 1);
    assert_eq!(afrim_desktop_engine_should_hide_candidates(engine.ptr()), 1);
}

#[test]
fn test_commit_on_focus_loss() {
    let engine = TestEngine::new();
    assert_eq!(afrim_desktop_engine_commit(engine.ptr()), 0);
    engine.type_str("c_");
    assert_eq!(afrim_desktop_engine_commit(engine.ptr()), 1);
    assert_eq!(engine.commit_text(), "ç");
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 1);
}

#[test]
fn test_reset_clears_caches() {
    let engine = TestEngine::new();
    engine.type_str("uu");
    afrim_desktop_engine_reset(engine.ptr());
    assert_eq!(afrim_desktop_engine_is_empty(engine.ptr()), 1);
    assert_eq!(engine.preedit(), "");
    assert_eq!(afrim_desktop_engine_get_candidate_count(engine.ptr()), 0);
}
