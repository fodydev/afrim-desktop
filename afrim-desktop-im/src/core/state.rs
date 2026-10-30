//! Input state machine
//!
//! Defines the states of the IME and transitions between them.

use super::preedit::Preedit;

/// The current state of the IME
#[derive(Debug, Clone, Default)]
pub enum InputState {
    /// No input, waiting for user to type
    #[default]
    Empty,

    /// Composing mode - afrim is rewriting the typed text in the preedit
    Composing {
        /// The preedit string being composed (afrim output)
        preedit: Preedit,
        /// The afrim input code being tracked (e.g. "uuaf3")
        code: String,
    },
}

impl InputState {
    /// Check if the engine is in the Empty (idle) state
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Get the current preedit if any
    pub fn preedit(&self) -> Option<&Preedit> {
        match self {
            Self::Empty => None,
            Self::Composing { preedit, .. } => Some(preedit),
        }
    }
}
