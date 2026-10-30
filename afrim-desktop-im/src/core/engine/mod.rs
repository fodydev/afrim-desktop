//! IME Engine - the core state machine and input processing
//!
//! This module contains the main `InputMethodEngine` struct that drives the
//! afrim engine and manages the IME state (Empty → Composing → Empty).

mod afrim;
mod types;

pub use types::*;

use afrim::AfrimCore;

#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::Result;
use tracing::{debug, info, trace};

use super::candidate::{Candidate, CandidateList};
use super::keycode::{KeyEvent, Keysym};
use super::preedit::{Preedit, PreeditSegment};
use super::state::InputState;
use crate::config::Settings;

/// The main IME engine
pub struct InputMethodEngine {
    /// Current input state
    state: InputState,
    /// Afrim preprocessor + translator
    afrim: AfrimCore,
    /// Engine configuration
    config: EngineConfig,
    /// Current input mode (Afrim or Direct)
    input_mode: InputMode,
    /// Composed text: the afrim output for the keys typed so far
    composed: String,
    /// Translator candidates for the current input
    candidates: CandidateList,
    /// Whether the user explicitly moved to a candidate (Tab/Up/Down)
    candidate_selected: bool,
}

impl InputMethodEngine {
    /// Create a new IME engine using the built-in sample dataset
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }

    /// Create with configuration
    pub fn with_config(config: EngineConfig) -> Self {
        Self {
            state: InputState::Empty,
            afrim: AfrimCore::builtin_sample(),
            candidates: CandidateList::new(Vec::new()),
            config,
            input_mode: InputMode::Afrim,
            composed: String::new(),
            candidate_selected: false,
        }
    }

    /// Load the afrim configuration selected by `settings`.
    ///
    /// Falls back to the built-in sample dataset when no configuration is
    /// found. On error the previously loaded dataset is kept, so typing still
    /// works, and the error is returned for the frontend to report.
    pub fn init_afrim(&mut self, settings: &Settings) -> Result<()> {
        match settings.afrim_config_path() {
            Some(path) => self.load_afrim_config(&path),
            None => {
                info!("No afrim config found, using the built-in sample dataset");
                self.reset();
                self.afrim = AfrimCore::builtin_sample();
                Ok(())
            }
        }
    }

    /// Load an afrim configuration file.
    pub fn load_afrim_config(&mut self, path: &Path) -> Result<()> {
        let afrim = AfrimCore::from_file(path)?;
        info!("Loaded afrim config from {}", path.display());
        self.reset();
        self.afrim = afrim;
        Ok(())
    }

    /// Get the current input mode
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Get the current state
    pub fn state(&self) -> &InputState {
        &self.state
    }

    /// Get the current preedit if any
    pub fn preedit(&self) -> Option<&Preedit> {
        self.state.preedit()
    }

    /// Get the current candidates, if any are shown
    pub fn candidates(&self) -> Option<&CandidateList> {
        (!self.candidates.is_empty()).then_some(&self.candidates)
    }

    /// Reset the engine state, discarding any composition
    pub fn reset(&mut self) {
        self.afrim.reset();
        self.composed.clear();
        self.candidates = CandidateList::new(Vec::new());
        self.candidate_selected = false;
        self.state = InputState::Empty;
    }

    /// Determine whether a key event would be consumed without changing engine state.
    ///
    /// This is used by TSF's `OnTestKeyDown` to report whether the key will be handled,
    /// without actually processing it. The real processing happens in `process_key`
    /// during `OnKeyDown`.
    pub fn would_consume_key(&self, key: &KeyEvent) -> bool {
        self.classify(key).consumes()
    }

    /// Process a key event
    pub fn process_key(&mut self, key: &KeyEvent) -> EngineResult {
        let action = self.classify(key);
        trace!("Processing key: {:?} → {:?}", key.keysym, action);

        match action {
            KeyAction::PassThrough => EngineResult::not_consumed(),
            KeyAction::CommitAndPassThrough => EngineResult {
                consumed: false,
                ..self.commit_composition("")
            },
            KeyAction::ToggleMode => self.toggle_mode(),
            KeyAction::Feed(ch) => self.feed(ch),
            KeyAction::Backspace => self.backspace(),
            KeyAction::Cancel => {
                self.reset();
                EngineResult::consumed().with_actions(Self::clear_ui_actions())
            }
            KeyAction::Commit { append_space } => {
                self.commit_composition(if append_space { " " } else { "" })
            }
            KeyAction::NextCandidate => self.move_candidate(CandidateList::move_next),
            KeyAction::PrevCandidate => self.move_candidate(CandidateList::move_prev),
            KeyAction::NextPage => self.move_candidate(CandidateList::next_page),
            KeyAction::PrevPage => self.move_candidate(CandidateList::prev_page),
        }
    }

    /// Commit the candidate at `page_index` (0-based) on the current candidate
    /// page, e.g. after a mouse click in the frontend's candidate window.
    pub fn select_candidate(&mut self, page_index: usize) -> EngineResult {
        if self.candidates.select_on_page(page_index + 1).is_none() {
            return EngineResult::not_consumed();
        }
        self.candidate_selected = true;
        self.commit_composition("")
    }

    /// Commit any pending input and return the text
    pub fn commit(&mut self) -> String {
        let text = self.composition_text();
        self.reset();
        text
    }

    /// Decide what to do with `key` (side-effect free).
    pub(in crate::core) fn classify(&self, key: &KeyEvent) -> KeyAction {
        if !key.is_press || key.keysym.is_modifier() {
            return KeyAction::PassThrough;
        }

        let mods = key.modifiers;

        // Ctrl+Shift+Space: toggle afrim / direct input (works in all states)
        if mods.control_key && mods.shift_key && key.keysym == Keysym::SPACE {
            return KeyAction::ToggleMode;
        }

        if self.input_mode == InputMode::Direct {
            return KeyAction::PassThrough;
        }

        let shortcut = mods.control_key || mods.alt_key || mods.super_key;

        if self.state.is_empty() {
            return match key.to_char() {
                Some(ch) if !shortcut && key.keysym != Keysym::SPACE => KeyAction::Feed(ch),
                _ => KeyAction::PassThrough,
            };
        }

        // Shortcuts (Ctrl+C, Alt+Tab, ...) end the composition
        if shortcut {
            return KeyAction::CommitAndPassThrough;
        }

        let has_candidates = !self.candidates.is_empty();

        match key.keysym {
            Keysym::ESCAPE => KeyAction::Cancel,
            Keysym::RETURN | Keysym::KP_ENTER => KeyAction::Commit {
                append_space: false,
            },
            Keysym::SPACE => KeyAction::Commit { append_space: true },
            Keysym::BACKSPACE => KeyAction::Backspace,
            Keysym::TAB if has_candidates && mods.shift_key => KeyAction::PrevCandidate,
            Keysym::TAB | Keysym::DOWN if has_candidates => KeyAction::NextCandidate,
            Keysym::UP if has_candidates => KeyAction::PrevCandidate,
            Keysym::PAGE_DOWN if has_candidates => KeyAction::NextPage,
            Keysym::PAGE_UP if has_candidates => KeyAction::PrevPage,
            _ => match key.to_char() {
                // Digits and punctuation are part of afrim codes (e.g. "af3", "c_")
                Some(ch) => KeyAction::Feed(ch),
                None => KeyAction::CommitAndPassThrough,
            },
        }
    }

    fn feed(&mut self, ch: char) -> EngineResult {
        self.afrim.feed(&mut self.composed, ch);
        self.candidate_selected = false;
        self.update_candidates();

        // Auto-commit: a translator flagged its prediction as final
        if self.afrim.auto_commit()
            && let Some(text) = self.auto_commit_text()
        {
            debug!("auto-commit: {}", text);
            self.afrim.replace_with(&mut self.composed, &text);
            return self.commit_composition("");
        }

        self.refresh()
    }

    fn backspace(&mut self) -> EngineResult {
        self.afrim.backspace(&mut self.composed);
        self.candidate_selected = false;

        if self.composed.is_empty() {
            self.reset();
            return EngineResult::consumed().with_actions(Self::clear_ui_actions());
        }

        self.update_candidates();
        self.refresh()
    }

    fn move_candidate(&mut self, step: fn(&mut CandidateList) -> bool) -> EngineResult {
        step(&mut self.candidates);
        self.candidate_selected = true;
        self.refresh()
    }

    fn toggle_mode(&mut self) -> EngineResult {
        // Leaving afrim mode commits whatever is being composed
        let result = if self.state.is_empty() {
            EngineResult::consumed()
        } else {
            self.commit_composition("")
        };
        self.input_mode = match self.input_mode {
            InputMode::Afrim => InputMode::Direct,
            InputMode::Direct => InputMode::Afrim,
        };
        debug!("input mode: {:?}", self.input_mode);
        result
    }

    /// Commit the composition (or the selected candidate) followed by `suffix`.
    fn commit_composition(&mut self, suffix: &str) -> EngineResult {
        let mut text = self.commit();
        text.push_str(suffix);

        let result = EngineResult::consumed().with_actions(Self::clear_ui_actions());
        if text.is_empty() {
            result
        } else {
            result.with_action(EngineAction::Commit(text))
        }
    }

    /// The text that a commit would send to the application
    fn composition_text(&self) -> String {
        if self.candidate_selected
            && let Some(text) = self.candidates.selected_text()
        {
            return text.to_string();
        }
        self.composed.clone()
    }

    fn auto_commit_text(&self) -> Option<String> {
        self.afrim
            .predicates()
            .into_iter()
            .find(|p| p.can_commit)
            .and_then(|p| p.texts.into_iter().find(|t| !t.is_empty()))
    }

    /// Rebuild the candidate list from the translator predictions.
    fn update_candidates(&mut self) {
        let mut candidates: Vec<Candidate> = Vec::new();
        for predicate in self.afrim.predicates() {
            for text in predicate.texts.into_iter().filter(|t| !t.is_empty()) {
                if candidates.len() >= self.config.max_candidates {
                    break;
                }
                if candidates.iter().any(|c| c.text == text) {
                    continue;
                }
                let mut candidate =
                    Candidate::with_reading(text, &predicate.code).with_index(candidates.len());
                // Show the rest of the code to type, e.g. "~lo" for "ha" → "halo"
                candidate.annotation = (!predicate.remaining_code.is_empty())
                    .then(|| format!("~{}", predicate.remaining_code));
                candidates.push(candidate);
            }
        }

        let page_size = self.afrim.page_size().unwrap_or(self.config.page_size);
        self.candidates = CandidateList::new(candidates).with_page_size(page_size);
    }

    /// Sync `state` with the composition and build the matching UI actions.
    fn refresh(&mut self) -> EngineResult {
        let code = self.afrim.input();

        let preedit = match self.candidates.selected_text() {
            // Preview the candidate the user moved to
            Some(text) if self.candidate_selected => Preedit::from_segments(
                vec![PreeditSegment::highlighted(text)],
                text.chars().count(),
            ),
            _ => Preedit::with_text_underlined(self.composed.as_str()),
        };

        self.state = InputState::Composing {
            preedit: preedit.clone(),
            code: code.clone(),
        };

        let mut result = EngineResult::consumed().with_action(EngineAction::UpdatePreedit(preedit));
        result = if self.candidates.is_empty() {
            result.with_action(EngineAction::HideCandidates)
        } else {
            result.with_action(EngineAction::ShowCandidates(self.candidates.clone()))
        };
        if code.is_empty() {
            result.with_action(EngineAction::HideAuxText)
        } else {
            result.with_action(EngineAction::UpdateAuxText(code))
        }
    }

    fn clear_ui_actions() -> [EngineAction; 3] {
        [
            EngineAction::UpdatePreedit(Preedit::new()),
            EngineAction::HideCandidates,
            EngineAction::HideAuxText,
        ]
    }
}

impl Default for InputMethodEngine {
    fn default() -> Self {
        Self::new()
    }
}
