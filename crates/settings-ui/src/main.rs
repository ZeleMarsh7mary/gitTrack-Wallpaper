//! `git-track-wallpaper-settings.exe` — the on-demand settings window.
//!
//! A Tauri v2 shell around `git-track-wallpaper-core`: the webview process is
//! only when the user opens Settings and is torn down completely when the window
//! closes, so the background daemon never pays for a WebView.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod mica;
mod ui_config;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::render_preview,
            commands::render_card_overlay,
            commands::background_image,
            commands::test_connection,
            commands::save_and_apply,
            commands::pick_image_file,
            commands::show_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the GitTrack Wallpaper settings window");
}
