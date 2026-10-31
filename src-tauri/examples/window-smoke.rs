//! Real AppKit check on the macOS CI desktop; no user stores, UI automation or input hooks.

#[cfg(target_os = "macos")]
fn main() {
    use std::time::{Duration, Instant};

    fn wait_for(label: &str, check: impl Fn() -> bool) -> Result<(), String> {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(12) {
            if check() { return Ok(()); }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(format!("Native window check timed out: {label}"))
    }

    tauri::Builder::default()
        .setup(|app| {
            let mut config = app.config().app.windows.iter().find(|window| window.label == "main")
                .expect("Missing main window config").clone();
            // Set the initial frame, just as production does with a persisted normal placement.
            config.width = 800.0;
            config.height = 400.0;
            config.center = true;
            let window = magpie_hud_lib::window_mode::build_main_window(app.handle(), &config)?;
            window.set_effects(None::<tauri::utils::config::WindowEffectsConfig>)?;
            window.set_background_color(Some(tauri::window::Color(23, 25, 30, 255)))?;
            window.show()?;
            window.set_focus()?;
            app.handle().set_dock_visibility(false)?;
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let result = (|| -> Result<(), String> {
                    std::thread::sleep(Duration::from_millis(700));
                    if !window.is_decorated().map_err(|e| e.to_string())?
                        || !window.is_closable().map_err(|e| e.to_string())?
                        || !window.is_minimizable().map_err(|e| e.to_string())?
                        || !window.is_maximizable().map_err(|e| e.to_string())? {
                        return Err("Native window controls are not enabled".into());
                    }
                    let mut failures = Vec::new();
                    for (label, dock, pinned) in [("regular", true, false), ("menu-bar", false, false),
                                                  ("pinned", true, true), ("menu-bar+pinned", false, true)] {
                        handle.set_dock_visibility(dock).map_err(|e| e.to_string())?;
                        window.set_always_on_top(pinned).map_err(|e| e.to_string())?;
                        // Policy changes can drop activation; focus after applying each policy.
                        window.set_focus().map_err(|e| e.to_string())?;
                        std::thread::sleep(Duration::from_millis(500));
                        let normal = window.outer_size().map_err(|e| e.to_string())?;
                        window.set_fullscreen(true).map_err(|e| e.to_string())?;
                        let entered = wait_for("fullscreen must occupy the monitor", || {
                            let (Ok(size), Ok(Some(monitor))) = (window.outer_size(), window.current_monitor()) else { return false };
                            window.is_fullscreen().unwrap_or(false)
                                && size.width.abs_diff(monitor.size().width) <= 4
                                && size.height.abs_diff(monitor.size().height) <= 4
                        });
                        println!("Native fullscreen {label}: result={entered:?}, requested={:?}, frame={:?}, screen={:?}, focus={:?}",
                            window.is_fullscreen(), window.outer_size(),
                            window.current_monitor().map(|m| m.map(|m| *m.size())), window.is_focused());
                        if entered.is_err() { failures.push(label); }
                        window.set_fullscreen(false).map_err(|e| e.to_string())?;
                        let restored = wait_for("normal frame must return after leaving fullscreen", || {
                            let Ok(size) = window.outer_size() else { return false };
                            !window.is_fullscreen().unwrap_or(true)
                                && size.width.abs_diff(normal.width) <= 4
                                && size.height.abs_diff(normal.height) <= 4
                        });
                        println!("Native restore {label}: result={restored:?}, expected={normal:?}, frame={:?}, requested={:?}",
                            window.outer_size(), window.is_fullscreen());
                        restored?;
                    }
                    if !failures.is_empty() { return Err(format!("Native fullscreen failed in modes: {failures:?}")); }
                    println!("Native macOS fullscreen: all four modes and original frame restoration verified");
                    Ok(())
                })();
                if let Err(error) = &result { eprintln!("{error}"); }
                handle.exit(if result.is_ok() { 0 } else { 1 });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Native macOS window smoke check failed to start");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
