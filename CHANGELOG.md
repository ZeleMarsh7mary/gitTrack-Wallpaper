# Changelog

All notable changes to GitTrack Wallpaper. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - unreleased

First public release. The project was previously developed privately as
"GitWallpaper"; this release renames it and opens the source under MIT.

### Added

- Tauri v2 settings window (`git-track-wallpaper-settings.exe`): React +
  TypeScript frontend, Windows 11 Settings layout with a 230 px navigation
  rail, live wallpaper preview, stacked setting rows, sticky action footer and
  toasts.
- Mica backdrop through `DWMWA_SYSTEMBACKDROP_TYPE`, with a flat `#202020`
  window on Windows 10.
- Typed IPC commands between the webview and Rust: `bootstrap`,
  `render_preview`, `test_connection`, `save_and_apply`, `pick_image_file`,
  `show_window`. Network and render work runs off the UI thread.
- Cargo workspace: `crates/core` (shared logic), `crates/daemon`,
  `crates/settings-ui`.
- MIT license, contribution guide and GitHub Actions CI.

### Changed

- The app is now **GitTrack Wallpaper**; the binaries are
  `git-track-wallpaper.exe` and `git-track-wallpaper-settings.exe`.
- The settings window is a separate process and no longer part of the daemon
  binary. `git-track-wallpaper.exe --settings` forwards to it.
- Data moved to `%APPDATA%\GitTrackWallpaper` and `%LOCALAPPDATA%\GitTrackWallpaper`.
  An existing `GitWallpaper` directory is migrated on first run, including the
  DPAPI-protected token.
- Renamed the mutex, the refresh event and the HKCU Run value; the installer
  removes the old `GitWallpaper` Run entry.

### Fixed

- The daemon resource script is generated into `OUT_DIR`, because `rc.exe`
  cannot resolve `..` inside a resource path from a workspace sub-crate.

[0.1.0]: https://github.com/ZeleMarsh7mary/gitTrack/releases/tag/v0.1.0
