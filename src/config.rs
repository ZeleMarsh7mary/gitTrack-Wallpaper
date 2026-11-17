//! Configuration schema, defaults, validation and file locations.
//!
//! Stored at `%APPDATA%\GitWallpaper\config.toml`. The GitHub token is never
//! written in plaintext: [`crate::security`] wraps it with Windows DPAPI and the
//! base64 blob goes into `source.encrypted_api_key`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const APP_DIR: &str = "GitWallpaper";
pub const CACHE_VERSION: u32 = 1;

/// Built-in heatmap palettes: (name, [level0..level4]).
pub const PALETTES: &[(&str, [&str; 5])] = &[
    ("GitHub Dark", ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"]),
    ("GitHub Light", ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"]),
    ("Dracula", ["#21222c", "#3a3f58", "#6272a4", "#bd93f9", "#ff79c6"]),
    ("Monochrome", ["#1b1b1b", "#3d3d3d", "#6b6b6b", "#9d9d9d", "#f0f0f0"]),
    ("Neon", ["#101820", "#0b3d2e", "#128a5a", "#22d38a", "#8affc1"]),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMode {
    Profile,
    Repository,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BgMode {
    Image,
    Solid,
    Gradient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BgFit {
    Cover,
    Contain,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKind {
    None,
    Solid,
    Glass,
    Acrylic,
    Mica,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Source {
    pub mode: SourceMode,
    /// GitHub username (`profile`) or a repository URL (`repository`).
    pub input_target: String,
    /// Base64 of a DPAPI blob. Empty when unset.
    pub encrypted_api_key: String,
    pub refresh_interval_minutes: u32,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            mode: SourceMode::Profile,
            input_target: String::new(),
            encrypted_api_key: String::new(),
            refresh_interval_minutes: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Background {
    pub mode: BgMode,
    pub image_path: String,
    pub fit: BgFit,
    pub color_primary: String,
    pub color_secondary: String,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            mode: BgMode::Gradient,
            image_path: String::new(),
            fit: BgFit::Cover,
            color_primary: "#0b1220".into(),
            color_secondary: "#12261c".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Card {
    pub position: Position,
    pub offset_x: i32,
    pub offset_y: i32,
    pub material: MaterialKind,
    pub tint_color: String,
    pub opacity: f32,
    pub blur_radius: f32,
    pub corner_radius: f32,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            position: Position::Center,
            offset_x: 0,
            offset_y: 0,
            material: MaterialKind::Glass,
            tint_color: "#10161f".into(),
            opacity: 0.55,
            blur_radius: 24.0,
            corner_radius: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Track {
    pub box_size: f32,
    pub box_gap: f32,
    pub box_radius: f32,
    pub colors: [String; 5],
    pub show_stats_header: bool,
    pub text_color: String,
}

impl Default for Track {
    fn default() -> Self {
        let p = PALETTES[0].1;
        Self {
            box_size: 14.0,
            box_gap: 4.0,
            box_radius: 3.0,
            colors: [
                p[0].to_string(),
                p[1].to_string(),
                p[2].to_string(),
                p[3].to_string(),
                p[4].to_string(),
            ],
            show_stats_header: true,
            text_color: "#e6edf3".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Config {
    pub source: Source,
    pub background: Background,
    pub card: Card,
    pub track: Track,
    pub start_with_windows: bool,
}

impl Config {
    /// Clamp every user-editable number into its documented range.
    pub fn sanitize(&mut self) {
        self.source.refresh_interval_minutes =
            self.source.refresh_interval_minutes.clamp(5, 1440);
        self.card.opacity = self.card.opacity.clamp(0.0, 1.0);
        self.card.blur_radius = self.card.blur_radius.clamp(0.0, 50.0);
        self.card.corner_radius = self.card.corner_radius.clamp(0.0, 80.0);
        self.track.box_size = self.track.box_size.clamp(6.0, 40.0);
        self.track.box_gap = self.track.box_gap.clamp(0.0, 12.0);
        self.track.box_radius = self
            .track
            .box_radius
            .clamp(0.0, self.track.box_size / 2.0);
        for c in &mut self.track.colors {
            if parse_hex(c).is_none() {
                *c = "#161b22".into();
            }
        }
        for c in [
            &mut self.background.color_primary,
            &mut self.background.color_secondary,
            &mut self.card.tint_color,
            &mut self.track.text_color,
        ] {
            if parse_hex(c).is_none() {
                *c = "#101010".into();
            }
        }
    }

    pub fn load_or_default() -> (Self, Option<String>) {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(text) => match toml::from_str::<Config>(&text) {
                Ok(mut cfg) => {
                    cfg.sanitize();
                    (cfg, None)
                }
                Err(e) => (
                    Config::default(),
                    Some(format!("config.toml tidak terbaca: {e}")),
                ),
            },
            Err(_) => (Config::default(), None),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = appdata_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("buat {}: {e}", dir.display()))?;
        let text = toml::to_string_pretty(self).map_err(|e| format!("serialize config: {e}"))?;
        write_atomic(&config_path(), text.as_bytes())
    }
}

/// `#rgb`, `#rrggbb` or bare hex digits to a packed 0xRRGGBB value.
pub fn parse_hex(s: &str) -> Option<u32> {
    let t = s.trim().trim_start_matches('#');
    let expand = |c: char| -> Option<String> {
        let v = c.to_digit(16)?;
        Some(format!("{:x}{:x}", v, v))
    };
    let hex = match t.len() {
        3 => t.chars().map(expand).collect::<Option<String>>()?,
        6 => t.to_string(),
        _ => return None,
    };
    u32::from_str_radix(&hex, 16).ok()
}

pub fn to_hex(rgb: u32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb >> 16) & 0xff,
        (rgb >> 8) & 0xff,
        rgb & 0xff
    )
}

pub fn rgb(hex: &str, fallback: u32) -> (u8, u8, u8) {
    let v = parse_hex(hex).unwrap_or(fallback);
    (((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

fn env_dir(var: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn appdata_dir() -> PathBuf {
    env_dir("APPDATA").join(APP_DIR)
}

pub fn local_data_dir() -> PathBuf {
    env_dir("LOCALAPPDATA").join(APP_DIR)
}

pub fn config_path() -> PathBuf {
    appdata_dir().join("config.toml")
}

pub fn cache_path() -> PathBuf {
    local_data_dir().join("cache.json")
}

pub fn wallpaper_path() -> PathBuf {
    local_data_dir().join("current_wallpaper.png")
}

/// Write via a sibling temp file + rename so a crash never truncates the target.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("buat {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("data")
    ));
    fs::write(&tmp, bytes).map_err(|e| format!("tulis {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))
}
