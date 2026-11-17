//! `git-wallpaper.exe`
//!
//! * no arguments        -> background daemon (tray icon, scheduler)
//! * `--settings`        -> on-demand settings window, exits when closed
//! * `--apply-once`      -> one fetch + render + set-wallpaper cycle, then exit
//! * `--render-once [p]` -> render one wallpaper PNG to `p` (debug/verification)
//! * `--version`         -> print version

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod daemon;
mod git_fetcher;
mod renderer;
mod security;
mod ui_settings;
mod win_api;

use config::Config;
use renderer::Scene;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let first = args.first().map(String::as_str).unwrap_or("");

    let result = match first {
        "--settings" => ui_settings::run(),
        "--apply-once" => daemon::apply_once(),
        "--render-once" => render_once(args.get(1).map(String::as_str)),
        "--version" | "-V" => {
            println!("git-wallpaper {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => daemon::run(),
    };

    if let Err(err) = result {
        daemon::log_line(&format!("fatal: {err}"));
        std::process::exit(1);
    }
}

/// Debug/verification entry point: renders the wallpaper exactly like the
/// daemon does, but writes it wherever the caller asks and never touches the
/// desktop unless no path was given.
fn render_once(out: Option<&str>) -> Result<(), String> {
    let (cfg, warn) = Config::load_or_default();
    if let Some(w) = warn {
        daemon::log_line(&w);
    }
    // Prefer fresh data, then the cache, then the deterministic demo year so the
    // compositor can be verified before GitHub is configured.
    let days = match git_fetcher::fetch(&cfg) {
        Ok(days) => days,
        Err(err) => {
            daemon::log_line(&format!("render-once fetch: {err}"));
            git_fetcher::load_cache().unwrap_or_else(git_fetcher::demo_days)
        }
    };

    let (w, h) = win_api::primary_monitor_size();
    let scene = Scene {
        config: &cfg,
        days: &days,
        size_px: (w, h),
        scale: 1.0,
    };
    let started = std::time::Instant::now();
    let png = renderer::render_png(&scene)?;
    let elapsed = started.elapsed();

    match out {
        Some(path) if !path.is_empty() => {
            config::write_atomic(std::path::Path::new(path), &png)?;
            println!(
                "render {w}x{h} -> {path} ({} byte, {elapsed:.1?})",
                png.len()
            );
        }
        _ => {
            let path = config::wallpaper_path();
            config::write_atomic(&path, &png)?;
            win_api::set_wallpaper(&path)?;
            println!("wallpaper set from {}", path.display());
        }
    }
    Ok(())
}
