//! Native window chrome: DWM system backdrop (Mica) and a dark caption.
//!
//! Nothing here is a plugin dependency: three `DwmSetWindowAttribute` calls are
//! all Windows 11 needs, and a pre-22H2 build simply keeps the caption colour.

use std::ffi::c_void;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMSBT_MAINWINDOW, DWMWA_CAPTION_COLOR, DWMWA_SYSTEMBACKDROP_TYPE,
    DWMWA_USE_IMMERSIVE_DARK_MODE,
};

/// Raw window handle as an `HWND` from *our* `windows` crate version.
fn hwnd_of(window: &tauri::Window) -> Option<HWND> {
    let handle = window.hwnd().ok()?;
    // Round-trip through `isize` so this keeps working even if the `windows`
    // crate version used by tauri differs from the one pinned here.
    let raw: isize = handle.0 as isize;
    Some(HWND(raw as *mut c_void))
}

/// Apply a Mica backdrop. Returns false on Windows 10 or when DWM refuses.
pub fn apply_mica(window: &tauri::Window) -> bool {
    let Some(hwnd) = hwnd_of(window) else {
        return false;
    };
    let backdrop = DWMSBT_MAINWINDOW;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const _ as *const c_void,
            std::mem::size_of_val(&backdrop) as u32,
        )
        .is_ok()
    }
}

/// Paint the native caption dark and match it to the app background.
pub fn apply_dark_caption(window: &tauri::Window) {
    let Some(hwnd) = hwnd_of(window) else {
        return;
    };
    let dark: i32 = 1;
    // COLORREF is 0x00BBGGRR; #202020 is the same in both orders.
    let caption: u32 = 0x0020_2020;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const i32 as *const c_void,
            std::mem::size_of::<i32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &caption as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}
