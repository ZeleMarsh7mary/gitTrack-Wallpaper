//! Tauri commands. Every piece of business logic delegates to `git-track-wallpaper-core`
//! so the settings window can never drift from what the daemon renders.

use std::path::PathBuf;
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::State;

use git_track_wallpaper_core::config::{self as core_config, Config};
use git_track_wallpaper_core::git_fetcher::{self, DayStat};
use git_track_wallpaper_core::renderer::{self, Scene};
use git_track_wallpaper_core::security;
use git_track_wallpaper_core::win_api;

use crate::ui_config::{AppStatus, ApplyResult, Bootstrap, ConnectionStats, UiConfig};

/// Physical width of a preview render. The UI shows it at roughly half that, so
/// it stays sharp on a 200% display.
const PREVIEW_W: f32 = 960.0;

/// Year of contribution data shared by every preview render in this window.
pub struct AppState {
    days: Mutex<Vec<DayStat>>,
}

impl AppState {
    pub fn new() -> Self {
        // Fresh data when the daemon already fetched some, else the deterministic
        // demo year so the preview is never an empty grid.
        let days = git_fetcher::load_cache().unwrap_or_else(git_fetcher::demo_days);
        Self {
            days: Mutex::new(days),
        }
    }

    fn snapshot(&self) -> Vec<DayStat> {
        self.days
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn replace(&self, days: Vec<DayStat>) {
        if let Ok(mut guard) = self.days.lock() {
            *guard = days;
        }
    }
}

fn daemon_exe() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidate = dir.join("git-track-wallpaper.exe");
    candidate.exists().then_some(candidate)
}

fn render_data_url(cfg: &Config, days: &[DayStat]) -> Result<String, String> {
    let (sw, sh) = win_api::primary_monitor_size();
    let scale = PREVIEW_W / sw.max(1) as f32;
    let height = (sh.max(1) as f32 * scale).round().max(90.0) as u32;
    let scene = Scene {
        config: cfg,
        days,
        size_px: (PREVIEW_W as u32, height),
        scale,
    };
    let png = renderer::render(&scene)?
        .encode_png()
        .map_err(|e| format!("encode_png: {e}"))?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

// ---------------------------------------------------------------- commands

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    let (cfg, warning) = Config::load_or_default();
    let api_key = security::decrypt(&cfg.source.encrypted_api_key).unwrap_or_default();
    let (w, h) = win_api::primary_monitor_size();
    let status = AppStatus {
        daemon_running: win_api::daemon_event_exists(),
        autostart: win_api::is_start_with_windows(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        resolution: [w, h],
        config_path: core_config::config_path().display().to_string(),
        cache_path: core_config::cache_path().display().to_string(),
        mica: false,
    };
    // Warm the cache-backed state once so the first preview has data even when
    // the daemon has never run in this session.
    if state.snapshot().is_empty() {
        state.replace(git_fetcher::demo_days());
    }
    Ok(Bootstrap {
        config: UiConfig::from_core(&cfg, api_key),
        status,
        warning,
    })
}

#[tauri::command]
pub async fn render_preview(
    config: UiConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let cfg = config.to_core();
    let days = state.snapshot();
    tauri::async_runtime::spawn_blocking(move || render_data_url(&cfg, &days))
        .await
        .map_err(|e| format!("preview task failed: {e}"))?
}

#[tauri::command]
pub async fn test_connection(
    config: UiConfig,
    state: State<'_, AppState>,
) -> Result<ConnectionStats, String> {
    let probe = config.to_core_with_secret()?;
    let days = tauri::async_runtime::spawn_blocking(move || git_fetcher::fetch(&probe))
        .await
        .map_err(|e| format!("fetch task failed: {e}"))??;
    let stats = renderer::stats_of(&days);
    let count = days.len() as u32;
    let _ = git_fetcher::save_cache(&days);
    state.replace(days);
    Ok(ConnectionStats {
        total: stats.total,
        best_day: stats.best_day,
        streak: stats.streak,
        days: count,
    })
}

#[tauri::command]
pub fn save_and_apply(config: UiConfig) -> Result<ApplyResult, String> {
    let mut cfg = config.to_core_with_secret()?;
    cfg.sanitize();
    cfg.save()
        .map_err(|e| format!("Could not save config.toml: {e}"))?;

    // Autostart must point at the daemon, never at this settings process.
    let mut autostart_error = None;
    if let Some(exe) = daemon_exe() {
        if let Err(e) = win_api::set_start_with_windows(&exe, cfg.start_with_windows) {
            autostart_error = Some(e);
        }
    }

    let target_missing = cfg.source.input_target.is_empty();
    let signalled = win_api::signal_refresh_event();
    let started = if signalled {
        true
    } else {
        daemon_exe()
            .map(|exe| std::process::Command::new(exe).spawn().is_ok())
            .unwrap_or(false)
    };

    let mut message = if signalled {
        "Saved. The daemon is refreshing the wallpaper.".to_string()
    } else if started {
        "Saved. The daemon was not running, so it was started.".to_string()
    } else {
        "Saved. The daemon is not running; start git-track-wallpaper.exe.".to_string()
    };
    if target_missing {
        message.push_str(" Add a GitHub username or repository URL to track something.");
    }
    if let Some(e) = autostart_error {
        message.push_str(&format!(" Autostart could not be updated: {e}"));
    }

    Ok(ApplyResult {
        message,
        daemon_running: win_api::daemon_event_exists(),
        autostart: win_api::is_start_with_windows(),
    })
}

#[tauri::command]
pub async fn pick_image_file() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "webp"])
            .pick_file()
            .map(|path| path.display().to_string())
    })
    .await
    .map_err(|e| format!("file dialog failed: {e}"))
}

/// Show the window once the backdrop is in place, so the user never sees an
/// unstyled white frame. Returns true when the Mica backdrop was applied.
#[tauri::command]
pub fn show_window(window: tauri::Window) -> Result<bool, String> {
    let mica = crate::mica::apply_mica(&window);
    crate::mica::apply_dark_caption(&window);
    window.show().map_err(|e| e.to_string())?;
    let _ = window.set_focus();
    Ok(mica)
}
