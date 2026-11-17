//! Background daemon: tray icon, Win32 message loop, sleep-based scheduler,
//! named-event IPC, and working-set trimming after every render.
//!
//! No window, no WebView, no async runtime: one worker thread does the network
//! and CPU compositing, the main thread only pumps messages.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, SetTimer, TranslateMessage, MSG, WM_TIMER,
};

use crate::config::{self, Config};
use crate::renderer::{render_png, Scene};
use crate::win_api;

/// Windows is still busy logging in; do not race it for network or disk.
const BOOT_DELAY_MS: u32 = 12_000;
/// Tray-menu polling cadence; the loop is otherwise blocked in `GetMessageW`.
const MENU_TICK_MS: u32 = 250;

pub fn log_line(msg: &str) {
    use std::io::Write;
    let dir = config::local_data_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("daemon.log"))
    {
        let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(f, "[{stamp}] {msg}");
    }
}

pub fn run() -> Result<(), String> {
    let Some(_guard) = win_api::acquire_single_instance() else {
        // Another daemon already owns the mutex; nothing to do.
        return Ok(());
    };

    let running = Arc::new(AtomicBool::new(true));
    let worker = {
        let running = Arc::clone(&running);
        std::thread::Builder::new()
            .name("git-wallpaper-worker".into())
            .spawn(move || worker_loop(running))
            .map_err(|e| format!("spawn worker: {e}"))?
    };

    let refresh_id = MenuId::new("refresh");
    let settings_id = MenuId::new("settings");
    let exit_id = MenuId::new("exit");

    let refresh_item = MenuItem::with_id(refresh_id.clone(), "Refresh Now", true, None);
    let settings_item = MenuItem::with_id(settings_id.clone(), "Settings...", true, None);
    let exit_item = MenuItem::with_id(exit_id.clone(), "Exit", true, None);
    let separator = PredefinedMenuItem::separator();
    let menu = Menu::new();
    menu.append_items(&[&refresh_item, &settings_item, &separator, &exit_item])
        .map_err(|e| format!("build tray menu: {e}"))?;

    let Some(icon) = tray_icon_image() else {
        return Err("could not create the tray icon".into());
    };
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("GitWallpaper - kontribusi GitHub di desktop")
        .with_icon(icon)
        .build()
        .map_err(|e| format!("create tray icon: {e}"))?;

    unsafe {
        // Thread-scoped timer so the blocking GetMessageW wakes up regularly and
        // we can drain the tray menu channel without a spinning thread.
        SetTimer(None, 1, MENU_TICK_MS, None);
    }

    let mut msg = MSG::default();
    loop {
        let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if ret.0 == 0 || ret.0 == -1 {
            break;
        }
        if msg.message == WM_TIMER {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == exit_id {
                    running.store(false, Ordering::Relaxed);
                    win_api::signal_refresh_event();
                    let _ = worker.join();
                    return Ok(());
                } else if event.id == refresh_id {
                    win_api::signal_refresh_event();
                } else if event.id == settings_id {
                    open_settings();
                }
            }
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    running.store(false, Ordering::Relaxed);
    win_api::signal_refresh_event();
    let _ = worker.join();
    Ok(())
}

fn open_settings() {
    let Ok(exe) = std::env::current_exe() else {
        log_line("current_exe failed; cannot open Settings");
        return;
    };
    if let Err(e) = std::process::Command::new(exe).arg("--settings").spawn() {
        log_line(&format!("spawn --settings: {e}"));
    }
}

fn worker_loop(running: Arc<AtomicBool>) {
    let Some(event) = win_api::create_refresh_event() else {
        log_line("CreateEventW failed; worker stopped");
        return;
    };
    let (cfg0, _) = Config::load_or_default();
    // Only ever *add* the Run value here. Removing it is the Settings window's
    // job: the installer legitimately pre-creates it while the config still has
    // `start_with_windows = false`.
    if cfg0.start_with_windows {
        if let Ok(exe) = std::env::current_exe() {
            if let Err(e) = win_api::set_start_with_windows(&exe, true) {
                log_line(&format!("autostart: {e}"));
            }
        }
    }

    let mut first = true;
    while running.load(Ordering::Relaxed) {
        let (cfg, _) = Config::load_or_default();
        let timeout = if first {
            BOOT_DELAY_MS
        } else {
            cfg.source
                .refresh_interval_minutes
                .saturating_mul(60_000)
                .max(60_000)
        };
        first = false;
        event.wait(timeout);
        if !running.load(Ordering::Relaxed) {
            break;
        }
        refresh_once(&cfg);
        win_api::trim_working_set();
    }
    log_line("worker stopped");
}

/// One full cycle: fetch (or cache), composite, write, set wallpaper.
pub fn refresh_once(cfg: &Config) {
    let (days, err) = crate::git_fetcher::resolve(cfg);
    if let Some(e) = err {
        log_line(&format!("fetch failed, using cache: {e}"));
    }
    let (w, h) = win_api::primary_monitor_size();
    let scene = Scene {
        config: cfg,
        days: &days,
        size_px: (w, h),
        scale: 1.0,
    };
    match render_png(&scene) {
        Ok(png) => {
            let path = config::wallpaper_path();
            match config::write_atomic(&path, &png) {
                Ok(()) => match win_api::set_wallpaper(&path) {
                    Ok(()) => log_line(&format!(
                        "wallpaper updated {}x{} ({} days, {} byte png)",
                        w,
                        h,
                        days.len(),
                        png.len()
                    )),
                    Err(e) => log_line(&format!("set wallpaper: {e}")),
                },
                Err(e) => log_line(&format!("write wallpaper: {e}")),
            }
        }
        Err(e) => log_line(&format!("render: {e}")),
    }
}

/// `--apply-once`: same cycle as the daemon, then exit. Used by the installer
/// verification and by hand when debugging.
pub fn apply_once() -> Result<(), String> {
    let (cfg, warn) = Config::load_or_default();
    if let Some(w) = warn {
        log_line(&w);
    }
    refresh_once(&cfg);
    Ok(())
}

fn tray_icon_image() -> Option<Icon> {
    const SIZE: usize = 32;
    let bg = [14u8, 18, 24, 255];
    let levels: [[u8; 4]; 4] = [
        [14, 68, 41, 255],
        [0, 109, 50, 255],
        [38, 166, 65, 255],
        [57, 211, 83, 255],
    ];
    let mut buf = vec![0u8; SIZE * SIZE * 4];
    for px in buf.chunks_exact_mut(4) {
        px.copy_from_slice(&bg);
    }
    for gy in 0..4usize {
        for gx in 0..4usize {
            let color = levels[(gx * 3 + gy * 5) % 4];
            let x0 = 3 + gx * 7;
            let y0 = 3 + gy * 7;
            for y in y0..(y0 + 5) {
                for x in x0..(x0 + 5) {
                    if x < SIZE && y < SIZE {
                        let i = (y * SIZE + x) * 4;
                        buf[i..i + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }
    Icon::from_rgba(buf, SIZE as u32, SIZE as u32).ok()
}
