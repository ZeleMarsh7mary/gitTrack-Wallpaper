//! Shared, UI-free logic for GitWallpaper.
//!
//! Everything that both the background daemon (`git-wallpaper.exe`) and the
//! on-demand settings window (`git-wallpaper-settings.exe`) need lives here:
//! configuration, DPAPI secret storage, the GitHub fetcher, the CPU compositor
//! and the thin Win32 wrappers.
//!
//! The renderer touches no Win32 API and no files unless the configured
//! background is an image, so it can be exercised from tests and preview renders.

pub mod config;
pub mod git_fetcher;
pub mod renderer;
pub mod security;
pub mod win_api;
