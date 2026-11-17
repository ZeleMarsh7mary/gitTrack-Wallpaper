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

Tiga zona, bukan satu kolom:

| Zona | Lebar | Isi |
|---|---|---|
| Kiri | 190 px | rail navigasi dengan ikon Phosphor + badge status Daemon & Autostart |
| Tengah | sisa | form vertikal di dalam kartu; satu-satunya area yang scroll |
| Kanan | 380 px | pratinjau langsung dalam bingkai monitor + info ukuran/posisi + tombol Test/Save di footer |

Tema adalah token di satu tempat, bukan `Visuals::dark()`: background `#16161A`,
surface kartu `#1F1F24`, border `#2D2D35`, input `#111114`, aksen `#2EA043`,
teks `#E6E6EA`, muted `#8B8B93`. Radius: jendela 12, kartu 10, input/tombol 6.
`item_spacing` 12/10, `button_padding` 14/8, tinggi kontrol 32 px.

Tipografi: `C:\Windows\Fonts\segoeui.ttf` disisipkan sebagai font primer
(fallback ke font bawaan egui bila berkas tidak ada) dan `consola.ttf` untuk
angka/hex; ikon dari `egui-phosphor` 0.7.3 (cocok dengan egui 0.29) di indeks 1
sehingga glyph teks tetap dari Segoe UI.

## Arsitektur singkat

```
main.rs          argumen: daemon | --settings | --apply-once | --render-once
config.rs        skema config, default, clamp, tulis-atomik, path
security.rs      DPAPI encrypt/decrypt (entropi "GitWallpaper/v1")
git_fetcher.rs   ureq: GraphQL profile + REST repo, cache offline, grid 53x7
renderer/        Scene -> Pixmap: background | material (blur 3-pass + tint) | heatmap 53x7
win_api.rs       SPI_SETDESKWALLPAPER, EnumDisplaySettingsW, mutex, named event, trim, HKCU Run
daemon.rs        tray (tray-icon) + message loop + worker thread + 12s boot delay
ui_settings.rs   eframe/egui: shell 3-zona (nav 190 / form / preview 380), tema token, Phosphor, pratinjau langsung
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
