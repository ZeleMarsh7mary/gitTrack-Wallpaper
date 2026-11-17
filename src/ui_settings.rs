//! `--settings` window: a separate, short-lived process (eframe/egui).
//!
//! Shell layout, outside-in: left navigation rail (190 px) -> right live-preview
//! panel (380 px) -> central scrolling form. The window exits completely when
//! closed, so none of this GUI memory stays resident in the daemon.

use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Button, Color32, FontFamily, FontId, Frame, Layout, Margin, RichText, Rounding,
    ScrollArea, Sense, Stroke, TextStyle, Vec2,
};
use egui_phosphor::regular as icons;

use crate::config::{
    self, parse_hex, to_hex, BgFit, BgMode, Config, MaterialKind, Position, SourceMode, PALETTES,
};
use crate::git_fetcher::{self, DayStat};
use crate::renderer::{layout as scene_layout, render, Scene};
use crate::{security, win_api};

// ---------------------------------------------------------------- theme tokens

const BG: Color32 = Color32::from_rgb(0x16, 0x16, 0x1A);
const SURFACE: Color32 = Color32::from_rgb(0x1F, 0x1F, 0x24);
const SURFACE_HI: Color32 = Color32::from_rgb(0x27, 0x27, 0x2E);
const INPUT: Color32 = Color32::from_rgb(0x11, 0x11, 0x14);
const BORDER: Color32 = Color32::from_rgb(0x2D, 0x2D, 0x35);
const ACCENT: Color32 = Color32::from_rgb(0x2E, 0xA0, 0x43);
const TEXT: Color32 = Color32::from_rgb(0xE6, 0xE6, 0xEA);
const MUTED: Color32 = Color32::from_rgb(0x8B, 0x8B, 0x93);
const WARN: Color32 = Color32::from_rgb(0xD9, 0x9A, 0x3F);
const DANGER: Color32 = Color32::from_rgb(0xE5, 0x53, 0x4B);

const NAV_WIDTH: f32 = 190.0;
const PREVIEW_WIDTH: f32 = 380.0;
const CARD_PAD_X: f32 = 16.0;
const CARD_PAD_Y: f32 = 14.0;
const ROW_HEIGHT: f32 = 32.0;

fn accent_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(0x2E, 0xA0, 0x43, 34)
}

// ---------------------------------------------------------------------- theme

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts
            .font_data
            .insert("segoe-ui".to_owned(), egui::FontData::from_owned(bytes));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe-ui".to_owned());
    }
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\consola.ttf") {
        fonts
            .font_data
            .insert("consolas".to_owned(), egui::FontData::from_owned(bytes));
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "consolas".to_owned());
    }
    // Icons land at index 1 of the proportional family, behind the system face.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
}

fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();

    v.panel_fill = BG;
    v.window_fill = SURFACE;
    v.extreme_bg_color = INPUT;
    v.faint_bg_color = SURFACE_HI;
    v.override_text_color = Some(TEXT);
    v.window_stroke = pen(1.0, BORDER);
    v.window_rounding = Rounding::same(12.0);
    v.menu_rounding = Rounding::same(8.0);
    v.selection = egui::style::Selection {
        bg_fill: accent_fill(),
        stroke: pen(1.0, ACCENT),
    };

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = SURFACE;
    w.noninteractive.weak_bg_fill = SURFACE;
    w.noninteractive.bg_stroke = pen(1.0, BORDER);
    w.noninteractive.fg_stroke = pen(1.0, MUTED);
    w.noninteractive.rounding = Rounding::same(6.0);

    w.inactive.bg_fill = INPUT;
    w.inactive.weak_bg_fill = SURFACE_HI;
    w.inactive.bg_stroke = pen(1.0, BORDER);
    w.inactive.fg_stroke = pen(1.0, TEXT);
    w.inactive.rounding = Rounding::same(6.0);

    w.hovered.bg_fill = SURFACE_HI;
    w.hovered.weak_bg_fill = SURFACE_HI;
    w.hovered.bg_stroke = pen(1.0, ACCENT.gamma_multiply(0.7));
    w.hovered.fg_stroke = pen(1.5, TEXT);
    w.hovered.rounding = Rounding::same(6.0);

    w.active.bg_fill = INPUT;
    w.active.weak_bg_fill = INPUT;
    w.active.bg_stroke = pen(1.0, ACCENT);
    w.active.fg_stroke = pen(1.5, TEXT);
    w.active.rounding = Rounding::same(6.0);

    w.open.bg_fill = SURFACE_HI;
    w.open.weak_bg_fill = SURFACE_HI;
    w.open.bg_stroke = pen(1.0, BORDER);
    w.open.fg_stroke = pen(1.0, TEXT);
    w.open.rounding = Rounding::same(6.0);

    style.visuals = v;

    let sp = &mut style.spacing;
    sp.item_spacing = Vec2::new(12.0, 10.0);
    sp.button_padding = Vec2::new(14.0, 8.0);
    sp.interact_size = Vec2::new(48.0, ROW_HEIGHT);
    sp.slider_width = 180.0;
    sp.indent = 16.0;
    sp.scroll.floating = false;
    sp.scroll.bar_width = 8.0;

    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(13.5, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.5, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_style(style);
}

// ---------------------------------------------------------------- entry point

pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 660.0])
            .with_min_inner_size([860.0, 540.0])
            .with_resizable(true)
            .with_title("GitWallpaper Settings"),
        ..Default::default()
    };
    eframe::run_native(
        "GitWallpaper Settings",
        options,
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(SettingsApp::new(cc)))
        }),
    )
    .map_err(|e| format!("eframe: {e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Source,
    Background,
    Material,
    Track,
}

impl Tab {
    const ALL: [(Tab, &'static str, &'static str); 4] = [
        (Tab::Source, icons::GIT_BRANCH, "Git Source"),
        (Tab::Background, icons::IMAGE, "Background"),
        (Tab::Material, icons::SPARKLE, "Material & Blur"),
        (Tab::Track, icons::GRID_FOUR, "Track & Boxes"),
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastKind {
    Success,
    Error,
    Info,
}

struct Toast {
    text: String,
    kind: ToastKind,
    until: Instant,
}

struct SettingsApp {
    cfg: Config,
    api_key: String,
    show_token: bool,
    tab: Tab,
    days: Vec<DayStat>,
    preview: Option<egui::TextureHandle>,
    last_rendered: Option<Config>,
    toast: Option<Toast>,
    resolution: (u32, u32),
}

impl SettingsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (mut cfg, warn) = Config::load_or_default();
        // The Run key is the source of truth for "starts with Windows"; trust it
        // when it says yes so a saved toggle never silently disables the daemon.
        cfg.start_with_windows = cfg.start_with_windows || win_api::is_start_with_windows();
        let api_key = security::decrypt(&cfg.source.encrypted_api_key).unwrap_or_default();
        let days = git_fetcher::load_cache().unwrap_or_else(git_fetcher::demo_days);
        let toast = warn.map(|text| Toast {
            text,
            kind: ToastKind::Error,
            until: Instant::now() + Duration::from_secs(6),
        });
        Self {
            cfg,
            api_key,
            show_token: false,
            tab: Tab::Source,
            days,
            preview: None,
            last_rendered: None,
            toast,
            resolution: win_api::primary_monitor_size(),
        }
    }

    fn notify(&mut self, kind: ToastKind, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            kind,
            until: Instant::now() + Duration::from_secs(5),
        });
    }

    // ------------------------------------------------------------- preview

    fn rebuild_preview(&mut self, ctx: &egui::Context, width: f32) {
        let (sw, sh) = self.resolution;
        let width = width.clamp(240.0, 360.0).round() as u32;
        let scale = width as f32 / sw as f32;
        let height = ((sh as f32 * scale).round() as u32).max(120);
        let scene = Scene {
            config: &self.cfg,
            days: &self.days,
            size_px: (width, height),
            scale,
        };
        let Ok(pm) = render(&scene) else {
            return;
        };
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [pm.width() as usize, pm.height() as usize],
            pm.data(),
        );
        let texture = ctx.load_texture("preview", image, egui::TextureOptions::LINEAR);
        self.preview = Some(texture);
        self.last_rendered = Some(self.cfg.clone());
    }

    // ------------------------------------------------------------- actions

    fn save_and_apply(&mut self) {
        if self.cfg.source.input_target.trim().is_empty() {
            self.notify(
                ToastKind::Info,
                "Target GitHub masih kosong: isi username atau URL repo agar ada data.",
            );
        }
        match security::encrypt(&self.api_key) {
            Ok(blob) => self.cfg.source.encrypted_api_key = blob,
            Err(e) => {
                self.notify(ToastKind::Error, format!("Token gagal dienkripsi: {e}"));
                return;
            }
        }
        self.cfg.sanitize();
        if let Err(e) = self.cfg.save() {
            self.notify(ToastKind::Error, format!("Gagal menyimpan config: {e}"));
            return;
        }
        if let Ok(exe) = std::env::current_exe() {
            let _ = win_api::set_start_with_windows(&exe, self.cfg.start_with_windows);
        }
        let autostart = win_api::is_start_with_windows();
        let signalled = win_api::signal_refresh_event();
        let mut note = if signalled {
            "Config disimpan, wallpaper sedang diperbarui.".to_string()
        } else if std::env::current_exe()
            .map(|exe| std::process::Command::new(exe).spawn().is_ok())
            .unwrap_or(false)
        {
            "Config disimpan, daemon dijalankan.".to_string()
        } else {
            "Config disimpan. Daemon tidak berjalan; jalankan git-wallpaper.exe.".to_string()
        };
        note.push_str(if autostart {
            " Autostart aktif."
        } else {
            " Autostart nonaktif."
        });
        self.notify(ToastKind::Success, note);
    }

    fn test_connection(&mut self) {
        let mut probe = self.cfg.clone();
        match security::encrypt(&self.api_key) {
            Ok(blob) => probe.source.encrypted_api_key = blob,
            Err(e) => {
                self.notify(ToastKind::Error, format!("Token gagal dienkripsi: {e}"));
                return;
            }
        }
        match git_fetcher::fetch(&probe) {
            Ok(days) => {
                let total: u32 = days.iter().map(|d| d.count).sum();
                self.days = days;
                let _ = git_fetcher::save_cache(&self.days);
                self.last_rendered = None;
                self.notify(
                    ToastKind::Success,
                    format!("Koneksi OK: {total} kontribusi dalam setahun terakhir."),
                );
            }
            Err(e) => self.notify(ToastKind::Error, format!("Koneksi gagal: {e}")),
        }
    }

    // -------------------------------------------------------------- shell

    fn draw_nav(&mut self, ctx: &egui::Context) {
        let daemon_up = win_api::daemon_event_exists();
        let autostart = win_api::is_start_with_windows();
        let mut tab = self.tab;

        egui::SidePanel::left("nav")
            .exact_width(NAV_WIDTH)
            .resizable(false)
            .frame(
                Frame::none()
                    .fill(SURFACE)
                    .inner_margin(Margin::symmetric(12.0, 14.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icons::PALETTE).size(20.0).color(ACCENT));
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("GitWallpaper").size(14.5).strong());
                        ui.label(
                            RichText::new("Kontribusi GitHub")
                                .size(10.5)
                                .color(MUTED),
                        );
                    });
                });
                ui.add_space(20.0);

                for (value, icon, label) in Tab::ALL {
                    nav_row(ui, value, icon, label, &mut tab);
                    ui.add_space(4.0);
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .size(10.0)
                            .color(MUTED),
                    );
                    ui.add_space(6.0);
                    badge_row(
                        ui,
                        "Autostart",
                        if autostart { "aktif" } else { "nonaktif" },
                        Some(autostart),
                    );
                    ui.add_space(6.0);
                    badge_row(
                        ui,
                        "Daemon",
                        if daemon_up { "berjalan" } else { "mati" },
                        Some(daemon_up),
                    );
                });
            });

        self.tab = tab;
    }

    fn draw_preview_panel(&mut self, ctx: &egui::Context) {
        let (sw, sh) = self.resolution;
        let scene = Scene {
            config: &self.cfg,
            days: &self.days,
            size_px: (320, 180),
            scale: 320.0 / sw as f32,
        };
        let l = scene_layout(&scene);
        let theme_name = material_name(self.cfg.card.material);
        let pos_name = position_name(self.cfg.card.position);
        let card_w = l.card_w.round();
        let card_h = l.card_h.round();

        egui::SidePanel::right("preview")
            .exact_width(PREVIEW_WIDTH)
            .resizable(false)
            .frame(
                Frame::none()
                    .fill(SURFACE)
                    .inner_margin(Margin::symmetric(16.0, 14.0)),
            )
            .show(ctx, |ui| {
                let mut save = false;
                let mut test = false;

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    save = ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            Button::new(
                                RichText::new(format!("{}  Save & Apply", icons::FLOPPY_DISK))
                                    .size(13.5)
                                    .color(Color32::WHITE),
                            )
                            .fill(ACCENT)
                            .stroke(Stroke::NONE)
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked();
                    ui.add_space(8.0);
                    test = ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            Button::new(
                                RichText::new(format!("{}  Test Connection", icons::PULSE))
                                    .size(13.0),
                            )
                            .fill(SURFACE_HI)
                            .stroke(pen(1.0, BORDER))
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked();

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);
                    info_row(ui, "Material", theme_name);
                    info_row(ui, "Posisi", pos_name);
                    info_row(
                        ui,
                        "Kartu",
                        &format!("{card_w:.0} x {card_h:.0} px"),
                    );
                    info_row(ui, "Resolusi", &format!("{sw} x {sh}"));
                    info_row(
                        ui,
                        "Data",
                        &format!("{} hari", self.days.len()),
                    );
                    ui.add_space(8.0);

                    // Monitor mockup, drawn above the info rows (bottom-up order).
                    Frame::none()
                        .fill(Color32::from_rgb(0x0E, 0x0E, 0x11))
                        .stroke(pen(1.0, BORDER))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            if let Some(tex) = &self.preview {
                                let size = tex.size_vec2();
                                ui.add(
                                    egui::Image::new(egui::load::SizedTexture::new(
                                        tex.id(),
                                        size,
                                    ))
                                    .max_width(ui.available_width()),
                                );
                            } else {
                                ui.allocate_space(Vec2::new(ui.available_width(), 140.0));
                            }
                        });
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("PRATINJAU LANGSUNG")
                            .size(10.0)
                            .strong()
                            .color(MUTED),
                    );
                });

                if save {
                    self.save_and_apply();
                }
                if test {
                    self.test_connection();
                }
            });
    }

    fn draw_form(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(18.0, 16.0)),
            )
            .show(ctx, |ui| {
                let content_w = ui.available_width();
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(content_w);
                        match self.tab {
                            Tab::Source => self.tab_source(ui, content_w),
                            Tab::Background => self.tab_background(ui, content_w),
                            Tab::Material => self.tab_material(ui, content_w),
                            Tab::Track => self.tab_track(ui, content_w),
                        }
                    });
            });
    }

    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        let remaining = toast.until.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            self.toast = None;
            return;
        }
        let (color, icon) = match toast.kind {
            ToastKind::Success => (ACCENT, icons::CHECK_CIRCLE),
            ToastKind::Error => (DANGER, icons::WARNING),
            ToastKind::Info => (MUTED, icons::INFO),
        };
        let screen = ctx.screen_rect();
        let pos = egui::pos2(NAV_WIDTH + 20.0, screen.bottom() - 62.0);
        let text = toast.text.clone();

        egui::Area::new(egui::Id::new("gitwallpaper-toast"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                Frame::none()
                    .fill(SURFACE_HI)
                    .stroke(pen(1.0, color.gamma_multiply(0.6)))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).size(14.0).color(color));
                            ui.label(RichText::new(text).size(12.0).color(TEXT));
                        });
                    });
            });
        ctx.request_repaint_after(remaining);
    }

    // --------------------------------------------------------------- tabs

    fn tab_source(&mut self, ui: &mut egui::Ui, width: f32) {
        card(
            ui,
            width,
            "Sumber data",
            Some("Dari mana kontribusi diambil."),
            |ui| {
                section_label(ui, "Tipe sumber");
                segmented(
                    ui,
                    &mut self.cfg.source.mode,
                    &[
                        (SourceMode::Profile, "Profile"),
                        (SourceMode::Repository, "Repository"),
                    ],
                );
                ui.add_space(10.0);

                let (title, hint, helper) = match self.cfg.source.mode {
                    SourceMode::Profile => (
                        "Username GitHub",
                        "octocat",
                        "Grafik kontribusi tahunan akun ini (GraphQL, butuh token).",
                    ),
                    SourceMode::Repository => (
                        "URL repository",
                        "https://github.com/owner/repo",
                        "Riwayat commit repo ini. Repo privat butuh token dengan izin Contents: Read.",
                    ),
                };
                section_label(ui, title);
                ui.add_sized(
                    [ui.available_width(), ROW_HEIGHT],
                    egui::TextEdit::singleline(&mut self.cfg.source.input_target)
                        .hint_text(hint),
                );
                helper_text(ui, helper);
            },
        );

        card(
            ui,
            width,
            "Autentikasi & sinkronisasi",
            Some("Token tidak pernah ditulis sebagai plaintext."),
            |ui| {
                section_label(ui, "Personal Access Token");
                ui.horizontal(|ui| {
                    let toggle_w = 36.0;
                    let field_w = (ui.available_width() - toggle_w - 8.0).max(80.0);
                    ui.add_sized(
                        [field_w, ROW_HEIGHT],
                        egui::TextEdit::singleline(&mut self.api_key)
                            .password(!self.show_token)
                            .hint_text("ghp_... / github_pat_..."),
                    );
                    let (icon, tip) = if self.show_token {
                        (icons::EYE_SLASH, "Sembunyikan token")
                    } else {
                        (icons::EYE, "Tampilkan token")
                    };
                    let resp = ui
                        .add_sized(
                            [toggle_w, ROW_HEIGHT],
                            Button::new(RichText::new(icon).size(15.0))
                                .fill(INPUT)
                                .stroke(pen(1.0, BORDER))
                                .rounding(Rounding::same(6.0)),
                        )
                        .on_hover_text(tip);
                    if resp.clicked() {
                        self.show_token = !self.show_token;
                    }
                });
                helper_text(
                    ui,
                    "Disimpan terenkripsi DPAPI (entropi GitWallpaper/v1), terikat akun Windows ini.",
                );

                ui.add_space(14.0);
                slider_row_u32(
                    ui,
                    "Interval refresh",
                    &mut self.cfg.source.refresh_interval_minutes,
                    5..=240,
                    " menit",
                );
                helper_text(ui, "Seberapa sering daemon mengambil data baru dari GitHub.");
            },
        );
    }

    fn tab_background(&mut self, ui: &mut egui::Ui, width: f32) {
        card(
            ui,
            width,
            "Latar belakang desktop",
            Some("Gambar, warna solid, atau gradient dua titik."),
            |ui| {
                segmented(
                    ui,
                    &mut self.cfg.background.mode,
                    &[
                        (BgMode::Image, "Gambar"),
                        (BgMode::Solid, "Solid"),
                        (BgMode::Gradient, "Gradient"),
                    ],
                );
                ui.add_space(10.0);

                section_label(ui, "Berkas gambar");
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [120.0, ROW_HEIGHT],
                            Button::new(RichText::new("Pilih berkas...").size(12.5)),
                        )
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "webp"])
                            .pick_file()
                        {
                            self.cfg.background.image_path = path.display().to_string();
                            self.cfg.background.mode = BgMode::Image;
                        }
                    }
                    let shown = if self.cfg.background.image_path.is_empty() {
                        "(belum ada gambar)".to_string()
                    } else {
                        self.cfg.background.image_path.clone()
                    };
                    ui.add(
                        egui::Label::new(RichText::new(shown).size(11.5).color(MUTED))
                            .truncate(),
                    );
                });
                helper_text(ui, "Dipakai hanya jika mode Gambar aktif.");

                ui.add_space(14.0);
                section_label(ui, "Cara gambar mengisi layar");
                segmented(
                    ui,
                    &mut self.cfg.background.fit,
                    &[
                        (BgFit::Cover, "Cover"),
                        (BgFit::Contain, "Contain"),
                        (BgFit::Stretch, "Stretch"),
                    ],
                );
            },
        );

        card(
            ui,
            width,
            "Warna",
            Some("Dipakai untuk solid, gradient, dan area kosong pada mode Contain."),
            |ui| {
                color_field(ui, "Warna utama", &mut self.cfg.background.color_primary);
                ui.add_space(10.0);
                color_field(ui, "Warna kedua", &mut self.cfg.background.color_secondary);
            },
        );
    }

    fn tab_material(&mut self, ui: &mut egui::Ui, width: f32) {
        card(
            ui,
            width,
            "Material kartu",
            Some("Efek backdrop di belakang heatmap."),
            |ui| {
                let mut kind = self.cfg.card.material;
                ui.horizontal_wrapped(|ui| {
                    for value in [
                        MaterialKind::None,
                        MaterialKind::Solid,
                        MaterialKind::Glass,
                        MaterialKind::Acrylic,
                        MaterialKind::Mica,
                    ] {
                        let selected = kind == value;
                        let label = material_name(value);
                        let btn = Button::new(
                            RichText::new(label)
                                .size(12.0)
                                .color(if selected { TEXT } else { MUTED }),
                        )
                        .fill(if selected { accent_fill() } else { INPUT })
                        .stroke(pen(
                            1.0,
                            if selected { ACCENT } else { BORDER },
                        ))
                        .rounding(Rounding::same(999.0))
                        .min_size(Vec2::new(0.0, 28.0));
                        if ui.add(btn).clicked() {
                            kind = value;
                        }
                    }
                });
                self.cfg.card.material = kind;
                helper_text(ui, material_hint(kind));

                ui.add_space(14.0);
                slider_row(
                    ui,
                    "Opacity",
                    &mut self.cfg.card.opacity,
                    0.0..=1.0,
                    "",
                    2,
                );
                ui.add_space(10.0);
                slider_row(
                    ui,
                    "Blur radius",
                    &mut self.cfg.card.blur_radius,
                    0.0..=50.0,
                    " px",
                    0,
                );
                ui.add_space(10.0);
                slider_row(
                    ui,
                    "Corner radius",
                    &mut self.cfg.card.corner_radius,
                    0.0..=80.0,
                    " px",
                    0,
                );
                ui.add_space(12.0);
                color_field(ui, "Tint kartu", &mut self.cfg.card.tint_color);
            },
        );

        card(
            ui,
            width,
            "Posisi kartu",
            Some("Klik salah satu sel untuk menempatkan kartu."),
            |ui| {
                position_grid(ui, &mut self.cfg.card.position);
                ui.add_space(10.0);
                let mut custom = self.cfg.card.position == Position::Custom;
                if ui
                    .checkbox(
                        &mut custom,
                        RichText::new("Posisi custom (offset piksel)").size(12.5),
                    )
                    .changed()
                {
                    self.cfg.card.position = if custom {
                        Position::Custom
                    } else {
                        Position::Center
                    };
                }
                if self.cfg.card.position == Position::Custom {
                    ui.add_space(8.0);
                    ui.add(
                        egui::Slider::new(&mut self.cfg.card.offset_x, -1200..=1200)
                            .text("Offset X (px)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.cfg.card.offset_y, -800..=800)
                            .text("Offset Y (px)"),
                    );
                }
            },
        );
    }

    fn tab_track(&mut self, ui: &mut egui::Ui, width: f32) {
        card(
            ui,
            width,
            "Tema warna",
            Some("Preset siap pakai; klik untuk menerapkan."),
            |ui| {
                let current = self.cfg.track.colors.clone();
                for (name, colors) in PALETTES {
                    let selected = current
                        .iter()
                        .zip(colors.iter())
                        .all(|(a, b)| a.eq_ignore_ascii_case(b));
                    if Frame::none()
                        .fill(if selected { accent_fill() } else { INPUT })
                        .stroke(pen(
                            1.0,
                            if selected { ACCENT } else { BORDER },
                        ))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(*name)
                                        .size(12.5)
                                        .color(if selected { TEXT } else { MUTED }),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    for hex in colors.iter().rev() {
                                        swatch(ui, hex, 14.0, false);
                                    }
                                });
                            });
                        })
                        .response
                        .interact(Sense::click())
                        .on_hover_text("Terapkan tema ini")
                        .clicked()
                    {
                        for (slot, hex) in self.cfg.track.colors.iter_mut().zip(colors.iter()) {
                            *slot = (*hex).to_string();
                        }
                    }
                    ui.add_space(6.0);
                }
            },
        );

        card(
            ui,
            width,
            "Level warna",
            Some("Level 0 = tanpa kontribusi, level 4 = paling padat."),
            |ui| {
                let last = self.cfg.track.colors.len() - 1;
                for (i, slot) in self.cfg.track.colors.iter_mut().enumerate() {
                    let mut hex = slot.clone();
                    if color_field(ui, &format!("Level {i}"), &mut hex) {
                        *slot = hex;
                    }
                    if i < last {
                        ui.add_space(8.0);
                    }
                }
            },
        );

        card(
            ui,
            width,
            "Kotak & header",
            Some("Ukuran grid heatmap 53 x 7."),
            |ui| {
                slider_row(
                    ui,
                    "Ukuran kotak",
                    &mut self.cfg.track.box_size,
                    6.0..=40.0,
                    " px",
                    1,
                );
                ui.add_space(10.0);
                slider_row(
                    ui,
                    "Jarak antar kotak",
                    &mut self.cfg.track.box_gap,
                    0.0..=12.0,
                    " px",
                    1,
                );
                ui.add_space(10.0);
                slider_row(
                    ui,
                    "Radius kotak",
                    &mut self.cfg.track.box_radius,
                    0.0..=20.0,
                    " px",
                    1,
                );
                ui.add_space(12.0);
                ui.checkbox(
                    &mut self.cfg.track.show_stats_header,
                    RichText::new("Tampilkan header statistik").size(12.5),
                );
                ui.add_space(10.0);
                color_field(ui, "Warna teks header", &mut self.cfg.track.text_color);
            },
        );

        card(
            ui,
            width,
            "Sistem",
            Some("Berjalan tanpa hak administrator."),
            |ui| {
                ui.checkbox(
                    &mut self.cfg.start_with_windows,
                    RichText::new("Jalankan otomatis saat Windows login").size(12.5),
                );
                ui.add_space(10.0);
                helper_text(ui, &format!("Config: {}", config::config_path().display()));
                helper_text(ui, &format!("Cache: {}", config::cache_path().display()));
            },
        );
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_nav(ctx);
        self.draw_preview_panel(ctx);
        self.draw_form(ctx);

        if self.last_rendered.as_ref() != Some(&self.cfg) {
            let width = PREVIEW_WIDTH - 48.0;
            self.rebuild_preview(ctx, width);
        }
        self.draw_toast(ctx);
    }
}

// ------------------------------------------------------------------ widgets

fn nav_row(ui: &mut egui::Ui, value: Tab, icon: &str, label: &str, current: &mut Tab) {
    let selected = *current == value;
    let text = RichText::new(format!("{icon}   {label}"))
        .size(13.0)
        .color(if selected { TEXT } else { MUTED });
    let button = Button::new(text)
        .fill(if selected { accent_fill() } else { Color32::TRANSPARENT })
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(8.0))
        .min_size(Vec2::new(ui.available_width(), 36.0));
    if ui.add(button).clicked() {
        *current = value;
    }
}

fn badge_row(ui: &mut egui::Ui, label: &str, value: &str, ok: Option<bool>) {
    let dot = match ok {
        Some(true) => ACCENT,
        Some(false) => MUTED,
        None => WARN,
    };
    Frame::none()
        .fill(INPUT)
        .stroke(pen(1.0, BORDER))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 3.0, dot);
                ui.label(RichText::new(label).size(10.5).color(MUTED));
                ui.label(RichText::new(value).size(10.5).color(TEXT));
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn card<R>(
    ui: &mut egui::Ui,
    outer_width: f32,
    title: &str,
    subtitle: Option<&str>,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let inner_width = (outer_width - CARD_PAD_X * 2.0).max(120.0);
    let mut out = None;
    Frame::none()
        .fill(SURFACE)
        .stroke(pen(1.0, BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::symmetric(CARD_PAD_X, CARD_PAD_Y))
        .show(ui, |ui| {
            ui.set_width(inner_width);
            ui.label(RichText::new(title).size(14.0).strong().color(TEXT));
            if let Some(sub) = subtitle {
                ui.label(RichText::new(sub).size(11.0).color(MUTED));
            }
            ui.add_space(10.0);
            out = Some(add(ui));
        });
    ui.add_space(12.0);
    out.expect("card body always runs")
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(12.0).strong().color(TEXT));
    ui.add_space(4.0);
}

fn helper_text(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.add(egui::Label::new(RichText::new(text).size(10.8).color(MUTED)).wrap());
}

fn segmented<T: Copy + PartialEq>(ui: &mut egui::Ui, current: &mut T, options: &[(T, &str)]) {
    Frame::none()
        .fill(INPUT)
        .stroke(pen(1.0, BORDER))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (value, text) in options {
                    let selected = *current == *value;
                    let button = Button::new(
                        RichText::new(*text)
                            .size(12.0)
                            .color(if selected { TEXT } else { MUTED }),
                    )
                    .fill(if selected { accent_fill() } else { Color32::TRANSPARENT })
                    .stroke(Stroke::NONE)
                    .rounding(Rounding::same(6.0))
                    .min_size(Vec2::new(0.0, 26.0));
                    if ui.add(button).clicked() && !selected {
                        *current = *value;
                    }
                }
            });
        });
}

fn slider_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    decimals: usize,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.0).color(TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let text = format!("{value:.*}{suffix}", decimals);
            Frame::none()
                .fill(INPUT)
                .stroke(pen(1.0, BORDER))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(text).size(11.0).color(TEXT));
                });
        });
    });
    ui.add_sized(
        [ui.available_width(), 20.0],
        egui::Slider::new(value, range).show_value(false),
    );
}

fn slider_row_u32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    suffix: &str,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.0).color(TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            Frame::none()
                .fill(INPUT)
                .stroke(pen(1.0, BORDER))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{value}{suffix}"))
                            .size(11.0)
                            .color(TEXT),
                    );
                });
        });
    });
    ui.add_sized(
        [ui.available_width(), 20.0],
        egui::Slider::new(value, range).show_value(false),
    );
}

fn color_field(ui: &mut egui::Ui, label: &str, hex: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut value = parse_hex(hex).unwrap_or(0x101010);
        let mut srgb = [
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8,
        ];
        if ui.color_edit_button_srgb(&mut srgb).changed() {
            value = ((srgb[0] as u32) << 16) | ((srgb[1] as u32) << 8) | srgb[2] as u32;
            *hex = to_hex(value);
            changed = true;
        }
        ui.label(RichText::new(label).size(12.0).color(TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let resp = ui.add_sized(
                [96.0, 26.0],
                egui::TextEdit::singleline(hex).font(TextStyle::Monospace),
            );
            changed |= resp.changed();
        });
    });
    changed
}

fn swatch(ui: &mut egui::Ui, hex: &str, size: f32, ring: bool) {
    let (rgb_r, rgb_g, rgb_b) = config::rgb(hex, 0x161b22);
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().rect_filled(
        rect,
        Rounding::same(4.0),
        Color32::from_rgb(rgb_r, rgb_g, rgb_b),
    );
    if ring {
        ui.painter()
            .rect_stroke(rect, Rounding::same(4.0), pen(1.5, ACCENT));
    }
}

fn position_grid(ui: &mut egui::Ui, current: &mut Position) {
    const CELL: f32 = 34.0;
    let mut chosen: Option<Position> = None;
    // Row 1, row 3 and the middle cell are live; the rest are placeholders to keep
    // a 3x3 rhythm without inventing enum values that do not exist.
    let grid: [[Option<(Position, &str)>; 3]; 3] = [
        [
            Some((Position::TopLeft, "Kiri atas")),
            None,
            Some((Position::TopRight, "Kanan atas")),
        ],
        [None, Some((Position::Center, "Tengah")), None],
        [
            Some((Position::BottomLeft, "Kiri bawah")),
            None,
            Some((Position::BottomRight, "Kanan bawah")),
        ],
    ];

    for row in grid {
        ui.horizontal(|ui| {
            for cell in row {
                match cell {
                    Some((value, tip)) => {
                        let selected = *current == value;
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::splat(CELL), Sense::click());
                        let fill = if selected { accent_fill() } else { INPUT };
                        let stroke = pen(
                            1.0,
                            if selected { ACCENT } else { BORDER },
                        );
                        ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
                        ui.painter()
                            .rect_stroke(rect, Rounding::same(6.0), stroke);
                        let dot = rect.shrink(CELL * 0.36);
                        ui.painter().rect_filled(
                            dot,
                            Rounding::same(2.0),
                            if selected { ACCENT } else { MUTED },
                        );
                        if resp.on_hover_text(tip).clicked() {
                            chosen = Some(value);
                        }
                    }
                    None => {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(CELL), Sense::hover());
                        ui.painter().rect_stroke(
                            rect,
                            Rounding::same(6.0),
                            pen(1.0, BORDER.gamma_multiply(0.5)),
                        );
                    }
                }
            }
        });
    }
    if let Some(pos) = chosen {
        *current = pos;
    }
}

fn info_row(ui: &mut egui::Ui, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).size(10.8).color(MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(10.8).color(TEXT));
        });
    });
}

fn material_name(kind: MaterialKind) -> &'static str {
    match kind {
        MaterialKind::None => "None",
        MaterialKind::Solid => "Solid",
        MaterialKind::Glass => "Glass",
        MaterialKind::Acrylic => "Acrylic",
        MaterialKind::Mica => "Mica",
    }
}

fn material_hint(kind: MaterialKind) -> &'static str {
    match kind {
        MaterialKind::None => "Tanpa kartu: heatmap langsung menempel di desktop.",
        MaterialKind::Solid => "Kartu legap berwarna tint, tanpa blur backdrop.",
        MaterialKind::Glass => "Backdrop blur + tint tipis + garis tepi halus.",
        MaterialKind::Acrylic => "Glass dengan grain monokrom deterministik.",
        MaterialKind::Mica => "Blur ringan dan desaturasi, mirip Mica Windows 11.",
    }
}

fn position_name(pos: Position) -> &'static str {
    match pos {
        Position::Center => "Tengah",
        Position::TopLeft => "Kiri atas",
        Position::TopRight => "Kanan atas",
        Position::BottomLeft => "Kiri bawah",
        Position::BottomRight => "Kanan bawah",
        Position::Custom => "Custom",
    }
}


/// `Stroke::new` takes `impl Into<f32>`, so bare float literals default to
/// `f64` and warn. Funneling every stroke through an `f32` parameter keeps the
/// call sites suffix-free and the build warning-free.
fn pen(width: f32, color: Color32) -> Stroke {
    Stroke::new(width, color)
}
