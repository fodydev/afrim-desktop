//! Engine bridge: wraps afrim-desktop-im's InputMethodEngine for TSF integration.
//!
//! This module provides a higher-level wrapper that handles initialization
//! and translates between Windows key events and the engine's KeyEvent type.

use anyhow::Result;
use tracing::debug;

use afrim_desktop_im::config::Settings;
use afrim_desktop_im::{EngineResult, InputMethodEngine, KeyEvent};

use crate::keymap;

/// Configuration for the engine bridge, derived from afrim-desktop-im Settings.
pub struct BridgeConfig {
    pub settings: Settings,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        // Ensure a default config file exists so users can discover and edit it
        Settings::ensure_default_config();

        Self {
            settings: Settings::load().unwrap_or_default(),
        }
    }
}

/// Bridge between Windows TSF and afrim-desktop-im's InputMethodEngine.
///
/// Uses InputMethodEngine directly (Rust-to-Rust, no FFI overhead).
pub struct EngineBridge {
    engine: InputMethodEngine,
    settings: Settings,
    initialized: bool,
}

impl EngineBridge {
    /// Create a new engine bridge with default settings.
    pub fn new() -> Self {
        let config = BridgeConfig::default();
        Self::with_config(config)
    }

    /// Create a new engine bridge with explicit configuration.
    pub fn with_config(config: BridgeConfig) -> Self {
        Self {
            engine: InputMethodEngine::new(),
            settings: config.settings,
            initialized: false,
        }
    }

    /// Load the afrim dataset selected by the settings.
    ///
    /// This should be called during TSF ActivateEx. On failure the built-in
    /// sample dataset stays active, so typing keeps working.
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        debug!("EngineBridge: loading afrim config");
        // Mark as initialized even on failure so a broken config is not
        // re-read on every activation.
        self.initialized = true;
        self.engine.init_afrim(&self.settings)?;
        debug!("EngineBridge: initialization complete");
        Ok(())
    }

    /// Process a Windows key event through the engine.
    ///
    /// Converts Windows VK code + modifiers to afrim-desktop KeyEvent, then delegates
    /// to InputMethodEngine::process_key.
    #[allow(clippy::too_many_arguments)]
    pub fn process_key(
        &mut self,
        vk: u32,
        unicode_char: Option<char>,
        shift: bool,
        control: bool,
        alt: bool,
        win: bool,
        is_press: bool,
    ) -> EngineResult {
        let key_event =
            keymap::create_key_event(vk, unicode_char, shift, control, alt, win, is_press);
        self.engine.process_key(&key_event)
    }

    /// Determine whether a key would be consumed without changing engine state.
    ///
    /// Used by TSF's `OnTestKeyDown` for side-effect-free key filtering.
    #[allow(clippy::too_many_arguments)]
    pub fn would_consume_key(
        &self,
        vk: u32,
        unicode_char: Option<char>,
        shift: bool,
        control: bool,
        alt: bool,
        win: bool,
        is_press: bool,
    ) -> bool {
        let key_event =
            keymap::create_key_event(vk, unicode_char, shift, control, alt, win, is_press);
        self.engine.would_consume_key(&key_event)
    }

    /// Process a pre-built KeyEvent through the engine.
    pub fn process_key_event(&mut self, key: &KeyEvent) -> EngineResult {
        self.engine.process_key(key)
    }

    /// Commit any pending input text.
    pub fn commit(&mut self) -> String {
        self.engine.commit()
    }

    /// Commit the candidate at `index` (0-based) on the current page.
    pub fn select_candidate(&mut self, index: usize) -> EngineResult {
        self.engine.select_candidate(index)
    }

    /// Reset the engine state.
    pub fn reset(&mut self) {
        self.engine.reset();
    }

    /// Get the current input mode.
    pub fn input_mode(&self) -> afrim_desktop_im::InputMode {
        self.engine.input_mode()
    }

    /// Get access to the underlying engine actions for inspection.
    pub fn engine(&self) -> &InputMethodEngine {
        &self.engine
    }

    /// Check if the engine has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for EngineBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = EngineBridge::new();
        assert!(!bridge.is_initialized());
    }

    #[test]
    fn test_bridge_process_key_before_init() {
        let mut bridge = EngineBridge::new();
        // Should work even before initialization (built-in sample dataset)
        let result = bridge.process_key(0x41, Some('a'), false, false, false, false, true);
        // 'a' in Empty state should start composing
        assert!(result.consumed);
    }

    #[test]
    fn test_bridge_escape_resets() {
        let mut bridge = EngineBridge::new();
        // Type 'a' to enter composing state
        bridge.process_key(0x41, Some('a'), false, false, false, false, true);
        // Press Escape to reset
        let result = bridge.process_key(0x1B, None, false, false, false, false, true);
        assert!(result.consumed);
    }

    #[test]
    fn test_bridge_commit() {
        let mut bridge = EngineBridge::new();
        // Type some text
        bridge.process_key(0x41, Some('a'), false, false, false, false, true);
        // Commit
        let text = bridge.commit();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_bridge_reset_after_commit() {
        let mut bridge = EngineBridge::new();
        // Type 'a' to enter composing state
        bridge.process_key(0x41, Some('a'), false, false, false, false, true);
        // Commit + reset (simulates OnCompositionTerminated)
        let text = bridge.commit();
        assert!(!text.is_empty());
        bridge.reset();
        // After reset, typing 'a' again should start fresh composing
        let result = bridge.process_key(0x41, Some('a'), false, false, false, false, true);
        assert!(result.consumed);
    }
}
