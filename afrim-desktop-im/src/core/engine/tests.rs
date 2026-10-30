use super::*;
use crate::core::keycode::KeyModifiers;

fn press(engine: &mut InputMethodEngine, keysym: Keysym) -> EngineResult {
    engine.process_key(&KeyEvent::press(keysym))
}

fn type_str(engine: &mut InputMethodEngine, input: &str) {
    for ch in input.chars() {
        let result = press(engine, Keysym(ch as u32));
        assert!(result.consumed, "'{ch}' should be consumed");
    }
}

fn preedit_text(engine: &InputMethodEngine) -> String {
    engine
        .preedit()
        .map(|p| p.text().to_string())
        .unwrap_or_default()
}

fn committed(result: &EngineResult) -> Option<&str> {
    result.actions.iter().find_map(|a| match a {
        EngineAction::Commit(text) => Some(text.as_str()),
        _ => None,
    })
}

fn ctrl_shift_space() -> KeyEvent {
    KeyEvent::new(
        Keysym::SPACE,
        KeyModifiers::new().with_control(true).with_shift(true),
        true,
    )
}

#[test]
fn test_initial_state() {
    let engine = InputMethodEngine::new();
    assert!(engine.state().is_empty());
    assert_eq!(engine.input_mode(), InputMode::Afrim);
    assert!(engine.candidates().is_none());
}

#[test]
fn test_compose_sequential_code() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uuaf3");
    assert_eq!(preedit_text(&engine), "ʉ̄ɑ̄");
    assert!(matches!(
        engine.state(),
        InputState::Composing { code, .. } if code == "uuaf3"
    ));
}

#[test]
fn test_aux_text_shows_code() {
    let mut engine = InputMethodEngine::new();
    let result = press(&mut engine, Keysym('u' as u32));
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::UpdateAuxText(code) if code == "u"))
    );
}

#[test]
fn test_space_commits_with_space() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "c_");
    let result = press(&mut engine, Keysym::SPACE);
    assert!(result.consumed);
    assert_eq!(committed(&result), Some("ç "));
    assert!(engine.state().is_empty());
}

#[test]
fn test_return_commits_preedit() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "mbuu");
    let result = press(&mut engine, Keysym::RETURN);
    assert!(result.consumed);
    assert_eq!(committed(&result), Some("mbʉ"));
    assert!(engine.state().is_empty());
}

#[test]
fn test_commit_action_order() {
    // Frontends (TSF edit session) expect the preedit to be cleared before the commit
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu");
    let result = press(&mut engine, Keysym::RETURN);
    assert!(matches!(result.actions.first(), Some(EngineAction::UpdatePreedit(p)) if p.is_empty()));
    assert!(matches!(result.actions.last(), Some(EngineAction::Commit(t)) if t == "ʉ"));
}

#[test]
fn test_new_word_after_commit() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu");
    press(&mut engine, Keysym::SPACE);
    type_str(&mut engine, "af");
    assert_eq!(preedit_text(&engine), "ɑ");
}

#[test]
fn test_escape_cancels() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu");
    let result = press(&mut engine, Keysym::ESCAPE);
    assert!(result.consumed);
    assert_eq!(committed(&result), None);
    assert!(engine.state().is_empty());
}

#[test]
fn test_backspace_rolls_back_then_empties() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu3");
    assert_eq!(preedit_text(&engine), "ʉ̄");

    press(&mut engine, Keysym::BACKSPACE);
    assert_eq!(preedit_text(&engine), "ʉ");

    // Undoing "uu" removes the whole sequence, which ends the composition
    let result = press(&mut engine, Keysym::BACKSPACE);
    assert!(result.consumed);
    assert!(engine.state().is_empty());
}

#[test]
fn test_empty_state_passthrough() {
    let mut engine = InputMethodEngine::new();
    assert!(!press(&mut engine, Keysym::BACKSPACE).consumed);
    assert!(!press(&mut engine, Keysym::SPACE).consumed);
    assert!(!press(&mut engine, Keysym::RETURN).consumed);
    assert!(!press(&mut engine, Keysym::LEFT).consumed);
    let ctrl_c = KeyEvent::new(
        Keysym('c' as u32),
        KeyModifiers::new().with_control(true),
        true,
    );
    assert!(!engine.process_key(&ctrl_c).consumed);
}

#[test]
fn test_releases_and_modifiers_pass_through() {
    let mut engine = InputMethodEngine::new();
    let release = KeyEvent::new(Keysym('a' as u32), KeyModifiers::new(), false);
    assert!(!engine.process_key(&release).consumed);
    assert!(!press(&mut engine, Keysym::SHIFT_L).consumed);
    assert!(engine.state().is_empty());
}

#[test]
fn test_shortcut_commits_and_passes_through() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu");
    let ctrl_s = KeyEvent::new(
        Keysym('s' as u32),
        KeyModifiers::new().with_control(true),
        true,
    );
    let result = engine.process_key(&ctrl_s);
    assert!(!result.consumed);
    assert_eq!(committed(&result), Some("ʉ"));
    assert!(engine.state().is_empty());
}

#[test]
fn test_arrow_without_candidates_commits_and_passes_through() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu");
    let result = press(&mut engine, Keysym::LEFT);
    assert!(!result.consumed);
    assert_eq!(committed(&result), Some("ʉ"));
}

#[test]
fn test_translation_candidates() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    let candidates = engine.candidates().expect("candidates for 'ha'");
    let halo = candidates
        .candidates()
        .iter()
        .find(|c| c.text == "hello")
        .expect("'halo' → 'hello'");
    assert_eq!(halo.annotation.as_deref(), Some("~lo"));
    assert_eq!(halo.reading.as_deref(), Some("halo"));
}

#[test]
fn test_enter_without_selection_keeps_typed_text() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    let result = press(&mut engine, Keysym::RETURN);
    assert_eq!(committed(&result), Some("ha"));
}

#[test]
fn test_tab_selects_candidate_then_enter_commits_it() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    let result = press(&mut engine, Keysym::TAB);
    assert!(result.consumed);
    assert_eq!(preedit_text(&engine), "hello");

    let result = press(&mut engine, Keysym::RETURN);
    assert_eq!(committed(&result), Some("hello"));
}

#[test]
fn test_typing_after_selection_returns_to_composition() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    press(&mut engine, Keysym::TAB);
    type_str(&mut engine, "l");
    assert_eq!(preedit_text(&engine), "hal");
}

#[test]
fn test_select_candidate_by_index() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    let index = engine
        .candidates()
        .unwrap()
        .page_candidates()
        .iter()
        .position(|c| c.text == "hello")
        .unwrap();
    let result = engine.select_candidate(index);
    assert!(result.consumed);
    assert_eq!(committed(&result), Some("hello"));
    assert!(engine.state().is_empty());
}

#[test]
fn test_select_candidate_out_of_range() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    assert!(!engine.select_candidate(100).consumed);
    assert!(!engine.state().is_empty());
}

#[test]
fn test_digits_feed_afrim_not_selection() {
    // Digits are tone markers in afrim codes, they must never select candidates
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "ha");
    assert!(engine.candidates().is_some());
    type_str(&mut engine, "1");
    assert_eq!(preedit_text(&engine), "ha1");
}

#[test]
fn test_toggle_direct_mode() {
    let mut engine = InputMethodEngine::new();
    type_str(&mut engine, "uu");

    let result = engine.process_key(&ctrl_shift_space());
    assert!(result.consumed);
    assert_eq!(committed(&result), Some("ʉ"));
    assert_eq!(engine.input_mode(), InputMode::Direct);

    // Direct mode: nothing is consumed
    assert!(!press(&mut engine, Keysym('u' as u32)).consumed);
    assert!(engine.state().is_empty());

    engine.process_key(&ctrl_shift_space());
    assert_eq!(engine.input_mode(), InputMode::Afrim);
    assert!(press(&mut engine, Keysym('u' as u32)).consumed);
}

#[test]
fn test_would_consume_matches_process_key() {
    let keys = [
        KeyEvent::press(Keysym('u' as u32)),
        KeyEvent::press(Keysym('3' as u32)),
        KeyEvent::press(Keysym::SPACE),
        KeyEvent::press(Keysym::RETURN),
        KeyEvent::press(Keysym::BACKSPACE),
        KeyEvent::press(Keysym::ESCAPE),
        KeyEvent::press(Keysym::TAB),
        KeyEvent::press(Keysym::LEFT),
        KeyEvent::press(Keysym::SHIFT_L),
        KeyEvent::new(
            Keysym('c' as u32),
            KeyModifiers::new().with_control(true),
            true,
        ),
        ctrl_shift_space(),
    ];
    for prefix in ["", "u", "ha"] {
        for key in &keys {
            let mut engine = InputMethodEngine::new();
            type_str(&mut engine, prefix);
            let predicted = engine.would_consume_key(key);
            let actual = engine.process_key(key).consumed;
            assert_eq!(predicted, actual, "prefix={prefix:?} key={key:?}");
        }
    }
}

#[test]
fn test_commit_api_returns_and_resets() {
    let mut engine = InputMethodEngine::new();
    assert_eq!(engine.commit(), "");
    type_str(&mut engine, "uu");
    assert_eq!(engine.commit(), "ʉ");
    assert!(engine.state().is_empty());
}

#[test]
fn test_load_afrim_config() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, "[data]\n\"n*\" = \"ŋ\"\n").unwrap();

    let mut engine = InputMethodEngine::new();
    engine.load_afrim_config(&config).unwrap();
    type_str(&mut engine, "n*");
    assert_eq!(preedit_text(&engine), "ŋ");
}

#[test]
fn test_failed_load_keeps_previous_dataset() {
    let mut engine = InputMethodEngine::new();
    assert!(
        engine
            .load_afrim_config(Path::new("/nonexistent/config.toml"))
            .is_err()
    );
    type_str(&mut engine, "uu");
    assert_eq!(preedit_text(&engine), "ʉ");
}

#[test]
fn test_auto_commit() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(
        &config,
        "[core]\nauto_commit = true\n\n[translation]\nhalo = \"hello\"\n",
    )
    .unwrap();

    let mut engine = InputMethodEngine::new();
    engine.load_afrim_config(&config).unwrap();
    type_str(&mut engine, "hal");
    let result = press(&mut engine, Keysym('o' as u32));
    assert_eq!(committed(&result), Some("hello"));
    assert!(engine.state().is_empty());
}

#[test]
fn test_page_size_from_afrim_config() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(
        &config,
        "[core]\npage_size = 2\n\n[translation]\nhaa = \"a\"\nhab = \"b\"\nhac = \"c\"\n",
    )
    .unwrap();

    let mut engine = InputMethodEngine::new();
    engine.load_afrim_config(&config).unwrap();
    type_str(&mut engine, "ha");
    let candidates = engine.candidates().unwrap();
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates.page_candidates().len(), 2);
}
