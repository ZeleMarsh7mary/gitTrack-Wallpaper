//! `--settings` window: a short-lived eframe/egui process styled after the
//! Windows 11 Settings app (WinUI 3 Fluent dark).
//!
//! Shell, outside-in: 220 px navigation rail -> sticky 56 px footer ->
//! scrollable 28 px content column with a page header, a compact desktop
//! preview, and stacked `SettingsRow`s. The process exits completely when the
//! window closes, so none of this GUI memory stays resident in the daemon.

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
use crate::renderer::{render, Scene};
use crate::{security, win_api};

const WINDOW_TITLE: &str = "GitWallpaper Settings";
const WINDOW_BG: u32 = 0x20_20_20;

// ------------------------------------------------------------- WinUI 3 tokens

const BG: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
const CARD: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x2B);
const CARD_HOVER: Color32 = Color32::from_rgb(0x32, 0x32, 0x32);
const INPUT: Color32 = Color32::from_rgb(0x22, 0x22, 0x22);
const SECONDARY_BG: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
const SECONDARY_BORDER: Color32 = Color32::from_rgb(0x45, 0x45, 0x45);
const NAV_ACTIVE: Color32 = Color32::from_rgb(0x2D, 0x2D, 0x2D);
const ACCENT: Color32 = Color32::from_rgb(0x2E, 0xA0, 0x43);
const TEXT: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF5);
const MUTED: Color32 = Color32::from_rgb(0x9E, 0x9E, 0x9E);
const DANGER: Color32 = Color32::from_rgb(0xE5, 0x53, 0x4B);

const NAV_WIDTH: f32 = 220.0;
const PAGE_PAD_X: f32 = 28.0;
const ROW_PAD_X: f32 = 16.0;
const ROW_PAD_Y: f32 = 12.0;
const CONTROL_H: f32 = 30.0;
const HERO_W: f32 = 320.0;
const HERO_H: f32 = 180.0;

fn hair() -> Color32 {
    Color32::from_white_alpha(14)
}

fn hair_strong() -> Color32 {
    Color32::from_white_alpha(26)
}

fn accent_soft() -> Color32 {
    Color32::from_rgba_unmultiplied(0x2E, 0xA0, 0x43, 48)
}

fn pen(width: f32, color: Color32) -> Stroke {
    Stroke::new(width, color)
}

// ------------------------------------------------------------------- theme

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
    // Phosphor lands at index 1 so text glyphs still come from Segoe UI.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();

    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = INPUT;
    v.faint_bg_color = CARD;
    v.override_text_color = Some(TEXT);
    v.window_stroke = pen(1.0, hair());
    v.window_rounding = Rounding::same(8.0);
    v.menu_rounding = Rounding::same(4.0);
    v.handle_shape = egui::style::HandleShape::Circle;
    v.selection = egui::style::Selection {
        bg_fill: accent_soft(),
        stroke: pen(1.0, ACCENT),
    };

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = CARD;
    w.noninteractive.weak_bg_fill = CARD;
    w.noninteractive.bg_stroke = pen(1.0, hair());
    w.noninteractive.fg_stroke = pen(1.0, MUTED);
    w.noninteractive.rounding = Rounding::same(4.0);

    w.inactive.bg_fill = INPUT;
    w.inactive.weak_bg_fill = CARD;
    w.inactive.bg_stroke = pen(1.0, hair());
    w.inactive.fg_stroke = pen(1.0, TEXT);
    w.inactive.rounding = Rounding::same(4.0);

    w.hovered.bg_fill = CARD_HOVER;
    w.hovered.weak_bg_fill = CARD_HOVER;
    w.hovered.bg_stroke = pen(1.0, hair_strong());
    w.hovered.fg_stroke = pen(1.4, TEXT);
    w.hovered.rounding = Rounding::same(4.0);

    w.active.bg_fill = INPUT;
    w.active.weak_bg_fill = INPUT;
    w.active.bg_stroke = pen(1.0, ACCENT);
    w.active.fg_stroke = pen(1.4, TEXT);
    w.active.rounding = Rounding::same(4.0);

    w.open.bg_fill = CARD_HOVER;
    w.open.weak_bg_fill = CARD_HOVER;
    w.open.bg_stroke = pen(1.0, hair_strong());
    w.open.fg_stroke = pen(1.0, TEXT);
    w.open.rounding = Rounding::same(4.0);

    style.visuals = v;

    let sp = &mut style.spacing;
    sp.item_spacing = Vec2::new(10.0, 6.0);
    sp.button_padding = Vec2::new(12.0, 5.0);
    sp.interact_size = Vec2::new(44.0, CONTROL_H);
    sp.slider_width = 170.0;
    sp.slider_rail_height = 4.0;
    sp.indent = 16.0;
    sp.scroll.floating = false;
    sp.scroll.bar_width = 8.0;

    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(20.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.0, FontFamily::Monospace),
        ),
    ]
    .into();

    ctx.set_style(style);
}

// ------------------------------------------------------------- entry point

pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([780.0, 520.0])
            .with_resizable(true)
            .with_title(WINDOW_TITLE),
        ..Default::default()
    };
    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            let app = SettingsApp::new(cc);
            // Blend the native caption into the app background (Windows 11).
            let _ = win_api::apply_dark_titlebar(WINDOW_TITLE, WINDOW_BG);
            Ok(Box::new(app))
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
    const ALL: [(Tab, &'static str, &'static str, &'static str); 4] = [
        (Tab::Source, icons::GIT_BRANCH, "Git Source", "Git Source"),
        (Tab::Background, icons::IMAGE, "Background", "Background"),
        (
            Tab::Material,
            icons::SPARKLE,
            "Material & Blur",
            "Material & Blur",
        ),
        (
            Tab::Track,
            icons::GRID_FOUR,
            "Track & Boxes",
            "Track & Boxes",
        ),
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
        cfg.start_with_windows = cfg.start_with_windows || win_api::is_start_with_windows();
        let api_key = security::decrypt(&cfg.source.encrypted_api_key).unwrap_or_default();
        let days = git_fetcher::load_cache().unwrap_or_else(git_fetcher::demo_days);
        let toast = warn.map(|text| Toast {
            text,
            kind: ToastKind::Error,
            until: Instant::now() + Duration::from_secs(8),
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
            until: Instant::now() + Duration::from_secs(6),
        });
    }

    fn preview(&mut self, ctx: &egui::Context) {
        let (sw, sh) = self.resolution;
        let scale = HERO_W / sw as f32;
        let height = (sh as f32 * scale).round().max(90.0) as u32;
        let scene = Scene {
            config: &self.cfg,
            days: &self.days,
            size_px: (HERO_W as u32, height),
            scale,
        };
        let Ok(pm) = render(&scene) else {
            return;
        };
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [pm.width() as usize, pm.height() as usize],
            pm.data(),
        );
        self.preview = Some(ctx.load_texture("preview", image, egui::TextureOptions::LINEAR));
        self.last_rendered = Some(self.cfg.clone());
    }

    // -------------------------------------------------------------- actions

    fn save_and_apply(&mut self) {
        if self.cfg.source.input_target.trim().is_empty() {
            self.notify(
                ToastKind::Info,
                "Nothing to track yet: enter a GitHub username or repository URL.",
            );
        }
        match security::encrypt(&self.api_key) {
            Ok(blob) => self.cfg.source.encrypted_api_key = blob,
            Err(e) => {
                self.notify(
                    ToastKind::Error,
                    format!("Could not encrypt the token: {e}"),
                );
                return;
            }
        }
        self.cfg.sanitize();
        if let Err(e) = self.cfg.save() {
            self.notify(ToastKind::Error, format!("Could not save config: {e}"));
            return;
        }
        if let Ok(exe) = std::env::current_exe() {
            let _ = win_api::set_start_with_windows(&exe, self.cfg.start_with_windows);
        }
        let autostart = win_api::is_start_with_windows();
        let signalled = win_api::signal_refresh_event();
        let mut note = if signalled {
            "Saved. The daemon is refreshing the wallpaper.".to_string()
        } else if std::env::current_exe()
            .map(|exe| std::process::Command::new(exe).spawn().is_ok())
            .unwrap_or(false)
        {
            "Saved. The daemon was not running, so it was started.".to_string()
        } else {
            "Saved. The daemon is not running; start git-wallpaper.exe.".to_string()
        };
        note.push_str(if autostart {
            " Autostart is on."
        } else {
            " Autostart is off."
        });
        self.notify(ToastKind::Success, note);
    }

    fn test_connection(&mut self) {
        let mut probe = self.cfg.clone();
        match security::encrypt(&self.api_key) {
            Ok(blob) => probe.source.encrypted_api_key = blob,
            Err(e) => {
                self.notify(
                    ToastKind::Error,
                    format!("Could not encrypt the token: {e}"),
                );
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
                    format!("Connection OK: {total} contributions in the last year."),
                );
            }
            Err(e) => self.notify(ToastKind::Error, format!("Connection failed: {e}")),
        }
    }

    // ---------------------------------------------------------------- shell

    fn draw_sidebar(&mut self, ctx: &egui::Context) {
        let daemon_up = win_api::daemon_event_exists();
        let autostart = win_api::is_start_with_windows();
        let mut tab = self.tab;

        egui::SidePanel::left("nav")
            .exact_width(NAV_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(12.0, 14.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icons::PALETTE).size(18.0).color(ACCENT));
                    ui.add_space(4.0);
                    ui.label(RichText::new("GitWallpaper").size(14.0).color(TEXT));
                });
                ui.add_space(18.0);

                for (value, icon, label, _) in Tab::ALL {
                    nav_item(ui, value, icon, label, &mut tab);
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    let status = format!(
                        "Daemon: {}  ·  Autostart: {}  ·  v{}",
                        if daemon_up { "Running" } else { "Stopped" },
                        if autostart { "On" } else { "Off" },
                        env!("CARGO_PKG_VERSION")
                    );
                    ui.add(egui::Label::new(RichText::new(status).size(11.0).color(MUTED)).wrap());
                });
            });

        self.tab = tab;
    }

    fn draw_footer(&mut self, ctx: &egui::Context) {
        let mut save = false;
        let mut test = false;

        egui::TopBottomPanel::bottom("footer")
            .exact_height(56.0)
            .show_separator_line(false)
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(20.0, 10.0)),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                ui.painter()
                    .hline(rect.x_range(), rect.top(), pen(1.0, hair_strong()));

                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        match &self.toast {
                            Some(toast) => {
                                let (color, icon) = toast_style(toast.kind);
                                ui.label(RichText::new(icon).size(14.0).color(color));
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&toast.text).size(12.0).color(TEXT),
                                    )
                                    .truncate(),
                                );
                            }
                            None => {
                                ui.label(RichText::new("Ready").size(12.0).color(MUTED));
                            }
                        }
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        save = ui
                            .add_sized(
                                [128.0, CONTROL_H],
                                Button::new(
                                    RichText::new("Save & Apply")
                                        .size(13.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(ACCENT)
                                .stroke(Stroke::NONE)
                                .rounding(Rounding::same(4.0)),
                            )
                            .clicked();
                        ui.add_space(8.0);
                        test = ui
                            .add_sized(
                                [128.0, CONTROL_H],
                                Button::new(RichText::new("Test Connection").size(13.0))
                                    .fill(SECONDARY_BG)
                                    .stroke(pen(1.0, SECONDARY_BORDER))
                                    .rounding(Rounding::same(4.0)),
                            )
                            .clicked();
                    });
                });
            });

        if save {
            self.save_and_apply();
        }
        if test {
            self.test_connection();
        }
    }

    fn draw_content(&mut self, ctx: &egui::Context) {
        let page_title = match self.tab {
            Tab::Source => "Git Source",
            Tab::Background => "Background",
            Tab::Material => "Material & Blur",
            Tab::Track => "Track & Boxes",
        };

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(PAGE_PAD_X, 20.0)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(page_title).size(26.0).strong().color(TEXT));
                        ui.add_space(14.0);
                        self.hero(ui);
                        ui.add_space(18.0);
                        match self.tab {
                            Tab::Source => self.tab_source(ui),
                            Tab::Background => self.tab_background(ui),
                            Tab::Material => self.tab_material(ui),
                            Tab::Track => self.tab_track(ui),
                        }
                    });
            });
    }

    fn hero(&self, ui: &mut egui::Ui) {
        Frame::none()
            .fill(Color32::from_rgb(0x17, 0x17, 0x17))
            .stroke(pen(1.0, hair()))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| match &self.preview {
                Some(tex) => {
                    let size = tex.size_vec2();
                    ui.add(
                        egui::Image::new(egui::load::SizedTexture::new(tex.id(), size))
                            .fit_to_exact_size(Vec2::new(HERO_W, HERO_H))
                            .rounding(6.0),
                    );
                }
                None => {
                    ui.allocate_space(Vec2::new(HERO_W, HERO_H));
                }
            });
    }

    // ----------------------------------------------------------------- tabs

    fn tab_source(&mut self, ui: &mut egui::Ui) {
        setting_row(
            ui,
            icons::GIT_BRANCH,
            "Data source",
            "Profile contribution graph, or the commits of one repository.",
            |ui| {
                segmented(
                    ui,
                    &mut self.cfg.source.mode,
                    &[
                        (SourceMode::Profile, "Profile"),
                        (SourceMode::Repository, "Repository"),
                    ],
                );
            },
        );

        let (title, desc, hint) = match self.cfg.source.mode {
            SourceMode::Profile => (
                "GitHub username",
                "Contributions of this account over the last year (GraphQL, token required).",
                "octocat",
            ),
            SourceMode::Repository => (
                "Repository URL",
                "Commit history of this repository. Private repos need a token with Contents: Read.",
                "https://github.com/owner/repo",
            ),
        };
        setting_row_stacked(ui, icons::USER, title, desc, |ui| {
            text_field(ui, &mut self.cfg.source.input_target, hint);
        });

        setting_row_stacked(
            ui,
            icons::KEY,
            "Personal access token",
            "Encrypted with DPAPI and bound to this Windows account. Never written in plaintext.",
            |ui| {
                ui.horizontal(|ui| {
                    let toggle = CONTROL_H;
                    let field_w = (ui.available_width() - toggle - 8.0).max(120.0);
                    ui.add_sized(
                        [field_w, CONTROL_H],
                        egui::TextEdit::singleline(&mut self.api_key)
                            .password(!self.show_token)
                            .hint_text("ghp_... or github_pat_...")
                            .margin(Margin::symmetric(8.0, 5.0)),
                    );
                    let (icon, tip) = if self.show_token {
                        (icons::EYE_SLASH, "Hide token")
                    } else {
                        (icons::EYE, "Show token")
                    };
                    if ui
                        .add_sized(
                            [toggle, CONTROL_H],
                            Button::new(RichText::new(icon).size(15.0))
                                .fill(INPUT)
                                .stroke(pen(1.0, hair()))
                                .rounding(Rounding::same(4.0)),
                        )
                        .on_hover_text(tip)
                        .clicked()
                    {
                        self.show_token = !self.show_token;
                    }
                });
            },
        );

        setting_row(
            ui,
            icons::CLOCK,
            "Refresh interval",
            "How often the daemon fetches new data.",
            |ui| {
                let mut minutes = self.cfg.source.refresh_interval_minutes as f32;
                slider_with_badge(ui, &mut minutes, 5.0..=240.0, " min", 0);
                self.cfg.source.refresh_interval_minutes = minutes.round().clamp(5.0, 240.0) as u32;
            },
        );
    }

    fn tab_background(&mut self, ui: &mut egui::Ui) {
        setting_row(
            ui,
            icons::IMAGE,
            "Background mode",
            "Desktop background behind the tracker card.",
            |ui| {
                segmented(
                    ui,
                    &mut self.cfg.background.mode,
                    &[
                        (BgMode::Image, "Image"),
                        (BgMode::Solid, "Solid"),
                        (BgMode::Gradient, "Gradient"),
                    ],
                );
            },
        );

        setting_row_stacked(
            ui,
            icons::FOLDER_OPEN,
            "Image file",
            "Used when the mode is Image.",
            |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [112.0, CONTROL_H],
                            Button::new(RichText::new("Browse...").size(13.0))
                                .fill(SECONDARY_BG)
                                .stroke(pen(1.0, SECONDARY_BORDER))
                                .rounding(Rounding::same(4.0)),
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
                        "No image selected".to_string()
                    } else {
                        self.cfg.background.image_path.clone()
                    };
                    ui.add(
                        egui::Label::new(RichText::new(shown).size(12.0).color(MUTED)).truncate(),
                    );
                });
            },
        );

        setting_row(
            ui,
            icons::ARROWS_OUT_SIMPLE,
            "Fit",
            "How the image fills the screen.",
            |ui| {
                dropdown(
                    ui,
                    "fit",
                    &mut self.cfg.background.fit,
                    &[
                        (BgFit::Cover, "Cover"),
                        (BgFit::Contain, "Contain"),
                        (BgFit::Stretch, "Stretch"),
                    ],
                );
            },
        );

        setting_row(
            ui,
            icons::PALETTE,
            "Primary color",
            "Used by Solid, Gradient, and empty areas in Contain mode.",
            |ui| {
                color_control(ui, &mut self.cfg.background.color_primary);
            },
        );

        setting_row(
            ui,
            icons::PALETTE,
            "Secondary color",
            "Gradient end color.",
            |ui| {
                color_control(ui, &mut self.cfg.background.color_secondary);
            },
        );
    }

    fn tab_material(&mut self, ui: &mut egui::Ui) {
        let hint = material_hint(self.cfg.card.material);
        setting_row(ui, icons::SPARKLE, "Material", hint, |ui| {
            dropdown(
                ui,
                "material",
                &mut self.cfg.card.material,
                &[
                    (MaterialKind::None, "None"),
                    (MaterialKind::Solid, "Solid"),
                    (MaterialKind::Glass, "Glass"),
                    (MaterialKind::Acrylic, "Acrylic"),
                    (MaterialKind::Mica, "Mica"),
                ],
            );
        });

        setting_row(
            ui,
            icons::CIRCLE_HALF,
            "Opacity",
            "Card tint strength over the backdrop.",
            |ui| {
                slider_with_badge(ui, &mut self.cfg.card.opacity, 0.0..=1.0, "", 2);
            },
        );

        setting_row(
            ui,
            icons::DROP,
            "Blur radius",
            "Backdrop blur behind the card only.",
            |ui| {
                slider_with_badge(ui, &mut self.cfg.card.blur_radius, 0.0..=50.0, " px", 0);
            },
        );

        setting_row(
            ui,
            icons::SQUARE,
            "Card corner radius",
            "Rounded corners of the card.",
            |ui| {
                slider_with_badge(ui, &mut self.cfg.card.corner_radius, 0.0..=80.0, " px", 0);
            },
        );

        setting_row(
            ui,
            icons::PALETTE,
            "Card tint",
            "Base color blended over the blurred backdrop.",
            |ui| {
                color_control(ui, &mut self.cfg.card.tint_color);
            },
        );

        setting_row(
            ui,
            icons::CROSSHAIR,
            "Position",
            "Where the card sits on the desktop.",
            |ui| {
                dropdown(
                    ui,
                    "position",
                    &mut self.cfg.card.position,
                    &[
                        (Position::Center, "Center"),
                        (Position::TopLeft, "Top left"),
                        (Position::TopRight, "Top right"),
                        (Position::BottomLeft, "Bottom left"),
                        (Position::BottomRight, "Bottom right"),
                        (Position::Custom, "Custom offset"),
                    ],
                );
            },
        );

        if self.cfg.card.position == Position::Custom {
            setting_row(
                ui,
                icons::ARROWS_HORIZONTAL,
                "Offset X",
                "Horizontal shift from the center, in pixels.",
                |ui| {
                    ui.add_sized(
                        [170.0, CONTROL_H],
                        egui::Slider::new(&mut self.cfg.card.offset_x, -1200..=1200),
                    );
                },
            );
            setting_row(
                ui,
                icons::ARROWS_VERTICAL,
                "Offset Y",
                "Vertical shift from the center, in pixels.",
                |ui| {
                    ui.add_sized(
                        [170.0, CONTROL_H],
                        egui::Slider::new(&mut self.cfg.card.offset_y, -800..=800),
                    );
                },
            );
        }
    }

    fn tab_track(&mut self, ui: &mut egui::Ui) {
        setting_row(
            ui,
            icons::PALETTE,
            "Palette preset",
            "Five-level color ramp for the heatmap.",
            |ui| {
                preset_control(ui, &mut self.cfg.track.colors);
            },
        );

        let mut colors = self.cfg.track.colors.clone();
        for (i, slot) in colors.iter_mut().enumerate() {
            let label = if i == 0 {
                "Level 0 color".to_string()
            } else {
                format!("Level {i} color")
            };
            let desc = if i == 0 {
                "No contributions."
            } else if i == 4 {
                "Densest day."
            } else {
                "Intermediate intensity."
            };
            setting_row(ui, icons::SQUARE, &label, desc, |ui| {
                color_control(ui, slot);
            });
        }
        if colors != self.cfg.track.colors {
            self.cfg.track.colors = colors;
        }

        setting_row(
            ui,
            icons::GRID_FOUR,
            "Box size",
            "Heatmap cell size in pixels.",
            |ui| {
                slider_with_badge(ui, &mut self.cfg.track.box_size, 6.0..=40.0, " px", 1);
            },
        );
        setting_row(
            ui,
            icons::GRID_FOUR,
            "Box gap",
            "Spacing between cells.",
            |ui| {
                slider_with_badge(ui, &mut self.cfg.track.box_gap, 0.0..=12.0, " px", 1);
            },
        );
        setting_row(
            ui,
            icons::SQUARE,
            "Box corner radius",
            "0 gives sharp squares, half the box size gives circles.",
            |ui| {
                slider_with_badge(ui, &mut self.cfg.track.box_radius, 0.0..=20.0, " px", 1);
            },
        );

        setting_row(
            ui,
            icons::TEXT_T,
            "Stats header",
            "Show total contributions, streak, and best day above the grid.",
            |ui| {
                ui.add(egui::Checkbox::without_text(
                    &mut self.cfg.track.show_stats_header,
                ));
            },
        );

        setting_row(
            ui,
            icons::PALETTE,
            "Header text color",
            "Used by the stats line only.",
            |ui| {
                color_control(ui, &mut self.cfg.track.text_color);
            },
        );

        setting_row(
            ui,
            icons::POWER,
            "Start with Windows",
            "Adds a per-user Run entry; no administrator rights required.",
            |ui| {
                ui.add(egui::Checkbox::without_text(
                    &mut self.cfg.start_with_windows,
                ));
            },
        );

        setting_row(
            ui,
            icons::FOLDER_SIMPLE,
            "Files",
            "Config and cache locations.",
            |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(config::config_path().display().to_string())
                            .size(11.0)
                            .color(MUTED)
                            .monospace(),
                    );
                    ui.label(
                        RichText::new(config::cache_path().display().to_string())
                            .size(11.0)
                            .color(MUTED)
                            .monospace(),
                    );
                });
            },
        );
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_rendered.as_ref() != Some(&self.cfg) {
            self.preview(ctx);
        }
        self.draw_sidebar(ctx);
        self.draw_footer(ctx);
        self.draw_content(ctx);

        if let Some(toast) = &self.toast {
            let remaining = toast.until.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                self.toast = None;
            } else {
                ctx.request_repaint_after(remaining);
            }
        }
    }
}

// ------------------------------------------------------------------ widgets

fn toast_style(kind: ToastKind) -> (Color32, &'static str) {
    match kind {
        ToastKind::Success => (ACCENT, icons::CHECK_CIRCLE),
        ToastKind::Error => (DANGER, icons::WARNING),
        ToastKind::Info => (MUTED, icons::INFO),
    }
}

fn nav_item(ui: &mut egui::Ui, value: Tab, icon: &str, label: &str, current: &mut Tab) {
    let selected = *current == value;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 36.0), Sense::click());
    let hovered = response.hovered();

    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, Rounding::same(4.0), NAV_ACTIVE);
        // 3 x 16 accent pill on the far-left edge.
        let pill = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 2.0, rect.center().y - 8.0),
            Vec2::new(3.0, 16.0),
        );
        painter.rect_filled(pill, Rounding::same(1.5), ACCENT);
    } else if hovered {
        painter.rect_filled(rect, Rounding::same(4.0), CARD_HOVER);
    }

    let text_color = if selected { TEXT } else { MUTED };
    let text_pos = egui::pos2(rect.left() + 16.0, rect.center().y);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        icon,
        FontId::new(15.0, FontFamily::Proportional),
        text_color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 40.0, text_pos.y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::new(14.0, FontFamily::Proportional),
        text_color,
    );

    if response.clicked() {
        *current = value;
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand);
    ui.add_space(4.0);
}

fn setting_row(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    desc: &str,
    control: impl FnOnce(&mut egui::Ui),
) {
    let response = Frame::none()
        .fill(CARD)
        .stroke(pen(1.0, hair()))
        .rounding(Rounding::same(5.0))
        .inner_margin(Margin::symmetric(ROW_PAD_X, ROW_PAD_Y))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(16.0).color(MUTED));
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(14.0).color(TEXT));
                    if !desc.is_empty() {
                        ui.add(
                            egui::Label::new(RichText::new(desc).size(12.0).color(MUTED)).wrap(),
                        );
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), control);
            });
        })
        .response;

    if response.hovered() {
        ui.painter()
            .rect_stroke(response.rect, Rounding::same(5.0), pen(1.0, hair_strong()));
    }
    ui.add_space(4.0);
}

fn setting_row_stacked(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    desc: &str,
    body: impl FnOnce(&mut egui::Ui),
) {
    let response = Frame::none()
        .fill(CARD)
        .stroke(pen(1.0, hair()))
        .rounding(Rounding::same(5.0))
        .inner_margin(Margin::symmetric(ROW_PAD_X, ROW_PAD_Y))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(16.0).color(MUTED));
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(14.0).color(TEXT));
                    if !desc.is_empty() {
                        ui.add(
                            egui::Label::new(RichText::new(desc).size(12.0).color(MUTED)).wrap(),
                        );
                    }
                });
            });
            ui.add_space(10.0);
            body(ui);
        })
        .response;

    if response.hovered() {
        ui.painter()
            .rect_stroke(response.rect, Rounding::same(5.0), pen(1.0, hair_strong()));
    }
    ui.add_space(4.0);
}

fn text_field(ui: &mut egui::Ui, text: &mut String, hint: &str) -> egui::Response {
    let response = ui.add_sized(
        [ui.available_width(), CONTROL_H],
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .margin(Margin::symmetric(8.0, 5.0)),
    );
    if response.has_focus() {
        // WinUI text box: 2 px accent underline while focused.
        let r = response.rect;
        let line = egui::Rect::from_min_max(
            egui::pos2(r.left(), r.bottom() - 2.0),
            egui::pos2(r.right(), r.bottom()),
        );
        ui.painter().rect_filled(line, Rounding::same(1.0), ACCENT);
    }
    response
}

fn segmented<T: Copy + PartialEq>(ui: &mut egui::Ui, current: &mut T, options: &[(T, &str)]) {
    Frame::none()
        .fill(INPUT)
        .stroke(pen(1.0, hair()))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::same(3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (value, label) in options {
                    let selected = *current == *value;
                    let button = Button::new(RichText::new(*label).size(12.0).color(if selected {
                        TEXT
                    } else {
                        MUTED
                    }))
                    .fill(if selected {
                        SECONDARY_BG
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(Stroke::NONE)
                    .rounding(Rounding::same(3.0))
                    .min_size(Vec2::new(0.0, 24.0));
                    if ui.add(button).clicked() && !selected {
                        *current = *value;
                    }
                }
            });
        });
}

fn dropdown<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    current: &mut T,
    options: &[(T, &str)],
) {
    let selected_text = options
        .iter()
        .find(|(value, _)| value == current)
        .map(|(_, label)| *label)
        .unwrap_or("");
    egui::ComboBox::from_id_salt(id)
        .width(160.0)
        .selected_text(RichText::new(selected_text).size(12.5).color(TEXT))
        .show_ui(ui, |ui| {
            for (value, label) in options {
                if ui
                    .selectable_label(*current == *value, RichText::new(*label).size(12.5))
                    .clicked()
                {
                    *current = *value;
                }
            }
        });
}

fn slider_with_badge(
    ui: &mut egui::Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    decimals: usize,
) {
    let text = format!("{value:.*}{suffix}", decimals);
    // right-to-left: badge first (rightmost), then the slider to its left.
    Frame::none()
        .fill(INPUT)
        .stroke(pen(1.0, hair()))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).color(TEXT).monospace());
        });
    ui.add_sized(
        [170.0, 26.0],
        egui::Slider::new(value, range)
            .show_value(false)
            .trailing_fill(true),
    );
}

fn color_control(ui: &mut egui::Ui, hex: &mut String) {
    let mut value = parse_hex(hex).unwrap_or(0x101010);
    let mut srgb = [
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
    ];
    if ui
        .add_sized([34.0, CONTROL_H], ColorButton::new(&mut srgb))
        .on_hover_text(hex.clone())
        .changed()
    {
        value = ((srgb[0] as u32) << 16) | ((srgb[1] as u32) << 8) | srgb[2] as u32;
        *hex = to_hex(value);
    }
    ui.label(
        RichText::new(hex.clone())
            .size(11.5)
            .color(MUTED)
            .monospace(),
    );
}

fn preset_control(ui: &mut egui::Ui, colors: &mut [String; 5]) {
    let current = PALETTES
        .iter()
        .find(|(_, preset)| {
            colors
                .iter()
                .zip(preset.iter())
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
        })
        .map(|(name, _)| *name)
        .unwrap_or("Custom");

    egui::ComboBox::from_id_salt("palette")
        .width(130.0)
        .selected_text(RichText::new(current).size(12.5).color(TEXT))
        .show_ui(ui, |ui| {
            for (name, preset) in PALETTES {
                if ui
                    .selectable_label(current == *name, RichText::new(*name).size(12.5))
                    .clicked()
                {
                    for (slot, hex) in colors.iter_mut().zip(preset.iter()) {
                        *slot = (*hex).to_string();
                    }
                }
            }
        });

    // Current ramp swatches, drawn left of the dropdown.
    for hex in colors.iter().rev() {
        let (r, g, b) = config::rgb(hex, 0x161b22);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, Rounding::same(3.0), Color32::from_rgb(r, g, b));
    }
}

fn material_hint(kind: MaterialKind) -> &'static str {
    match kind {
        MaterialKind::None => "No card: the heatmap sits directly on the wallpaper.",
        MaterialKind::Solid => "Opaque tinted card, no backdrop blur.",
        MaterialKind::Glass => "Backdrop blur, thin tint, and a soft 1 px edge.",
        MaterialKind::Mica => "Light blur and desaturation, close to Windows 11 Mica.",
        MaterialKind::Acrylic => "Glass plus deterministic monochrome grain.",
    }
}

/// Small square color button; wraps `ui.color_edit_button_srgb` with a fixed
/// size so rows line up.
struct ColorButton<'a> {
    srgb: &'a mut [u8; 3],
}

impl<'a> ColorButton<'a> {
    fn new(srgb: &'a mut [u8; 3]) -> Self {
        Self { srgb }
    }
}

impl egui::Widget for ColorButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.color_edit_button_srgb(self.srgb)
    }
}
