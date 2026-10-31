//! Native fullscreen; transition geometry must never replace the normal window frame.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, WebviewWindow};

/// Share the exact production window style with the native macOS smoke check.
pub fn build_main_window(app: &AppHandle, config: &tauri::utils::config::WindowConfig) -> tauri::Result<WebviewWindow> {
    let builder = tauri::WebviewWindowBuilder::from_config(app, config)?;
    #[cfg(target_os = "macos")]
    let builder = builder.decorations(true).transparent(false).theme(Some(tauri::Theme::Dark))
        .title_bar_style(tauri::TitleBarStyle::Overlay).hidden_title(true)
        .traffic_light_position(tauri::LogicalPosition::new(16.0, 22.0));
    builder.build()
}

#[derive(Default)]
pub struct FullscreenTransition(pub AtomicBool);

/// Accessory apps cannot enter AppKit fullscreen. Visible windows use Regular policy.
pub fn show_window(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_dock_visibility(true)?;
    #[cfg(not(target_os = "macos"))]
    let _ = app;
    window.unminimize()?;
    crate::constrain_current(window)?;
    window.show()?;
    window.set_focus()
}

pub fn hide_window(app: &AppHandle, window: &WebviewWindow, keep_in_dock: bool) -> tauri::Result<()> {
    window.hide()?;
    #[cfg(target_os = "macos")]
    sync_dock_visibility(app, keep_in_dock)?;
    #[cfg(not(target_os = "macos"))]
    let _ = (app, keep_in_dock);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn sync_dock_visibility(app: &AppHandle, keep_in_dock: bool) -> tauri::Result<()> {
    let visible = app.webview_windows().values().any(|window|
        window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(false));
    app.set_dock_visibility(visible || keep_in_dock)
}

pub fn observe(window: &WebviewWindow) -> bool {
    let fullscreen = window.is_fullscreen().unwrap_or(false);
    if fullscreen {
        window.state::<FullscreenTransition>().0.store(false, Ordering::Release);
    }
    fullscreen
}

pub fn suppress_geometry(window: &WebviewWindow) -> bool {
    window.is_fullscreen().unwrap_or(false)
        || (window.label() == "main"
            && window.state::<FullscreenTransition>().0.load(Ordering::Acquire))
}

#[tauri::command]
pub fn set_fullscreen(app: AppHandle, fullscreen: bool) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("窗口不存在")?;
    let transition = app.state::<FullscreenTransition>();
    if transition.0.load(Ordering::Acquire) {
        return Err("正在进入全屏，请稍后再试".into());
    }
    if window.is_fullscreen().map_err(|error| error.to_string())? == fullscreen {
        return Ok(());
    }
    if fullscreen {
        crate::remember_window(&app)?;
        transition.0.store(true, Ordering::Release);
    }
    if let Err(error) = window.set_fullscreen(fullscreen) {
        transition.0.store(false, Ordering::Release);
        return Err(error.to_string());
    }
    Ok(())
}
