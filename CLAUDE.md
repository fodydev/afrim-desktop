# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

afrim-desktop is a cross-platform input method built on the [afrim](https://github.com/fodydev/afrim) engine (`afrim-config`, `afrim-preprocessor`, `afrim-translator` from crates.io). It is a fork of the karukan Japanese IME whose neural engine was replaced by afrim. Rust crates:

- **afrim-desktop-im**: Shared IME engine core — afrim state machine, preedit, candidate list, key handling, settings, C FFI, and the fcitx5 addon (Linux)
- **afrim-desktop-tsf**: Windows TSF input method (COM DLL) using afrim-desktop-im directly
- **afrim-desktop-macos**: macOS IMKit input method (Swift app + Rust staticlib) using afrim-desktop-im

## Build and Development Commands

All commands run from the repository root (Cargo workspace).

```bash
cargo build --release
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### afrim-desktop-im (Linux / fcitx5)

```bash
cargo test -p afrim-desktop-im

cd afrim-desktop-im/fcitx5-addon
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr     # or $HOME/.local (needs FCITX_ADDON_DIRS)
cmake --build build -j
sudo cmake --install build
```

### afrim-desktop-tsf (Windows)

```bash
cargo build -p afrim-desktop-tsf --release                                   # native MSVC
cargo build -p afrim-desktop-tsf --release --target x86_64-pc-windows-gnu    # cross (mingw-w64)
cargo check -p afrim-desktop-tsf --target x86_64-pc-windows-gnu --all-targets  # type-check from Linux, no linker needed
```

### afrim-desktop-macos

```bash
cd afrim-desktop-macos
scripts/build.sh     # cargo build + swiftc + .app bundle
scripts/install.sh   # copy to ~/Library/Input Methods/
# From Linux: cargo check -p afrim-desktop-macos --target aarch64-apple-darwin (ffi.rs is macOS-only)
```

## Architecture

### afrim-desktop-im (`afrim-desktop-im/src/`)

- `core/engine/mod.rs` — `InputMethodEngine`: Empty → Composing → Empty; `classify()` maps a key to a `KeyAction` without side effects, used by both `would_consume_key` (TSF `OnTestKeyDown`) and `process_key` so they always agree
- `core/engine/afrim.rs` — `AfrimCore`: wraps afrim's `Preprocessor` + `Translator`, loads configs (file or embedded sample via afrim's `FileSystem` trait)
- `core/engine/types.rs` — `EngineAction`, `EngineResult`, `EngineConfig`, `InputMode` (Afrim / Direct), `KeyAction`
- `core/engine/tests.rs` — engine tests
- `core/preedit.rs`, `core/candidate.rs`, `core/keycode.rs` (XKB keysyms), `core/state.rs`
- `config/settings.rs` — Settings (`[afrim] config_path`) and afrim config path resolution
- `ffi/` — C ABI for the fcitx5 addon (header: `include/afrim_desktop.h`)
- `fcitx5-addon/src/afrim-desktop.cpp` — C++ fcitx5 wrapper
- `config/default.toml` — default settings; `config/afrim-sample.toml` — built-in afrim dataset

### afrim-desktop-tsf (`afrim-desktop-tsf/src/`)

- `engine_bridge.rs` — wraps `InputMethodEngine` for TSF; `keymap.rs` — VK → keysym
- `tsf/` — TSF COM implementations (key event sink, edit session, composition sink, lang bar, ...)
- `globals.rs` — CLSID / profile / GUIDs, profile LANGID (en-US)
- `candidate/` — Direct2D candidate window

### afrim-desktop-macos (`afrim-desktop-macos/src/`)

- `engine_bridge.rs`, `keymap.rs` (Carbon key code → keysym), `ffi.rs` (C ABI for Swift, header `include/afrim_desktop_macos.h`)
- `swift/AfrimDesktopInputMethod/` — IMKit app (controller, candidates, settings view)

## Key Design Patterns

- afrim's preprocessor was built for frontends that type into the app: it emits `Delete(text)` / `CommitText(text)` commands. `AfrimCore` applies them to an in-memory `composed` string (push the typed key first, then delete `text.chars().count()` chars per `Delete`). `Pause`/`Resume` are ignored.
- Backspace rollback follows afrim: undoing `uu` (→ `ʉ`) removes the whole sequence, not one char.
- Digits and punctuation are part of afrim codes (`af3`, `c_`), so they are always fed to afrim and never select candidates. Candidate selection uses Tab/arrows + Return, or `select_candidate(index)` (0-based, exposed as `afrim_desktop_engine_select_candidate` / `afrim_desktop_macos_select_candidate`) for mouse clicks.
- Commit actions are emitted in the order `UpdatePreedit(empty)`, `HideCandidates`, `HideAuxText`, `Commit(text)` (the TSF edit session relies on it).
- afrim's memory is `Rc`-based (not `Send`); every frontend drives the engine from one UI thread.
- Afrim dataset lookup: `$AFRIM_CONFIG` → `[afrim] config_path` (relative to the config dir) → `<config dir>/afrim/config.toml` → embedded sample. A failed load keeps the previous dataset and returns an error to the frontend.
- The C ABIs keep `set_surrounding_text`, `save_learning` and `get_last_conversion_ms` as no-ops/aliases for compatibility with the C++/Swift callers.

## Documenting Known Issues

When a platform-specific known issue or workaround is identified, add it to the corresponding platform README's "Known issues and workarounds" section (e.g. `afrim-desktop-tsf/README.md`), creating the section if needed.
