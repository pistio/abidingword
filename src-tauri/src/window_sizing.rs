//! Keep the main Windows WebView2 surface aligned with the native client area.
//!
//! Do not guess a CSS height from a screenshot. Measure the window and webview
//! separately; change only the embedded webview when their physical sizes differ.
//! Tauri's WebviewWindow auto-fills its parent already: set_auto_resize() is a
//! documented no-op here and deliberately not called.
//! Native events schedule work off the event-handler thread to avoid blocking
//! WebView2's Windows message loop. No Scripture or user data is read or logged.

use tauri::PhysicalSize;
#[cfg(target_os = "windows")]
use tauri::WebviewWindow;

const PIXEL_TOLERANCE: u32 = 2;

fn needs_repair(expected: PhysicalSize<u32>, actual: PhysicalSize<u32>) -> bool {
    expected.width > 0
        && expected.height > 0
        && (expected.width.abs_diff(actual.width) > PIXEL_TOLERANCE
            || expected.height.abs_diff(actual.height) > PIXEL_TOLERANCE)
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{mpsc, LazyLock};
    use std::thread;
    use std::time::Duration;

    // The only reconciled window is "main"; coalesce bursts of resize events.
    static REQUESTED: AtomicU64 = AtomicU64::new(0);
    static WORKER_ACTIVE: AtomicBool = AtomicBool::new(false);
    static INITIAL_MEASUREMENT_LOGGED: AtomicBool = AtomicBool::new(false);
    // Diagnostic A/B runs use the same executable and disposable profiles.
    // Unknown explicit modes fail closed to observation, never a forced resize.
    static REPAIR_ENABLED: LazyLock<bool> = LazyLock::new(|| {
        std::env::var_os("ABIDING_WORD_VIEWPORT_MODE")
            .map(|mode| mode == "repair")
            .unwrap_or(true)
    });
    static TRACE: LazyLock<bool> =
        LazyLock::new(|| std::env::var("ABIDING_WORD_VIEWPORT_TRACE").as_deref() == Ok("1"));

    // Wry 0.57 returns a zero position for a WindowContent webview, even if
    // controller.Bounds has an offset. Read the actual COM RECT on the UI
    // thread via Tauri's dispatcher; never move a COM interface to this worker.
    fn controller_bounds(webview: &tauri::Webview) -> Option<serde_json::Value> {
        let (tx, rx) = mpsc::channel();
        webview
            .with_webview(move |platform| {
                let controller = platform.controller();
                let mut rect = Default::default();
                let mut parent = Default::default();
                let mut zoom = 0.0;
                // SAFETY: both COM calls run on WebView2's owning UI thread.
                let measured = unsafe { controller.Bounds(&mut rect) }.ok().map(|()| {
                    let parent = unsafe { controller.ParentWindow(&mut parent) }
                        .ok()
                        .map(|()| format!("{:x}", parent.0 as usize));
                    serde_json::json!({
                        "rect": [rect.left, rect.top, rect.right, rect.bottom],
                        "parentHwnd": parent,
                        "zoomFactor": unsafe { controller.ZoomFactor(&mut zoom) }.ok().map(|()| zoom)
                    })
                });
                let _ = tx.send(measured);
            })
            .ok()?;
        rx.recv_timeout(Duration::from_secs(2)).ok().flatten()
    }

    fn sample(
        main: &WebviewWindow,
        revision: u64,
        phase: &str,
        expected: PhysicalSize<u32>,
        actual: PhysicalSize<u32>,
    ) {
        let webview: &tauri::Webview = main.as_ref();
        let controller = controller_bounds(webview);
        let bounds = webview.bounds().ok().map(|rect| format!("{rect:?}"));
        let current_client = main.inner_size().ok();
        log::info!(
            "Viewport sample {}",
            serde_json::json!({
                "sequence": revision, "phase": phase,
                "repairEnabled": *REPAIR_ENABLED,
                "client": [expected.width, expected.height],
                "webview": [actual.width, actual.height],
                "controller": controller, "tauriBounds": bounds,
                "scale": main.scale_factor().ok(),
                "stableClient": current_client == Some(expected)
            })
        );
    }

    fn reconcile(main: &WebviewWindow, revision: u64) {
        if main.label() != "main" || main.is_minimized().unwrap_or(true) {
            return;
        }
        let Ok(expected) = main.inner_size() else {
            return;
        };
        if expected.width == 0 || expected.height == 0 {
            return;
        }
        let webview: &tauri::Webview = main.as_ref();
        let Ok(actual) = webview.size() else {
            log::warn!("Could not measure the main WebView2 surface");
            return;
        };
        let mismatch = needs_repair(expected, actual);
        let initial = !INITIAL_MEASUREMENT_LOGGED.swap(true, Ordering::AcqRel);
        if initial || mismatch || *TRACE {
            sample(main, revision, "observed", expected, actual);
        }
        if !mismatch || !*REPAIR_ENABLED {
            return;
        }
        // Independent dispatcher reads are not an atomic geometry snapshot.
        // Drop stale observations; the coalescer will process the latest event.
        if REQUESTED.load(Ordering::Acquire) != revision
            || main.inner_size().ok() != Some(expected)
            || main.is_minimized().unwrap_or(true)
        {
            return;
        }
        match webview.set_size(expected) {
            Ok(()) => {
                // The setter queues work. This getter is a subsequent dispatcher
                // readback, still not proof of DOM layout/compositor repaint.
                if let Ok(observed) = webview.size() {
                    sample(main, revision, "after-request", expected, observed);
                }
            }
            Err(error) => log::warn!("WebView2 size request failed: {error}"),
        }
    }

    /// Native event callbacks must return promptly. Post a small bounded
    /// worker that measures current geometry after WebView2's own resize
    /// notifications. The 40ms coalescing interval is not a geometry/height fix
    /// and is not taken as evidence that the compositor has settled.
    pub(super) fn schedule(main: WebviewWindow) {
        if main.label() != "main" {
            return;
        }
        REQUESTED.fetch_add(1, Ordering::AcqRel);
        if WORKER_ACTIVE.swap(true, Ordering::AcqRel) {
            return;
        }
        thread::spawn(move || loop {
            let revision = REQUESTED.load(Ordering::Acquire);
            thread::sleep(Duration::from_millis(40));
            reconcile(&main, revision);
            if REQUESTED.load(Ordering::Acquire) != revision {
                continue;
            }
            WORKER_ACTIVE.store(false, Ordering::Release);
            if REQUESTED.load(Ordering::Acquire) == revision
                || WORKER_ACTIVE.swap(true, Ordering::AcqRel)
            {
                break;
            }
        });
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn initialise(main: &WebviewWindow) {
    windows::schedule(main.clone());
}

#[cfg(target_os = "windows")]
pub(crate) fn schedule(main: WebviewWindow) {
    windows::schedule(main);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_owner_shaped_bottom_gap() {
        assert!(needs_repair(
            PhysicalSize::new(1903, 999),
            PhysicalSize::new(1903, 806)
        ));
    }

    #[test]
    fn ignores_matching_geometry_and_small_dpi_rounding() {
        assert!(!needs_repair(
            PhysicalSize::new(1280, 800),
            PhysicalSize::new(1280, 800)
        ));
        assert!(!needs_repair(
            PhysicalSize::new(1280, 800),
            PhysicalSize::new(1279, 799)
        ));
    }

    #[test]
    fn ignores_zero_sized_minimised_surfaces() {
        assert!(!needs_repair(
            PhysicalSize::new(0, 0),
            PhysicalSize::new(1280, 800)
        ));
        assert!(!needs_repair(
            PhysicalSize::new(1280, 0),
            PhysicalSize::new(1280, 800)
        ));
    }

    #[test]
    fn catches_width_mismatch_independently_of_height() {
        assert!(needs_repair(
            PhysicalSize::new(1600, 900),
            PhysicalSize::new(1590, 900)
        ));
    }
}
