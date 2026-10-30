//! afrim-desktop-macos: macOS Input Method Kit (IMKit) addon for afrim-desktop
//!
//! This crate provides a static library that bridges Swift (IMKit) and
//! afrim-desktop-im's InputMethodEngine via C FFI.

#[cfg(target_os = "macos")]
pub mod engine_bridge;
#[cfg(target_os = "macos")]
pub mod ffi;
#[cfg(target_os = "macos")]
pub mod keymap;

#[cfg(not(target_os = "macos"))]
pub mod engine_bridge;
#[cfg(not(target_os = "macos"))]
pub mod keymap;
