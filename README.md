# GitWallpaper

GitHub contribution wallpaper for Windows, built as a Cargo workspace with two
binaries:

| Binary | Role |
|---|---|
| `git-wallpaper.exe` | windowless tray daemon: fetch, CPU-composite, write the wallpaper. No GUI toolkit, no WebView, no async runtime. |
| `git-wallpaper-settings.exe` | on-demand settings window: Tauri v2 + WebView2 + React. Spawned only when the user opens Settings, dies completely when the window closes. |

Everything that renders pixels or talks to GitHub lives in `crates/core` and is
shared by both, so the settings preview and the desktop wallpaper cannot drift
apart.

## Budgets that are actually held

| Budget | Value | How it is met |
|---|---|---|
| Idle RAM | < 10 MB (daemon) | the daemon links no GUI: `tray-icon` + Win32 only, with `SetProcessWorkingSetSize` after every render |
| Idle GPU | 0% | the wallpaper is written to a PNG and applied with `SystemParametersInfoW(SPI_SETDESKWALLPAPER)`; no window or overlay |
| Settings cost | exits with the window | the WebView2 host is a separate process; closing the window returns all of it to Windows |
| Boot | does not fight login | the first fetch is delayed 12 s; the last wallpaper is already cached natively by Windows |
| Installer | no UAC | Inno Setup `PrivilegesRequired=lowest` into `%LOCALAPPDATA%\Programs\GitWallpaper`, autostart under HKCU |
| Token | never plaintext | Windows DPAPI (`CryptProtectData`) with app entropy, base64 inside `config.toml` |

## Build and run

The settings crate embeds `crates/settings-ui/ui/dist` at compile time, so the
frontend is built first:

```powershell
Set-Location crates\settings-ui\ui
& "C:\Program Files\nodejs\npm.cmd" install
& "C:\Program Files\nodejs\npm.cmd" run build      # -> ui\dist

Set-Location ..\..\..
cargo build --release                              # target\release\git-wallpaper.exe
                                                   # target\release\git-wallpaper-settings.exe
cargo test                                         # 8 tests, including the headless compositor
cargo run --release -- --render-once out.png       # render one wallpaper without touching the desktop
cargo run --release --                             # daemon + tray icon
cargo run --release -p git-wallpaper-settings      # settings window (closing it returns all memory)
cargo run --release -- --apply-once                # one fetch + render + set-wallpaper cycle
```

`git-wallpaper.exe --settings` still works: it forwards to
`git-wallpaper-settings.exe` next to it, so old shortcuts keep working.

Installer:

```powershell
iscc installer\setup.iss   # output: dist\GitWallpaper-0.1.0-setup.exe
```

Regenerate the daemon icon with
`powershell -NoProfile -ExecutionPolicy Bypass -File assets\make-icon.ps1`
(16/32/48/256 px, no imaging library), and the settings-window PNGs with
`powershell -NoProfile -ExecutionPolicy Bypass -File crates\settings-ui\icons\make-icons.ps1`.

## Files and locations

| Path | Contents |
|---|---|
| `%APPDATA%\GitWallpaper\config.toml` | every setting; the token only as a DPAPI blob |
| `%LOCALAPPDATA%\GitWallpaper\cache.json` | per-day cache (`version`, `fetched_at`, `tz`, `days`), written atomically |
| `%LOCALAPPDATA%\GitWallpaper\current_wallpaper.png` | the most recent rendered wallpaper |
| `%LOCALAPPDATA%\GitWallpaper\daemon.log` | one-line log (failed fetch, render, PNG size) |

## Settings window

Tauri v2 shell, 980 x 650 (minimum 820 x 540), centered, dark native caption.
The frontend is React 18 + TypeScript (strict) + Vite with hand-written CSS —
no component library — reproducing the Windows 11 Settings layout:

| Zone | Size | Contents |
|---|---|---|
| Left rail | 230 px | section header, four 40 px nav rows with inline SVG icons; the active row is a `rgba(255,255,255,0.06)` pill with a 3 x 16 px accent bar on the leading edge; status pills (`Daemon: running`, `Auto-start: on`) and the version pinned at the bottom |
| Content | remaining | 28 px page title, one-line description, the 480 x 270 live wallpaper render with `1920 x 1080 · live render` caption, then grouped setting rows (4 px gap, 5 px radius, `rgba(255,255,255,0.05)` surface, 1 px hairline) |
| Footer | 57 px | hairline above, last-action status on the left, `Test Connection` + accent `Save & Apply` on the right, both with spinner + disabled states |

Controls are built from scratch and keyboard-operable: segmented switches, text
and password fields (eye toggle, token never in a tooltip or toast), value-badge
sliders, selects, colour swatch + hex, the five-chip heatmap palette with a
2 px accent ring, a 3 x 3 card-alignment grid with optional custom offsets,
WinUI-style switches, and a bottom-right toast that auto-dismisses after 4 s.

Backdrop: `DWMWA_SYSTEMBACKDROP_TYPE = Mica` plus
`DWMWA_USE_IMMERSIVE_DARK_MODE`/`DWMWA_CAPTION_COLOR` via `DwmSetWindowAttribute`
(`crates/settings-ui/src/mica.rs`). The window is transparent, so the Mica
backdrop shows through; `show_window` returns whether the call succeeded and the
frontend falls back to a flat `#202020` window when it did not (Windows 10).

Design tokens live in one place (`ui/src/styles.css`): background `#202020`
(fallback only), row `#2B2B2B`, input `#121215`, accent `#2EA043`, text
`#F5F5F5`, muted `#9E9E9E`, secondary button `#353535` with a `#454545` border.
All interface copy is English.

### IPC contract

Typed commands in `crates/settings-ui/src/commands.rs`, called from
`ui/src/lib/ipc.ts` (which falls back to a demo backend outside Tauri so the
page can be reviewed in a plain browser):

| Command | Returns |
|---|---|
| `bootstrap` | `{ config, status, warning }` — DPAPI-decrypted token for editing, daemon/autostart state, monitor size, file paths |
| `render_preview({ config })` | PNG data URL rendered by `core::renderer` at the real monitor aspect ratio |
| `test_connection({ config })` | `{ total, bestDay, streak, days }`, also refreshes `cache.json` |
| `save_and_apply({ config })` | writes `config.toml` (token re-encrypted), updates the HKCU Run key, signals the daemon or starts it |
| `pick_image_file` | absolute path or `null` (`rfd` file dialog on a blocking thread) |
| `show_window` | applies Mica + dark caption, shows the window, returns whether Mica was applied |

Network fetches and preview renders run on `spawn_blocking`, so a slow GitHub
response never freezes the window.

## Architecture at a glance

```
crates/core        git-wallpaper-core (library, no GUI)
  config.rs        config schema, defaults, clamping, atomic writes, paths
  security.rs      DPAPI encrypt/decrypt (entropy "GitWallpaper/v1")
  git_fetcher.rs   ureq: GraphQL profile + REST repo, offline cache, 53x7 grid
  renderer/        Scene -> Pixmap: background | material (3-pass blur + tint) | 53x7 heatmap
  win_api.rs       SPI_SETDESKWALLPAPER, EnumDisplaySettingsW, mutex, named event, trim, HKCU Run

crates/daemon      git-wallpaper.exe
  main.rs          argument router: daemon | --settings (forward) | --apply-once | --render-once
  daemon.rs        tray (tray-icon) + message loop + worker thread + 12 s boot delay
  build.rs         icon + DPI manifest embedded through a generated .rc

crates/settings-ui git-wallpaper-settings.exe
  main.rs          Tauri builder + command registration
  commands.rs      the six IPC commands, every one delegating to core
  ui_config.rs     camelCase wire types <-> core::Config
  mica.rs          DWM system backdrop + dark caption
  ui/              React + TypeScript + Vite frontend (embedded in the exe)
```

Cross-process IPC: the auto-reset named event `Local\GitWallpaperRefreshEvent`.
The settings process writes `config.toml` and calls `SetEvent`; the daemon
reloads the config and re-renders. `--settings`/the tray item launch
`git-wallpaper-settings.exe`. A single daemon instance is enforced by the mutex
`Local\GitWallpaperDaemonMutex`.

## Checks already run

- `cargo check --release --all-targets` clean.
- `cargo clippy --release --all-targets` clean (no warnings).
- `cargo test` — 8 tests pass (DPAPI round trip, repository URL parsing, 53x7
  grid, opaque wallpaper with gradient, heatmap cells drawn, material changes
  pixels).
- `cargo build --release` — `git-wallpaper.exe` 3,146,752 bytes,
  `git-wallpaper-settings.exe` 5,817,344 bytes.
- `--render-once` produces a 1920x1080, 1,700,407-byte PNG — byte-identical in
  size to the pre-migration daemon output for the same config.
- `git-wallpaper-settings.exe` launches, reports window title `GitWallpaper
  Settings` (996 x 689 including the native caption), stays responsive, and
  `PrintWindow` captures a painted Mica-tinted frame (4,779 distinct sampled
  colours, 307 accent pixels, dominant backdrop `#1A2224` from the blurred
  desktop) rather than a blank surface.
- `iscc installer\setup.iss` compiles:
  `dist\GitWallpaper-0.1.0-setup.exe`, 4,456,479 bytes, both binaries compressed.
- Frontend `npm run build`: 41 modules, `dist` = 0.45 kB HTML + 16.24 kB CSS +
  176.02 kB JS (56.30 kB gzip).

Not verified here: a full install/uninstall cycle on a clean machine, and the
Mica result on a Windows 10 host (the flat `#202020` fallback path).
