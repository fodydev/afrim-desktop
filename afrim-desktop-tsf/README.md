# afrim-desktop-tsf

The Windows frontend of Afrim Desktop: a TSF (Text Services Framework) text
service DLL that uses the shared `afrim-desktop-im` engine directly from Rust.

## Build

### Prerequisites

- Rust stable toolchain
- Windows 10/11

### Native build (Windows MSVC)

```powershell
cargo build -p afrim-desktop-tsf --release
```

### Cross-compile from Linux

```bash
# Requires the mingw-w64 toolchain
cargo build -p afrim-desktop-tsf --release --target x86_64-pc-windows-gnu
```

### Installer

The release workflow builds an NSIS installer from
`installer/afrim-desktop-installer.nsi`. To install by hand, register the DLL
from an administrator command prompt:

```powershell
regsvr32 target\release\afrim_desktop_tsf.dll
```

Unregister:

```powershell
regsvr32 /u target\release\afrim_desktop_tsf.dll
```

After registration, **Afrim Desktop** appears under the English (United
States) language in *Settings > Time & language > Language & region*. afrim
datasets cover many languages that have no Windows language ID of their own,
so the profile is registered under English.

## Usage

| Key | Action |
|-----|--------|
| Hankaku/Zenkaku, Ctrl+Space | Turn the IME on/off |
| Ctrl+Shift+Space | Toggle afrim / direct input |

The language bar shows `ɔ` (afrim), `A` (direct input) or `×` (off). Other
keys are listed in the [root README](../README.md#keys).

## Configuration

Settings file: `%APPDATA%\fodydev\afrim-desktop\config\config.toml`. It is
created with the default content the first time the IME is activated. To choose
the afrim dataset, see [Choosing a dataset](../README.md#choosing-a-dataset).

## Known issues and workarounds

### Shortcuts and arrow keys while composing

On Linux and macOS, a shortcut (Ctrl+S, ...) or an arrow key pressed while
composing first commits the composition. TSF does not let the IME act on keys
it does not consume, so on Windows the key goes straight to the application,
and the text being composed stays as typed only once the application ends the
composition. If an application does not end it, press Space or Return first.

## License

MIT OR Apache-2.0
