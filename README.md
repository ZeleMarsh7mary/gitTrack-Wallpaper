# GitWallpaper

GitHub contribution wallpaper for Windows. One Rust binary, two modes (a
windowless tray daemon and a throw-away settings window), with the whole
compositor running on the CPU — no WebView, no `tokio`/`reqwest`, no GPU
overlay.

## Budgets that are actually held

| Budget | Value | How it is met |
|---|---|---|
| Idle RAM | < 10 MB | no GUI in the daemon, `SetProcessWorkingSetSize` after every render |
| Idle GPU | 0% | the wallpaper is written to a PNG and applied with `SystemParametersInfoW(SPI_SETDESKWALLPAPER)`; no window or overlay |
| Boot | does not fight login | the first fetch is delayed 12 s; the last wallpaper is already cached natively by Windows |
| Installer | no UAC | Inno Setup `PrivilegesRequired=lowest` into `%LOCALAPPDATA%\Programs\GitWallpaper`, autostart under HKCU |
| Token | never plaintext | Windows DPAPI (`CryptProtectData`) with app entropy, base64 inside `config.toml` |

## Build and run

```powershell
cargo build --release                         # target\release\git-wallpaper.exe
cargo test                                    # 8 tests, including the headless compositor
cargo run --release -- --render-once out.png  # render one wallpaper without touching the desktop
cargo run --release --                        # daemon + tray icon
cargo run --release -- --settings             # settings window (closing it returns all memory)
cargo run --release -- --apply-once           # one fetch + render + set-wallpaper cycle
```

Installer:

```powershell
iscc installer\setup.iss   # output: dist\GitWallpaper-0.1.0-setup.exe
```

Regenerate the icon with
`powershell -NoProfile -ExecutionPolicy Bypass -File assets\make-icon.ps1`
(16/32/48/256 px, no imaging library).

## Files and locations

| Path | Contents |
|---|---|
| `%APPDATA%\GitWallpaper\config.toml` | every setting; the token only as a DPAPI blob |
| `%LOCALAPPDATA%\GitWallpaper\cache.json` | per-day cache (`version`, `fetched_at`, `tz`, `days`), written atomically |
| `%LOCALAPPDATA%\GitWallpaper\current_wallpaper.png` | the most recent rendered wallpaper |
| `%LOCALAPPDATA%\GitWallpaper\daemon.log` | one-line log (failed fetch, render, PNG size) |

## Settings window

Two columns in the Windows 11 Settings style (WinUI 3 Fluent dark), 960 x 640:

| Zone | Width | Contents |
|---|---|---|
| Left | 220 px | navigation rail (36 px rows, 4 px radius) with an active `#2D2D2D` pill plus a 3 x 16 px accent bar on the leading edge; `Daemon: Running · Autostart: On · v0.1.0` pinned at the bottom |
| Right | remaining | 26 px page header, 320 x 180 desktop preview at the top left, then stacked `SettingsRow`s; the only scrolling column |
| Bottom | 56 px | sticky footer: status/toast on the left, `Test Connection` + `Save & Apply` on the right, 1 px hairline above |

Design tokens live in one place: background `#202020`, row `#2B2B2B` (hover
`#323232`), input `#222222`, hairline `Color32::from_white_alpha(14)`, accent
`#2EA043`, text `#F5F5F5`, muted `#9E9E9E`. Radii: 4 px for controls, 5 px for
rows, 8 px for the preview frame; secondary buttons are `#353535` with a
`#454545` border, control height 30 px.

The native caption is blended into the app background through
`DwmSetWindowAttribute` (`DWMWA_USE_IMMERSIVE_DARK_MODE`, `DWMWA_CAPTION_COLOR`,
`DWMWA_BORDER_COLOR`), called once when the window is created. If the OS refuses,
the default title bar stays and the app keeps running.

Typography: `C:\Windows\Fonts\segoeui.ttf` is inserted as the primary face
(falling back to egui's bundled font when the file is missing) and `consola.ttf`
covers numbers and hex values; icons come from `egui-phosphor` 0.7.3 (the only
release that matches egui 0.29) inserted at index 1 so text glyphs still come
from Segoe UI. All interface copy is English.

## Architecture at a glance

```
main.rs          argument router: daemon | --settings | --apply-once | --render-once
config.rs        config schema, defaults, clamping, atomic writes, paths
security.rs      DPAPI encrypt/decrypt (entropy "GitWallpaper/v1")
git_fetcher.rs   ureq: GraphQL profile + REST repo, offline cache, 53x7 grid
renderer/        Scene -> Pixmap: background | material (3-pass blur + tint) | 53x7 heatmap
win_api.rs       SPI_SETDESKWALLPAPER, EnumDisplaySettingsW, mutex, named event, trim, HKCU Run
daemon.rs        tray (tray-icon) + message loop + worker thread + 12 s boot delay
ui_settings.rs   eframe/egui: two-column WinUI 3 shell (220 nav / content / 56 footer), token theme, Phosphor, preview
```

Cross-process IPC: the auto-reset named event `Local\GitWallpaperRefreshEvent`.
`--settings` writes `config.toml` and calls `SetEvent`; the daemon reloads the
config and re-renders. A single instance is enforced by the mutex
`Local\GitWallpaperDaemonMutex`.

## Checks already run

- `cargo check --all-targets` clean.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo test` — 8 tests pass (DPAPI round trip, repository URL parsing, 53x7
  grid, opaque wallpaper with gradient, heatmap cells drawn, material changes
  pixels).
- `--render-once` produces a 1920x1080 PNG matching the monitor resolution.

Idle footprint numbers and the installer build status are recorded in
`AGENT_RESULT.md`.
