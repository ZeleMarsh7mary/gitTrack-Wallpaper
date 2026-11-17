# Contributing to GitTrack Wallpaper

Thanks for taking the time. This is a small Windows tray app; the whole thing
builds in about five minutes on a warm cargo cache, and the parts are meant to
stay readable.

## Prerequisites

| Tool | Version | Why |
|---|---|---|
| Rust (MSVC toolchain) | 1.88+ | `rustup default stable-x86_64-pc-windows-msvc` |
| Visual Studio C++ build tools | 2019+ | linker plus `rc.exe` for the icon/manifest |
| Node.js | 20+ | the settings window frontend (Vite + React) |
| Inno Setup 6 | any | only needed to build the installer |
| WebView2 runtime | — | preinstalled on Windows 11 |

## Build and run

The settings binary embeds `crates/settings-ui/ui/dist` at compile time, so the
frontend has to be built first:

```powershell
Set-Location crates\settings-ui\ui
& "C:\Program Files\nodejs\npm.cmd" install
& "C:\Program Files\nodejs\npm.cmd" run build

Set-Location ..\..\..
cargo build --release
cargo run --release                                  # tray daemon
cargo run --release -p git-track-wallpaper-settings  # settings window
```

During frontend work, `npm run dev` serves the UI in a browser: `ui/src/lib/ipc.ts`
falls back to a demo backend whenever Tauri's IPC is unavailable, so the layout
can be iterated without rebuilding Rust.

## Checks before a pull request

```powershell
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
Set-Location crates\settings-ui\ui; & "C:\Program Files\nodejs\npm.cmd" run build
```

CI runs the same commands on `windows-latest`.

## Where things live

| Path | Responsibility |
|---|---|
| `crates/core/src/config.rs` | `config.toml` schema, clamping, atomic writes, data paths |
| `crates/core/src/security.rs` | DPAPI encrypt/decrypt for the GitHub token |
| `crates/core/src/git_fetcher.rs` | GitHub GraphQL (profile) and REST (repo), offline cache |
| `crates/core/src/renderer/` | the CPU compositor: background, material, 53x7 heatmap |
| `crates/core/src/win_api.rs` | Win32 wrappers: wallpaper, monitors, mutex, named event, HKCU Run |
| `crates/daemon/src/daemon.rs` | tray icon, message loop, worker thread, boot delay |
| `crates/settings-ui/src/commands.rs` | the IPC commands; every one delegates to `core` |
| `crates/settings-ui/ui/src/` | React pages, controls and the CSS design tokens |

Business logic belongs in `crates/core`. A command in `crates/settings-ui` may
translate, validate and forward, but must not re-implement rendering, fetching
or config handling; the preview and the wallpaper would drift apart.

## Style

- `cargo fmt` decides formatting; do not hand-format around it.
- Comments explain *why*. The *what* is in the code.
- UI copy is English, sentence case, no exclamation marks.
- New Win32 calls need a comment naming the flag and the failure mode.

## Pull requests

- One concern per pull request. A rename, a behaviour change and a formatting
  pass in one diff cannot be reviewed.
- Say what you ran. "Tests pass" is not a claim anyone can verify; paste the
  command and the summary line.
- New behaviour needs a test. The compositor and the fetcher are both pure
  enough to test without Windows APIs.
- Do not bump dependency versions in an unrelated pull request.
- Do not commit build output: `target/`, `dist/`, `crates/settings-ui/ui/dist`,
  `node_modules`.

## Reporting a bug

Include the Windows version, the app version (bottom of the settings rail), and
the relevant lines from `%LOCALAPPDATA%\GitTrackWallpaper\daemon.log`. Never
paste your GitHub token.

## License

Contributions are accepted under the MIT license in [LICENSE](LICENSE).
