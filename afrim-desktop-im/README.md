# afrim-desktop-im

The shared engine core of Afrim Desktop, and its Linux frontend (fcitx5 addon).

- `src/core/engine/` — `InputMethodEngine`: drives afrim's preprocessor and
  translator, owns the preedit and the candidate list
- `src/ffi/` — C ABI used by the fcitx5 addon (`include/afrim_desktop.h`)
- `fcitx5-addon/` — the fcitx5 addon (C++)
- `config/default.toml` — default Afrim Desktop settings
- `config/afrim-sample.toml` — built-in afrim sample dataset

## Install

### Prerequisites

Debian/Ubuntu:

```bash
sudo apt install fcitx5 fcitx5-modules-dev libfcitx5core-dev \
    libfcitx5config-dev libfcitx5utils-dev extra-cmake-modules \
    cmake build-essential
```

Arch Linux: use the [`PKGBUILD`](../PKGBUILD) at the repository root (`makepkg -si`).

### System install

Installs to `/usr` (needs sudo, no `FCITX_ADDON_DIRS` needed):

```bash
cd afrim-desktop-im/fcitx5-addon
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr
cmake --build build -j
sudo cmake --install build
fcitx5 -r
```

### User-local install

Installs to `~/.local` (no sudo):

```bash
cd afrim-desktop-im/fcitx5-addon
cmake -B build -DCMAKE_INSTALL_PREFIX=$HOME/.local
cmake --build build -j
cmake --install build
```

fcitx5 then needs `FCITX_ADDON_DIRS` to include both the local and the system
addon directories. Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, ...):

```bash
export FCITX_ADDON_DIRS=$HOME/.local/lib/fcitx5:$(pkg-config --variable=libdir Fcitx5Core)/fcitx5
```

> [!WARNING]
> If `FCITX_ADDON_DIRS` only lists the local directory, fcitx5 can no longer
> find its own system addons (`Could not locate library libwayland.so for addon wayland`).

### Enable it

Run `fcitx5 -r -d` and check that the log contains `Loaded addon afrim-desktop`.
Then open **fcitx5-configtool**, search for "Afrim Desktop" under
*Available Input Method*, and add it to the left-hand list.

If the log says `Could not locate library afrim-desktop.so for addon afrim-desktop`,
the addon was installed outside fcitx5's addon directory (for example
`/usr/local/lib/fcitx5`). Reinstall with `-DCMAKE_INSTALL_PREFIX=/usr`, or add
that directory to `FCITX_ADDON_DIRS` as shown above.

## Configuration

Settings file: `~/.config/afrim-desktop/config.toml`. To choose the afrim
dataset, see [Choosing a dataset](../README.md#choosing-a-dataset).

The fcitx5 addon loads the dataset the first time you type. After editing the
config, restart fcitx5 (`fcitx5 -r`).

## Development

```bash
cargo test -p afrim-desktop-im
cargo clippy -p afrim-desktop-im --all-targets -- -D warnings
```

Features: `rhai` (afrim `[translators]` scripts) and `strsim` (fuzzy matching
of dictionary codes) are enabled by default.
