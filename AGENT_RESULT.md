# AGENT_RESULT — GitWallpaper

Milestone status with the numbers and commands that were actually run on this
machine (Windows, Rust 1.100.0 stable MSVC, VS Build Tools 2022 + Windows SDK).

## Environment installed first

| Component | Status |
|---|---|
| rustup + stable-x86_64-pc-windows-msvc 1.100.0 | installed (`winget install Rustlang.Rustup`, then `rustup default stable`) |
| Visual Studio 2022 Build Tools, VCTools workload + Windows SDK | installed (`winget install Microsoft.VisualStudio.2022.BuildTools` through UAC) |
| Inno Setup (`iscc`) | **not installed** — the second UAC prompt was cancelled, so the installer has not been compiled |

## Code verification

```powershell
cargo check --all-targets                               # clean
cargo clippy --all-targets -- -D warnings               # clean
cargo test                                              # 8 tests pass
cargo build --release                                   # target\release\git-wallpaper.exe, 6.413.312 bytes
```

Tests:

- `security::tests::roundtrip`, `empty_is_empty` — real DPAPI encrypt/decrypt.
- `git_fetcher::tests::repo_url_shapes` — `https://`, `.git/`, `git@github.com:`.
- `git_fetcher::tests::grid_is_53x7_and_ends_with_today` — 371 slots, ending today.
- `git_fetcher::tests::levels_are_relative` — levels 0..4 relative to the maximum.
- `renderer::tests::renders_opaque_full_size_wallpaper` — 640x360, every alpha 255, top gradient != bottom.
- `renderer::tests::draws_heatmap_boxes_inside_the_card` — 12,040 level-4 pixels inside the grid area.
- `renderer::tests::card_material_changes_the_backdrop` — glass changes pixels.

## Real render

```
git-wallpaper.exe --render-once <path>   ->  render 1920x1080 -> 21,936 byte PNG (1.8 s in a debug build)
git-wallpaper.exe --render-once          ->  wallpaper set from %LOCALAPPDATA%\GitWallpaper\current_wallpaper.png
```

Pixel checks on the produced wallpaper (System.Drawing):
- gradient background: pixel (5,5) = 11,18,31 and (5,1074) = 17,37,28 → different.
- card area: 12,040 pixels exactly `#39d353` (level 4) → the 53x7 grid is drawn.
- every pixel is opaque (alpha 255).

## Idle footprint (release, no settings window)

Command: `Get-Process git-wallpaper` over 4 minutes, starting 25 s after launch.

| Metric | Result |
|---|---|
| Working set | 0.91 MB (25 s, right after the trim) → 2.19 MB (t+4 min) |
| Private bytes | 4.35 MB → 3.54 MB |
| Handles | 205 → 203 |
| Threads | 7 → 4 |
| Idle CPU (60 s) | 0.052 % of one core |
| GPU | no non-zero GPU engine counter for this pid |

Honest note: the working set *grew* 1.28 MB between the first sample and
t+4 min. That is page fault-in after `SetProcessWorkingSetSize` trimmed pages,
not a leak — private bytes over the same window fell 0.80 MB. 2.19 MB is still
well under the 10 MB budget.

## GitHub data

A real fetch was run against `https://github.com/rust-lang/rust` and GitHub
rejected it: `HTTP 403 rate limit exceeded` (no token; this machine's IP had
already used the 60/hour quota). The fallback path works: the reason is logged,
the cache is used when present, and the wallpaper is still rendered. For real
data, enter a personal access token in Settings (required for Profile mode,
recommended for Repository).

## Settings window (WinUI 3 redesign)

Measured from the process that was actually running (`--settings`, window
screenshot plus `GetWindowRect`/`GetClientRect`/`GetPixel`):

| Check | Result |
|---|---|
| Client area | **960 x 640** (requested 960x640); window 976x679 including the border |
| Maximized? | no — 976 px of a 1920 px screen |
| Caption bar | pixel (200,6) = `#202020` → `DwmSetWindowAttribute` (immersive dark + caption color) took effect |
| Left rail (220 px) | pixel (30,300) = `#202020` |
| Content background | pixel (300,635) = `#202020` |
| Settings rows | pixels (500,320..560) = `#2B2B2B`; text `#9B9B9B` at y=440 |
| Active navigation pill | 6,307 pixels of `#2D2D2D` in the rail, 98 accent pixels including the 3x16 px bar |
| Hero preview 320x180 | area x≈256-576, y≈120-260 holds wallpaper colors (`#295966`, `#37494E`) |
| Save & Apply button | pixel (930,630) = `#2EA043` |
| Memory while open | 90.0 MB working set |
| Closing the window | process disappears completely (`closed cleanly: True`) |

`cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings` and
`cargo test` (8 passing) were re-run after the redesign: clean, no warnings.
All interface copy is English, including the `git_fetcher` and `security` error
strings that surface in the toast.

## Not verified

- **The installer has not been compiled.** `installer/setup.iss` is complete
  (`PrivilegesRequired=lowest`, HKCU Run key, autostart task, clean uninstall)
  but `iscc` is missing on this machine. After
  `winget install JRSoftware.InnoSetup` (one UAC click), run
  `iscc installer\setup.iss` → `dist\GitWallpaper-0.1.0-setup.exe`. No claim of
  a proven one-click install is made in this report.
- Autostart has not been tested across a reboot; only the `HKCU\...\Run` value
  is written and read back (verified through the registry read-back in the UI).
- The tray menu (Refresh Now / Settings / Exit) was not clicked automatically.
  What is verified: the daemon process stays alive with its message loop, the
  tray icon was created (a `TrayIconBuilder` failure would end the process), and
  the worker renders on the 12 s schedule.
- Multi-monitor: only the primary display (`EnumDisplaySettingsW`) is used.
- The user's previous wallpaper was not recorded before this app ran;
  `HKCU\Control Panel\Desktop\Wallpaper` now points at
  `%LOCALAPPDATA%\GitWallpaper\current_wallpaper.png`.
