//! Afrim core: wraps afrim's preprocessor (sequential codes) and translator
//! (dictionary / script predictions) behind a preedit-oriented API.
//!
//! The afrim preprocessor was designed for frontends that type directly into
//! the application: it emits `Delete`/`CommitText` commands relative to text
//! already on screen. An IME owns a preedit instead, so every command is
//! applied to the composed string and the application only sees the final
//! result on commit.

use std::path::Path;
use std::rc::Rc;

use afrim_config::{Config, FileSystem};
use afrim_preprocessor::{Command, Key, KeyboardEvent, NamedKey, Preprocessor, utils};
use afrim_translator::{Predicate, Translator};
use anyhow::{Context, Result};
use tracing::{debug, warn};

/// Built-in afrim dataset used when no configuration file is available
const SAMPLE_CONFIG_TOML: &str = include_str!("../../../config/afrim-sample.toml");

/// Default afrim values, mirroring the afrim reference frontend
const DEFAULT_BUFFER_SIZE: usize = 64;
const DEFAULT_MIN_CONFIDENCE: f64 = 0.7;

/// In-memory filesystem serving the embedded sample config
struct EmbeddedFileSystem;

impl FileSystem for EmbeddedFileSystem {
    fn read_to_string(&self, _filepath: &Path) -> Result<String, std::io::Error> {
        Ok(SAMPLE_CONFIG_TOML.to_owned())
    }
}

/// Preprocessor + translator pair built from an afrim configuration.
///
/// Not `Send`: afrim's memory is `Rc`-based. Every frontend drives the engine
/// from a single UI thread (fcitx5 event loop, TSF STA thread, IMKit main
/// thread), so this is fine.
pub(in crate::core) struct AfrimCore {
    preprocessor: Preprocessor,
    translator: Translator,
    auto_commit: bool,
    page_size: Option<usize>,
}

impl AfrimCore {
    /// An engine without any data: keys are composed and committed verbatim.
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            preprocessor: Preprocessor::new(Rc::new(utils::build_map(vec![])), DEFAULT_BUFFER_SIZE),
            translator: Translator::new(Default::default(), false, DEFAULT_MIN_CONFIDENCE),
            auto_commit: false,
            page_size: None,
        }
    }

    /// Load the afrim configuration at `path` (TOML, may reference other files).
    pub fn from_file(path: &Path) -> Result<Self> {
        let config = Config::from_file(path)
            .with_context(|| format!("failed to load afrim config {}", path.display()))?;
        Ok(Self::from_config(&config))
    }

    /// Load the sample dataset embedded in the binary.
    pub fn builtin_sample() -> Self {
        let config = Config::from_filesystem(Path::new("afrim-sample.toml"), &EmbeddedFileSystem)
            .expect("embedded afrim-sample.toml must be valid");
        Self::from_config(&config)
    }

    fn from_config(config: &Config) -> Self {
        let core = config.core.as_ref();
        let buffer_size = core
            .and_then(|c| c.buffer_size)
            .unwrap_or(DEFAULT_BUFFER_SIZE);
        let auto_commit = core.and_then(|c| c.auto_commit).unwrap_or(false);
        let min_confidence = core
            .and_then(|c| c.min_confidence)
            .unwrap_or(DEFAULT_MIN_CONFIDENCE);
        let page_size = core.and_then(|c| c.page_size).filter(|&n| n > 0);

        let data = config.extract_data();
        let memory = utils::build_map(
            data.iter()
                .map(|(code, text)| vec![code.as_str(), text.as_str()])
                .collect(),
        );
        let preprocessor = Preprocessor::new(Rc::new(memory), buffer_size);

        #[allow(unused_mut)]
        let mut translator =
            Translator::new(config.extract_translation(), auto_commit, min_confidence);
        #[cfg(feature = "rhai")]
        match config.extract_translators() {
            Ok(translators) => {
                for (name, ast) in translators {
                    translator.register(name, ast);
                }
            }
            // Non-fatal: keep the sequential codes and dictionary working
            Err(e) => warn!("Failed to load afrim translators: {e:#}"),
        }

        debug!(
            "afrim loaded: {} codes, buffer_size={}, auto_commit={}",
            data.len(),
            buffer_size,
            auto_commit
        );

        Self {
            preprocessor,
            translator,
            auto_commit,
            page_size,
        }
    }

    /// Whether predicates flagged `can_commit` should be committed immediately
    pub fn auto_commit(&self) -> bool {
        self.auto_commit
    }

    /// Candidate page size requested by the afrim config, if any
    pub fn page_size(&self) -> Option<usize> {
        self.page_size
    }

    /// Type `ch` at the end of `composed` and let afrim rewrite it.
    pub fn feed(&mut self, composed: &mut String, ch: char) {
        // Mirror a direct-typing frontend: the key lands first, then afrim's
        // commands delete and replace text (the key included).
        composed.push(ch);
        self.preprocessor.process(KeyboardEvent {
            key: Key::Character(ch.to_string()),
            ..Default::default()
        });
        self.apply_queue(composed);
    }

    /// Undo the last afrim step on `composed`.
    pub fn backspace(&mut self, composed: &mut String) {
        self.preprocessor.process(KeyboardEvent {
            key: Key::Named(NamedKey::Backspace),
            ..Default::default()
        });
        // A direct-typing frontend lets the physical backspace delete one
        // character; afrim's `Delete` commands already account for it.
        self.apply_queue(composed);
    }

    /// Replace everything afrim is tracking in `composed` with `text`.
    pub fn replace_with(&mut self, composed: &mut String, text: &str) {
        self.preprocessor.commit(text.to_owned());
        self.apply_queue(composed);
    }

    /// The afrim input code currently tracked (e.g. `"uuaf3"`)
    pub fn input(&self) -> String {
        self.preprocessor.get_input()
    }

    /// Translator predictions for the current input
    pub fn predicates(&self) -> Vec<Predicate> {
        self.translator.translate(&self.input())
    }

    /// Forget the tracked sequence (after a commit or cancel).
    pub fn reset(&mut self) {
        // A non-character key clears the preprocessor's cursor
        self.preprocessor.process(KeyboardEvent::default());
        self.preprocessor.clear_queue();
    }

    fn apply_queue(&mut self, composed: &mut String) {
        while let Some(command) = self.preprocessor.pop_queue() {
            match command {
                Command::CommitText(text) => composed.push_str(&text),
                Command::Delete(text) => {
                    for _ in 0..text.chars().count() {
                        if composed.pop().is_none() {
                            break;
                        }
                    }
                }
                // Only meaningful for frontends that inject OS key events
                Command::Pause | Command::Resume => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_str(core: &mut AfrimCore, input: &str) -> String {
        let mut composed = String::new();
        for ch in input.chars() {
            core.feed(&mut composed, ch);
        }
        composed
    }

    #[test]
    fn test_sequential_codes() {
        let mut core = AfrimCore::builtin_sample();
        assert_eq!(type_str(&mut core, "uu"), "ʉ");
        core.reset();
        assert_eq!(type_str(&mut core, "uuaf3"), "ʉ̄ɑ̄");
        core.reset();
        assert_eq!(type_str(&mut core, "c_"), "ç");
    }

    #[test]
    fn test_unmatched_text_is_kept() {
        let mut core = AfrimCore::builtin_sample();
        assert_eq!(type_str(&mut core, "mbuu"), "mbʉ");
        assert_eq!(core.input(), "mbuu");
    }

    #[test]
    fn test_auto_capitalize() {
        let mut core = AfrimCore::builtin_sample();
        assert_eq!(type_str(&mut core, "Af"), "Ɑ");
    }

    #[test]
    fn test_backspace_rolls_back() {
        let mut core = AfrimCore::builtin_sample();
        let mut composed = type_str(&mut core, "uu3");
        assert_eq!(composed, "ʉ̄");
        core.backspace(&mut composed);
        assert_eq!(composed, "ʉ");
        // Like every afrim frontend, undoing "uu" removes the whole sequence
        core.backspace(&mut composed);
        assert_eq!(composed, "");
    }

    #[test]
    fn test_replace_with() {
        let mut core = AfrimCore::builtin_sample();
        let mut composed = type_str(&mut core, "halo");
        core.replace_with(&mut composed, "hello");
        assert_eq!(composed, "hello");
    }

    #[test]
    fn test_translation_predicates() {
        let mut core = AfrimCore::builtin_sample();
        type_str(&mut core, "ha");
        let predicates = core.predicates();
        assert!(
            predicates
                .iter()
                .any(|p| p.code == "halo" && p.texts == ["hello"])
        );
    }

    #[test]
    fn test_empty_core_is_verbatim() {
        let mut core = AfrimCore::empty();
        assert_eq!(type_str(&mut core, "uu3"), "uu3");
        assert!(core.predicates().is_empty());
    }

    #[test]
    fn test_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data.toml");
        std::fs::write(&data, "[data]\n\"n*\" = \"ŋ\"\n").unwrap();
        let config = dir.path().join("config.toml");
        std::fs::write(&config, "[data]\nlang = { path = \"./data.toml\" }\n").unwrap();

        let mut core = AfrimCore::from_file(&config).unwrap();
        assert_eq!(type_str(&mut core, "n*"), "ŋ");
    }

    #[test]
    fn test_from_missing_file_fails() {
        assert!(AfrimCore::from_file(Path::new("/nonexistent/afrim.toml")).is_err());
    }
}
