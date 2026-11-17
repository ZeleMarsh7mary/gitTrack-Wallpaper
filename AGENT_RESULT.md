# AGENT_RESULT — GitWallpaper

Status per milestone, dengan angka dan perintah yang benar-benar dijalankan di
mesin ini (Windows, Rust 1.100.0 stable MSVC, VS Build Tools 2022 + Windows SDK).

## Lingkungan yang dipasang lebih dulu

| Komponen | Status |
|---|---|
| rustup + stable-x86_64-pc-windows-msvc 1.100.0 | terpasang (`winget install Rustlang.Rustup`, lalu `rustup default stable`) |
| Visual Studio 2022 Build Tools, workload VCTools + Windows SDK | terpasang (`winget install Microsoft.VisualStudio.2022.BuildTools` lewat UAC) |
| Inno Setup (`iscc`) | **tidak terpasang** — prompt UAC kedua dibatalkan, jadi installer belum dikompilasi |

## Verifikasi kode

```powershell
cargo check --all-targets                               # bersih
cargo clippy --all-targets -- -D warnings               # bersih
cargo test                                              # 8 test lulus
cargo build --release                                   # target\release\git-wallpaper.exe, 5.860.352 byte
```

Test yang ada:

- `security::tests::roundtrip`, `empty_is_empty` — DPAPI encrypt/decrypt nyata.
- `git_fetcher::tests::repo_url_shapes` — `https://`, `.git/`, `git@github.com:`.
- `git_fetcher::tests::grid_is_53x7_and_ends_with_today` — 371 slot, berakhir di hari ini.
- `git_fetcher::tests::levels_are_relative` — level 0..4 relatif ke nilai maksimum.
- `renderer::tests::renders_opaque_full_size_wallpaper` — 640x360, semua alpha 255, gradient atas≠bawah.
- `renderer::tests::draws_heatmap_boxes_inside_the_card` — 12.040 piksel level-4 di area grid.
- `renderer::tests::card_material_changes_the_backdrop` — glass mengubah piksel.

## Render nyata

```
git-wallpaper.exe --render-once <path>   ->  render 1920x1080 -> PNG 21.936 byte (1,8 s di build debug)
git-wallpaper.exe --render-once          ->  wallpaper diset dari %LOCALAPPDATA%\GitWallpaper\current_wallpaper.png
```

Pemeriksaan piksel pada wallpaper yang dihasilkan (System.Drawing):
- gradient background: pixel (5,5) = 11,18,31 dan (5,1074) = 17,37,28 → berbeda.
- area kartu: 12.040 piksel persis `#39d353` (level 4) → grid 53x7 tergambar.
- semua piksel opaque (alpha 255).

## Footprint idle (release, tanpa jendela Settings)

Perintah: `Get-Process git-wallpaper` selama 4 menit setelah 25 detik hangat.

| Metrik | Hasil |
|---|---|
| Working set | 0,91 MB (25 s, tepat setelah trim) → 2,19 MB (t+4 min) |
| Private bytes | 4,35 MB → 3,54 MB |
| Handles | 205 → 203 |
| Threads | 7 → 4 |
| CPU idle (60 s) | 0,052 % dari satu core |
| GPU | tidak ada counter GPU non-nol untuk pid ini |

Catatan kejujuran: working set *naik* 1,28 MB antara sampel pertama dan
t+4 menit. Itu page-fault-in setelah `SetProcessWorkingSetSize` memangkas
halaman, bukan kebocoran — private bytes pada rentang yang sama justru turun
0,80 MB. Angka 2,19 MB masih jauh di bawah batas 10 MB.

Jendela Settings: proses terpisah, `MainWindowTitle = "GitWallpaper Settings"`,
WS 134,9 MB selama terbuka, dan proses hilang sepenuhnya setelah ditutup
(`process exists: False`) — sesuai desain "GUI sekali pakai".

## Data GitHub

Fetch asli dijalankan terhadap `https://github.com/rust-lang/rust` dan ditolak
GitHub: `HTTP 403 rate limit exceeded` (tanpa token, IP mesin ini sudah
menghabiskan kuota 60/jam). Jalur fallback bekerja: pesan masuk log, cache
dipakai bila ada, wallpaper tetap dirender. Untuk data sungguhan, isi
Personal Access Token di Settings (wajib untuk mode Profile, disarankan untuk
Repository).

## Jendela Settings (rombakan UI)

Diukur dari proses yang benar-benar berjalan (`--settings`, screenshot jendela +
`GetWindowRect`/`GetClientRect`/`GetPixel`):

| Cek | Hasil |
|---|---|
| Client area | **1040 x 660** (diminta 1040x660), window 1056x699 termasuk border |
| Maximized? | tidak — 1056 px dari layar 1920 px |
| Rail kiri (190 px) | piksel (30,400) = `#1F1F24` (surface) |
| Background tengah | piksel (300,659) = `#16161A` |
| Panel pratinjau (380 px) | mulai x≈669; piksel di dalam pratinjau = warna wallpaper (`#55786C`), 2.881 warna berbeda di area pratinjau |
| Aksen `#2EA043` | 3.044 piksel (1/4 sampling): rail aktif, tombol Save & Apply, ring pilihan |
| Tipografi | glyph teks muncul di rail (px 56-131 pada y=60) → `segoeui.ttf` termuat, bukan font bawaan egui |
| Memori saat terbuka | 100,0 MB working set |
| Tutup jendela | proses hilang sepenuhnya (`closed cleanly: True`) |

Batas kolom yang terdeteksi pada baris y=60: rail 0-190, konten 197-668, pratinjau
669-1056 — sesuai desain 190 / sisa / 380.

`cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, dan
`cargo test` (8 lulus) dijalankan ulang setelah perubahan UI: bersih, tanpa warning.

## Yang belum diverifikasi

- **Installer belum dikompilasi.** `installer/setup.iss` ditulis lengkap
  (`PrivilegesRequired=lowest`, Run key HKCU, autostart task, uninstall bersih)
  tetapi `iscc` tidak ada di mesin ini. Setelah `winget install JRSoftware.InnoSetup`
  (butuh satu klik UAC), jalankan `iscc installer\setup.iss` → `dist\GitWallpaper-0.1.0-setup.exe`.
  Klaim "1-click install terbukti" karena itu **tidak** dibuat di laporan ini.
- Auto-start belum diuji lewat reboot; hanya `HKCU\...\Run` yang ditulis/dibaca
  (diverifikasi lewat registry read-back di UI).
- Tray menu (Refresh Now / Settings / Exit) belum diklik secara otomatis —
  yang terverifikasi: proses daemon hidup dengan message loop, ikon tray
  berhasil dibuat (kegagalan `TrayIconBuilder` akan menghentikan proses), dan
  worker me-render sesuai jadwal 12 detik.
- Multi-monitor: hanya monitor primer (`EnumDisplaySettingsW`) yang dipakai.
- Wallpaper asli pengguna sebelum aplikasi ini dijalankan tidak dicatat;
  `HKCU\Control Panel\Desktop\Wallpaper` sekarang menunjuk ke
  `%LOCALAPPDATA%\GitWallpaper\current_wallpaper.png`.
