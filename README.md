<div align="center">
  <img src="icon.png" width="128" alt="afrim-desktop" />
  <h1>Afrim Desktop</h1>
  <p>A cross-platform input method powered by <a href="https://github.com/fodydev/afrim">afrim</a> — fcitx5 (Linux) / TSF (Windows) / IMKit (macOS)</p>

  [![CI (im)](https://github.com/fodydev/afrim-desktop/actions/workflows/afrim-desktop-im-ci.yml/badge.svg)](https://github.com/fodydev/afrim-desktop/actions/workflows/afrim-desktop-im-ci.yml)
  [![CI (tsf)](https://github.com/fodydev/afrim-desktop/actions/workflows/afrim-desktop-tsf-ci.yml/badge.svg)](https://github.com/fodydev/afrim-desktop/actions/workflows/afrim-desktop-tsf-ci.yml)
  [![CI (macos)](https://github.com/fodydev/afrim-desktop/actions/workflows/afrim-desktop-macos-ci.yml/badge.svg)](https://github.com/fodydev/afrim-desktop/actions/workflows/afrim-desktop-macos-ci.yml)
  [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
</div>

Afrim Desktop is a native input method for the three desktop platforms, using
the [afrim](https://github.com/fodydev/afrim) engine. It types any language
that has an afrim dataset (sequential codes such as `uuaf3` → `ʉ̄ɑ̄`, plus
dictionary and script suggestions). One shared engine core runs behind a thin
native frontend for each platform.

## Project layout

| Crate | Platform | Description |
|-------|----------|-------------|
| [afrim-desktop-im](afrim-desktop-im/) | Linux (fcitx5) | Shared engine core (afrim state machine, preedit, candidates, keys) + C FFI + fcitx5 addon |
| [afrim-desktop-tsf](afrim-desktop-tsf/) | Windows | Text Services Framework input processor (COM DLL) |
| [afrim-desktop-macos](afrim-desktop-macos/) | macOS | Input Method Kit app (Swift) on top of a Rust static library |

```
            afrim-config ─┐
      afrim-preprocessor ─┼─► afrim-desktop-im (InputMethodEngine)
        afrim-translator ─┘        │
              ┌────────────────────┼─────────────────────┐
         C FFI (cdylib)       Rust (direct)          C FFI (staticlib)
              │                    │                     │
      fcitx5 addon (C++)   TSF text service (COM)   IMKit controller (Swift)
           Linux                Windows                  macOS
```

## Install

- **Linux (fcitx5)**: see [afrim-desktop-im](afrim-desktop-im/README.md#install)
- **Windows (TSF)**: see [afrim-desktop-tsf](afrim-desktop-tsf/README.md#build)
- **macOS (IMKit)**: see [afrim-desktop-macos](afrim-desktop-macos/README.md)

## Choosing a dataset

Afrim Desktop ships with a tiny built-in sample dataset so it works right after
installation. To type a real language, download a dataset from
[afrim-data](https://github.com/fodydev/afrim-data) and point Afrim Desktop at
its `.toml` file. The first match wins:

1. the `AFRIM_CONFIG` environment variable
2. `config_path` in the `[afrim]` section of Afrim Desktop's `config.toml`
3. `afrim/config.toml` next to Afrim Desktop's `config.toml`
4. the built-in sample

| Platform | Afrim Desktop `config.toml` |
|----------|-----------------------------|
| Linux | `~/.config/afrim-desktop/config.toml` |
| macOS | `~/Library/Application Support/org.fodydev.afrim-desktop/config.toml` |
| Windows | `%APPDATA%\fodydev\afrim-desktop\config\config.toml` |

```toml
[afrim]
config_path = "/home/me/afrim-data/gm/gm.toml"
```

Relative paths are resolved against the directory of `config.toml`. The dataset
format is documented in the [afrim configuration manual](https://fodydev.github.io/afrim-man/configuration/index.html).
Its `[core]` options (`buffer_size`, `auto_capitalize`, `auto_commit`,
`page_size`, `min_confidence`), `[translation]` dictionaries and `[translators]`
Rhai scripts are all supported.

## Keys

| Key | While composing |
|-----|-----------------|
| Letters, digits, punctuation | Fed to afrim (digits and punctuation are part of codes, e.g. `af3`, `c_`) |
| Space | Commit, followed by a space |
| Return | Commit (the highlighted suggestion if you moved to one, otherwise the typed text) |
| Tab / Shift+Tab, ↓ / ↑ | Move through suggestions |
| Page Down / Page Up | Next / previous page of suggestions |
| Backspace | Undo the last afrim step |
| Esc | Cancel the composition |
| Ctrl/Alt/Super shortcuts, arrows | Commit, then pass the key to the application |
| Ctrl+Shift+Space | Toggle between afrim and direct input (any time) |

Clicking a suggestion in the candidate window commits it (fcitx5, macOS).

## History

Afrim Desktop is a fork of [karukan](https://github.com/togatoga/karukan), a
Japanese input method by [@togatoga](https://github.com/togatoga) that already
ran natively on fcitx5, Windows TSF and macOS IMKit. Its neural kana-kanji
engine has been replaced by afrim, so the per-platform afrim frontends
(afrim-wish, fcitx5-afrim, ibus-afrim) can be replaced by this one project.
Many thanks to the karukan authors.

## License

Dual-licensed under MIT OR Apache-2.0:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

The afrim crates it depends on are licensed under the
[Mozilla Public License 2.0](https://github.com/fodydev/afrim/blob/main/LICENSE).
