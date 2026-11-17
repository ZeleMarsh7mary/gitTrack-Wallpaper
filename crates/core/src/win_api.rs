//! Thin wrappers over the Win32 calls the app needs. No business logic here.

use std::ffi::c_void;
use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_ALREADY_EXISTS, HANDLE, HLOCAL, WAIT_OBJECT_0,
};
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE,
};
use windows::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, GetCurrentProcess, OpenEventW, SetEvent, SetProcessWorkingSetSize,
    WaitForSingleObject, EVENT_MODIFY_STATE, INFINITE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetWindowThreadProcessId, SystemParametersInfoW, SPIF_SENDCHANGE,
    SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

pub const MUTEX_NAME: &str = "Local\\GitTrackWallpaperDaemonMutex";
pub const REFRESH_EVENT: &str = "Local\\GitTrackWallpaperRefreshEvent";
pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const RUN_VALUE: &str = "GitTrackWallpaper";

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Physical pixels of the primary display, from the display driver itself.
pub fn primary_monitor_size() -> (u32, u32) {
    let mut devmode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };
    let ok = unsafe {
        EnumDisplaySettingsW(PCWSTR::null(), ENUM_CURRENT_SETTINGS, &mut devmode).as_bool()
    };
    if ok && devmode.dmPelsWidth > 0 && devmode.dmPelsHeight > 0 {
        (devmode.dmPelsWidth, devmode.dmPelsHeight)
    } else {
        (1920, 1080)
    }
}

pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    let mut wide = to_wide(&path.to_string_lossy());
    let result = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_mut_ptr() as *mut c_void),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };
    result.map_err(|e| format!("SPI_SETDESKWALLPAPER: {e}"))
}

/// Ask Windows to page out our committed memory. Not a leak fix; a footprint fix.
pub fn trim_working_set() {
    unsafe {
        // -1 in both slots means "trim as much as Windows allows".
        let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

/// Holds the single-instance mutex for as long as it lives.
pub struct InstanceGuard(HANDLE);

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// `None` means another daemon already owns the mutex.
pub fn acquire_single_instance() -> Option<InstanceGuard> {
    let name = to_wide(MUTEX_NAME);
    let handle = unsafe { CreateMutexW(None, true, PCWSTR(name.as_ptr())) }.ok()?;
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            let _ = CloseHandle(handle);
        }
        return None;
    }
    Some(InstanceGuard(handle))
}

pub struct NamedEvent(HANDLE);

impl Drop for NamedEvent {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Auto-reset event the daemon waits on; the settings window signals it.
pub fn create_refresh_event() -> Option<NamedEvent> {
    let name = to_wide(REFRESH_EVENT);
    unsafe { CreateEventW(None, false, false, PCWSTR(name.as_ptr())) }
        .ok()
        .map(NamedEvent)
}

/// Read-only probe: does the daemon's refresh event exist right now?
pub fn daemon_event_exists() -> bool {
    let name = to_wide(REFRESH_EVENT);
    match unsafe { OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(name.as_ptr())) } {
        Ok(h) => {
            unsafe {
                let _ = CloseHandle(h);
            }
            true
        }
        Err(_) => false,
    }
}

/// Used by `--settings`: opens the daemon's event and pokes it.
pub fn signal_refresh_event() -> bool {
    let name = to_wide(REFRESH_EVENT);
    let handle = unsafe { OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(name.as_ptr())) };
    match handle {
        Ok(h) => {
            let ok = unsafe { SetEvent(h).is_ok() };
            unsafe {
                let _ = CloseHandle(h);
            }
            ok
        }
        Err(_) => false,
    }
}

impl NamedEvent {
    /// Sleep until signalled or the timeout expires. Returns true when signalled.
    pub fn wait(&self, timeout_ms: u32) -> bool {
        let ms = if timeout_ms == u32::MAX {
            INFINITE
        } else {
            timeout_ms
        };
        unsafe { WaitForSingleObject(self.0, ms) == WAIT_OBJECT_0 }
    }
}

// ---- auto-start (HKCU, never HKLM: installer runs without admin) ----

pub fn set_start_with_windows(exe: &Path, enabled: bool) -> Result<(), String> {
    let key = if enabled {
        windows_registry::CURRENT_USER
            .create(RUN_KEY)
            .map_err(|e| format!("open {RUN_KEY}: {e}"))?
    } else {
        match windows_registry::CURRENT_USER.open(RUN_KEY) {
            Ok(k) => k,
            Err(_) => return Ok(()),
        }
    };
    if enabled {
        let value = format!("\"{}\"", exe.display());
        key.set_string(RUN_VALUE, value)
            .map_err(|e| format!("write Run value: {e}"))
    } else {
        match key.remove_value(RUN_VALUE) {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

pub fn is_start_with_windows() -> bool {
    windows_registry::CURRENT_USER
        .open(RUN_KEY)
        .and_then(|k| k.get_string(RUN_VALUE))
        .is_ok()
}

/// Free a DPAPI output buffer allocated with `LocalAlloc`.
pub fn local_free(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            LocalFree(Some(HLOCAL(ptr as *mut c_void)));
        }
    }
}

/// Paint the native caption dark and blend it into the app background.
///
/// Best effort: a missing window or an OS without DWMWA_CAPTION_COLOR (pre
/// Windows 11) leaves the default title bar, which is still usable.
pub fn apply_dark_titlebar(title: &str, background: u32) -> Result<(), String> {
    let wide_title = to_wide(title);
    let hwnd = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(wide_title.as_ptr())) }
        .map_err(|e| format!("FindWindowW: {e}"))?;

    // Never touch a window that is not ours.
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid != std::process::id() {
        return Err("top-level window belongs to another process".into());
    }

    let dark: i32 = 1;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const i32 as *const c_void,
            std::mem::size_of::<i32>() as u32,
        )
    }
    .map_err(|e| format!("DWMWA_USE_IMMERSIVE_DARK_MODE: {e}"))?;

    // COLORREF is 0x00BBGGRR; the palette value is 0xRRGGBB, so swap R and B.
    let color =
        ((background & 0x0000ff) << 16) | (background & 0x00ff00) | ((background & 0xff0000) >> 16);
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &color as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        )
    }
    .map_err(|e| format!("DWMWA_CAPTION_COLOR: {e}"))?;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &color as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
    Ok(())
}
