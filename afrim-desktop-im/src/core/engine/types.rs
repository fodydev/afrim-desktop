//! Type definitions for the IME engine

use super::super::candidate::CandidateList;
use super::super::preedit::Preedit;

/// Action to be performed by the framework/UI layer
#[derive(Debug, Clone)]
pub enum EngineAction {
    /// Update the preedit display
    UpdatePreedit(Preedit),
    /// Show the candidate window with candidates
    ShowCandidates(CandidateList),
    /// Hide the candidate window
    HideCandidates,
    /// Commit text to the application
    Commit(String),
    /// Update auxiliary text (e.g., the afrim input code being typed)
    UpdateAuxText(String),
    /// Hide auxiliary text
    HideAuxText,
}

/// Result of processing a key event
#[derive(Debug, Clone, Default)]
pub struct EngineResult {
    /// Whether the key was consumed by the IME
    pub consumed: bool,
    /// Actions to perform
    pub actions: Vec<EngineAction>,
}

impl EngineResult {
    pub fn consumed() -> Self {
        Self {
            consumed: true,
            actions: Vec::new(),
        }
    }

    pub fn not_consumed() -> Self {
        Self {
            consumed: false,
            actions: Vec::new(),
        }
    }

    pub fn with_action(mut self, action: EngineAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_actions(mut self, actions: impl IntoIterator<Item = EngineAction>) -> Self {
        self.actions.extend(actions);
        self
    }
}

/// Configuration for the IME engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Number of candidates per page (overridden by afrim's `core.page_size`)
    pub page_size: usize,
    /// Maximum number of candidates kept from the afrim translator
    pub max_candidates: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            page_size: CandidateList::DEFAULT_PAGE_SIZE,
            max_candidates: 50,
        }
    }
}

/// Input mode for the IME engine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Afrim mode (default) — keys are processed by the afrim engine
    Afrim,
    /// Direct input mode — keys bypass afrim and go straight to the application
    Direct,
}

/// What the engine will do with a key event.
///
/// Computed without side effects by `InputMethodEngine::classify`, so that
/// `would_consume_key` (TSF `OnTestKeyDown`) and `process_key` always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::core) enum KeyAction {
    /// Let the key through untouched
    PassThrough,
    /// Commit the composition, then let the key through
    CommitAndPassThrough,
    /// Toggle between afrim and direct input
    ToggleMode,
    /// Feed a character to the afrim preprocessor
    Feed(char),
    /// Undo the last afrim step
    Backspace,
    /// Discard the composition
    Cancel,
    /// Commit the composition (or selected candidate), optionally followed by a space
    Commit {
        append_space: bool,
    },
    /// Move the candidate cursor
    NextCandidate,
    PrevCandidate,
    NextPage,
    PrevPage,
}

impl KeyAction {
    /// Whether the key event is consumed by the IME
    pub fn consumes(self) -> bool {
        !matches!(self, Self::PassThrough | Self::CommitAndPassThrough)
    }
}
