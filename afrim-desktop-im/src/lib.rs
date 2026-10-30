//! afrim-desktop-im: A cross-platform input method engine core powered by afrim
//!
//! This crate drives the [afrim](https://github.com/fodydev/afrim) input
//! method engine (sequential codes + translator) and exposes it as an IME
//! state machine shared by the fcitx5 (Linux), TSF (Windows) and IMKit (macOS)
//! frontends.

pub mod config;
pub mod core;
#[cfg(not(target_os = "windows"))]
pub mod ffi;

pub use core::engine::{EngineAction, EngineResult, InputMethodEngine, InputMode};
pub use core::keycode::{KeyEvent, KeyModifiers, Keysym};
pub use core::state::InputState;
