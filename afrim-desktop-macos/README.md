# afrim-desktop-macos

The macOS frontend of Afrim Desktop: an Input Method Kit app written in Swift,
linked against a Rust static library that wraps the shared `afrim-desktop-im`
engine (`include/afrim_desktop_macos.h`).

## Requirements

- macOS 14.0 or later
- Rust toolchain
- Xcode Command Line Tools (for `swiftc`)

## Build

```bash
cd afrim-desktop-macos
scripts/build.sh
```

This produces `afrim-desktop-macos/build/AfrimDesktop.app`.

## Install

```bash
scripts/install.sh
```

This copies the app to `~/Library/Input Methods/AfrimDesktop.app`.

> **Important:** macOS only loads input methods from `~/Library/Input Methods/`
> and `/Library/Input Methods/`. It will not work from `/Applications/`.

## Enable it

1. Log out and log back in (or restart)
2. System Settings > Keyboard > Input Sources > Edit > +
3. Add **Afrim Desktop**, listed under English

To try it without logging out:

```bash
killall AfrimDesktopInputMethod 2>/dev/null
open ~/Library/Input\ Methods/AfrimDesktop.app
```

## Configuration

Settings file: `~/Library/Application Support/org.fodydev.afrim-desktop/config.toml`.
To choose the afrim dataset, see [Choosing a dataset](../README.md#choosing-a-dataset).
The dataset is loaded the first time the input method is activated in an
application; after editing the config, quit and reopen that application.

## Development

```bash
cargo test -p afrim-desktop-macos
```
