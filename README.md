# GitWallpaper

Wallpaper kontribusi GitHub di desktop Windows. Satu binary Rust, dua mode
(daemon tray tanpa jendela dan jendela Settings sekali-pakai), compositing
sepenuhnya di CPU — tidak ada WebView, tidak ada `tokio`/`reqwest`, tidak ada
overlay GPU.

## Batasan yang dipegang

| Batasan | Nilai | Cara dipenuhi |
|---|---|---|
| RAM idle | < 10 MB | daemon tanpa GUI, `SetProcessWorkingSetSize` setelah setiap render |
| GPU idle | 0% | wallpaper ditulis ke PNG lalu `SystemParametersInfoW(SPI_SETDESKWALLPAPER)`, tidak ada window/overlay |
| Boot | tidak menyela login | fetch pertama ditunda 12 detik; wallpaper terakhir sudah tersimpan native di Windows |
| Installer | tanpa UAC | Inno Setup `PrivilegesRequired=lowest`, ke `%LOCALAPPDATA%\Programs\GitWallpaper`, auto-start HKCU |
| Token | tidak plaintext | Windows DPAPI (`CryptProtectData`) + entropi aplikasi, base64 di `config.toml` |

## Bangun & jalankan

```powershell
cargo build --release                       # target\release\git-wallpaper.exe
cargo test                                  # 8 test, termasuk compositor headless
cargo run --release -- --render-once out.png  # render satu wallpaper tanpa menyentuh desktop
cargo run --release --                        # daemon + ikon tray
cargo run --release -- --settings             # jendela pengaturan (keluar = seluruh memori kembali)
cargo run --release -- --apply-once           # satu siklus fetch + render + set wallpaper
```

Installer:

```powershell
iscc installer\setup.iss   # hasil: dist\GitWallpaper-0.1.0-setup.exe
```

Ikon dibuat ulang dengan `powershell -NoProfile -ExecutionPolicy Bypass -File assets\make-icon.ps1`
(16/32/48/256 px, tanpa pustaka gambar).

## Berkas & lokasi

| Path | Isi |
|---|---|
| `%APPDATA%\GitWallpaper\config.toml` | semua pengaturan; token hanya sebagai blob DPAPI |
| `%LOCALAPPDATA%\GitWallpaper\cache.json` | cache harian (`version`, `fetched_at`, `tz`, `days`), ditulis atomik |
| `%LOCALAPPDATA%\GitWallpaper\current_wallpaper.png` | wallpaper hasil render terakhir |
| `%LOCALAPPDATA%\GitWallpaper\daemon.log` | log ringkas (fetch gagal, render, ukuran PNG) |

## Jendela Settings

Dua kolom bergaya Windows 11 Settings (WinUI 3 Fluent dark), 960 x 640:

| Zona | Lebar | Isi |
|---|---|---|
| Kiri | 220 px | rail navigasi (row 36 px, radius 4) dengan pil aktif `#2D2D2D` + bar aksen 3 x 16 px di tepi kiri, status `Daemon: Running · Autostart: On · v0.1.0` di bawah |
| Kanan | sisa | header halaman 26 px, hero preview desktop 320 x 180 di kiri atas, lalu deretan `SettingsRow`; hanya kolom ini yang scroll |
| Bawah | 56 px | footer sticky: status/toast di kiri, `Test Connection` + `Save & Apply` di kanan, hairline 1 px di atasnya |

Token desain di satu tempat: background `#202020`, row `#2B2B2B` (hover `#323232`),
input `#222222`, hairline `Color32::from_white_alpha(14)`, aksen `#2EA043`, teks
`#F5F5F5`, muted `#9E9E9E`. Radius 4 px untuk kontrol, 5 px untuk row, 8 px untuk
hero preview; tombol sekunder `#353535` dengan border `#454545`, tinggi kontrol 30 px.

Caption bar natif dibuat menyatu lewat `DwmSetWindowAttribute`
(`DWMWA_USE_IMMERSIVE_DARK_MODE`, `DWMWA_CAPTION_COLOR`, `DWMWA_BORDER_COLOR`),
dipanggil sekali saat window dibuat; kalau OS menolak, title bar tetap default dan
aplikasi tetap jalan.

Tipografi: `C:\Windows\Fonts\segoeui.ttf` sebagai font primer (fallback ke font
bawaan egui bila berkas tidak ada) dan `consola.ttf` untuk angka/hex; ikon
`egui-phosphor` 0.7.3 (satu-satunya versi untuk egui 0.29) di indeks 1 sehingga
glyph teks tetap dari Segoe UI. Seluruh copy UI berbahasa Inggris.

## Arsitektur singkat

```
main.rs          argumen: daemon | --settings | --apply-once | --render-once
config.rs        skema config, default, clamp, tulis-atomik, path
security.rs      DPAPI encrypt/decrypt (entropi "GitWallpaper/v1")
git_fetcher.rs   ureq: GraphQL profile + REST repo, cache offline, grid 53x7
renderer/        Scene -> Pixmap: background | material (blur 3-pass + tint) | heatmap 53x7
win_api.rs       SPI_SETDESKWALLPAPER, EnumDisplaySettingsW, mutex, named event, trim, HKCU Run
daemon.rs        tray (tray-icon) + message loop + worker thread + 12s boot delay
ui_settings.rs   eframe/egui: shell 2-kolom WinUI 3 (nav 220 / konten / footer 56), tema token, Phosphor, hero preview
```

IPC antardua proses: named event `Local\GitWallpaperRefreshEvent` (auto-reset).
`--settings` menulis `config.toml` lalu `SetEvent`; daemon membaca ulang config
dan me-render. Single instance dijaga mutex `Local\GitWallpaperDaemonMutex`.

## Pemeriksaan yang sudah dijalankan

- `cargo check --all-targets` bersih.
- `cargo clippy --all-targets -- -D warnings` bersih.
- `cargo test` — 8 test lulus (DPAPI roundtrip, parsing URL repo, grid 53x7,
  wallpaper opaque + gradient, kotak heatmap tergambar, material mengubah piksel).
- `--render-once` menghasilkan PNG 1920x1080 sesuai resolusi monitor.

Angka footprint idle dan hasil kompilasi installer dicatat di `AGENT_RESULT.md`.
