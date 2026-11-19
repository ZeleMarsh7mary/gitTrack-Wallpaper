//! Wire format between the webview and Rust.
//!
//! The core config uses `snake_case` on disk; the UI schema is `camelCase` so
//! TypeScript stays idiomatic. The conversion lives here and nowhere else.

use serde::{Deserialize, Serialize};

use git_track_wallpaper_core::config as core;
use git_track_wallpaper_core::security;

// ------------------------------------------------------------------ enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceMode {
    Profile,
    Repository,
}

impl From<SourceMode> for core::SourceMode {
    fn from(v: SourceMode) -> Self {
        match v {
            SourceMode::Profile => core::SourceMode::Profile,
            SourceMode::Repository => core::SourceMode::Repository,
        }
    }
}

impl From<core::SourceMode> for SourceMode {
    fn from(v: core::SourceMode) -> Self {
        match v {
            core::SourceMode::Profile => SourceMode::Profile,
            core::SourceMode::Repository => SourceMode::Repository,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BgMode {
    Image,
    Solid,
    Gradient,
}

impl From<BgMode> for core::BgMode {
    fn from(v: BgMode) -> Self {
        match v {
            BgMode::Image => core::BgMode::Image,
            BgMode::Solid => core::BgMode::Solid,
            BgMode::Gradient => core::BgMode::Gradient,
        }
    }
}

impl From<core::BgMode> for BgMode {
    fn from(v: core::BgMode) -> Self {
        match v {
            core::BgMode::Image => BgMode::Image,
            core::BgMode::Solid => BgMode::Solid,
            core::BgMode::Gradient => BgMode::Gradient,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BgFit {
    Cover,
    Contain,
    Stretch,
}

impl From<BgFit> for core::BgFit {
    fn from(v: BgFit) -> Self {
        match v {
            BgFit::Cover => core::BgFit::Cover,
            BgFit::Contain => core::BgFit::Contain,
            BgFit::Stretch => core::BgFit::Stretch,
        }
    }
}

impl From<core::BgFit> for BgFit {
    fn from(v: core::BgFit) -> Self {
        match v {
            core::BgFit::Cover => BgFit::Cover,
            core::BgFit::Contain => BgFit::Contain,
            core::BgFit::Stretch => BgFit::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MaterialKind {
    None,
    Solid,
    Glass,
    Acrylic,
    Mica,
}

impl From<MaterialKind> for core::MaterialKind {
    fn from(v: MaterialKind) -> Self {
        match v {
            MaterialKind::None => core::MaterialKind::None,
            MaterialKind::Solid => core::MaterialKind::Solid,
            MaterialKind::Glass => core::MaterialKind::Glass,
            MaterialKind::Acrylic => core::MaterialKind::Acrylic,
            MaterialKind::Mica => core::MaterialKind::Mica,
        }
    }
}

impl From<core::MaterialKind> for MaterialKind {
    fn from(v: core::MaterialKind) -> Self {
        match v {
            core::MaterialKind::None => MaterialKind::None,
            core::MaterialKind::Solid => MaterialKind::Solid,
            core::MaterialKind::Glass => MaterialKind::Glass,
            core::MaterialKind::Acrylic => MaterialKind::Acrylic,
            core::MaterialKind::Mica => MaterialKind::Mica,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Position {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Custom,
}

impl From<Position> for core::Position {
    fn from(v: Position) -> Self {
        match v {
            Position::Center => core::Position::Center,
            Position::TopLeft => core::Position::TopLeft,
            Position::TopRight => core::Position::TopRight,
            Position::BottomLeft => core::Position::BottomLeft,
            Position::BottomRight => core::Position::BottomRight,
            Position::Custom => core::Position::Custom,
        }
    }
}

impl From<core::Position> for Position {
    fn from(v: core::Position) -> Self {
        match v {
            core::Position::Center => Position::Center,
            core::Position::TopLeft => Position::TopLeft,
            core::Position::TopRight => Position::TopRight,
            core::Position::BottomLeft => Position::BottomLeft,
            core::Position::BottomRight => Position::BottomRight,
            core::Position::Custom => Position::Custom,
        }
    }
}

// ----------------------------------------------------------------- structs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSource {
    pub mode: SourceMode,
    pub input_target: String,
    /// Plaintext in memory only; the core config stores the DPAPI blob.
    pub api_key: String,
    pub refresh_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiBackground {
    pub mode: BgMode,
    pub image_path: String,
    pub fit: BgFit,
    pub color_primary: String,
    pub color_secondary: String,
    /// Image framing, mirrored from `core::Background`.
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiCard {
    pub position: Position,
    pub offset_x: i32,
    pub offset_y: i32,
    pub material: MaterialKind,
    pub tint_color: String,
    pub opacity: f32,
    pub blur_radius: f32,
    pub corner_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTrack {
    pub box_size: f32,
    pub box_gap: f32,
    pub box_radius: f32,
    pub colors: [String; 5],
    pub show_stats_header: bool,
    pub text_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub source: UiSource,
    pub background: UiBackground,
    pub card: UiCard,
    pub track: UiTrack,
    pub start_with_windows: bool,
}

impl UiConfig {
    /// Config plus the decrypted token, ready for editing.
    pub fn from_core(cfg: &core::Config, api_key: String) -> Self {
        Self {
            source: UiSource {
                mode: cfg.source.mode.into(),
                input_target: cfg.source.input_target.clone(),
                api_key,
                refresh_interval_minutes: cfg.source.refresh_interval_minutes,
            },
            background: UiBackground {
                mode: cfg.background.mode.into(),
                image_path: cfg.background.image_path.clone(),
                fit: cfg.background.fit.into(),
                color_primary: cfg.background.color_primary.clone(),
                color_secondary: cfg.background.color_secondary.clone(),
                zoom: cfg.background.zoom,
                pan_x: cfg.background.pan_x,
                pan_y: cfg.background.pan_y,
            },
            card: UiCard {
                position: cfg.card.position.into(),
                offset_x: cfg.card.offset_x,
                offset_y: cfg.card.offset_y,
                material: cfg.card.material.into(),
                tint_color: cfg.card.tint_color.clone(),
                opacity: cfg.card.opacity,
                blur_radius: cfg.card.blur_radius,
                corner_radius: cfg.card.corner_radius,
            },
            track: UiTrack {
                box_size: cfg.track.box_size,
                box_gap: cfg.track.box_gap,
                box_radius: cfg.track.box_radius,
                colors: cfg.track.colors.clone(),
                show_stats_header: cfg.track.show_stats_header,
                text_color: cfg.track.text_color.clone(),
            },
            start_with_windows: cfg.start_with_windows,
        }
    }

    /// Structural conversion without touching DPAPI. The token is dropped: use
    /// [`UiConfig::to_core_with_secret`] before saving or testing.
    pub fn to_core(&self) -> core::Config {
        let mut cfg = core::Config {
            source: core::Source {
                mode: self.source.mode.into(),
                input_target: self.source.input_target.trim().to_string(),
                encrypted_api_key: String::new(),
                refresh_interval_minutes: self.source.refresh_interval_minutes,
            },
            background: core::Background {
                mode: self.background.mode.into(),
                image_path: self.background.image_path.clone(),
                fit: self.background.fit.into(),
                color_primary: self.background.color_primary.clone(),
                color_secondary: self.background.color_secondary.clone(),
                zoom: self.background.zoom,
                pan_x: self.background.pan_x,
                pan_y: self.background.pan_y,
            },
            card: core::Card {
                position: self.card.position.into(),
                offset_x: self.card.offset_x,
                offset_y: self.card.offset_y,
                material: self.card.material.into(),
                tint_color: self.card.tint_color.clone(),
                opacity: self.card.opacity,
                blur_radius: self.card.blur_radius,
                corner_radius: self.card.corner_radius,
            },
            track: core::Track {
                box_size: self.track.box_size,
                box_gap: self.track.box_gap,
                box_radius: self.track.box_radius,
                colors: self.track.colors.clone(),
                show_stats_header: self.track.show_stats_header,
                text_color: self.track.text_color.clone(),
            },
            start_with_windows: self.start_with_windows,
        };
        cfg.sanitize();
        cfg
    }

    /// Same as [`UiConfig::to_core`] but the token is DPAPI-encrypted first.
    pub fn to_core_with_secret(&self) -> Result<core::Config, String> {
        let mut cfg = self.to_core();
        cfg.source.encrypted_api_key = security::encrypt(self.source.api_key.trim())
            .map_err(|e| format!("Could not encrypt the token with DPAPI: {e}"))?;
        Ok(cfg)
    }
}

// --------------------------------------------------------------- responses

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub daemon_running: bool,
    pub autostart: bool,
    pub version: String,
    pub resolution: [u32; 2],
    pub config_path: String,
    pub cache_path: String,
    pub mica: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub config: UiConfig,
    pub status: AppStatus,
    /// Non-fatal problem, e.g. a `config.toml` that could not be parsed.
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStats {
    pub total: u32,
    pub best_day: u32,
    pub streak: u32,
    pub days: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub message: String,
    pub daemon_running: bool,
    pub autostart: bool,
}
