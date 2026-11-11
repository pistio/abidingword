//! Keep the main Windows WebView2 surface aligned with the native client area.
//!
//! Do not guess a CSS height from a screenshot. Measure the window and webview
//! separately; change only the embedded webview when their physical sizes differ.
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
    use std::thread;
    use std::time::Duration;

    // The only reconciled window is "main"; coalesce bursts of resize events.
    static REQUESTED: AtomicU64 = AtomicU64::new(0);
    static WORKER_ACTIVE: AtomicBool = AtomicBool::new(false);

    /// Restore Tauri's automatic parent-to-webview resizing. This is a native
    /// call and needs no extra frontend command permission.
    pub(super) fn enable_auto_resize(main: &WebviewWindow) {
        if main.label() != "main" {
            return;
        }
        let webview: &tauri::Webview = main.as_ref();
        if let Err(error) = webview.set_auto_resize(true) {
            log::warn!("Main WebView2 auto-resize could not be enabled: {error}");
        }
    }

    fn reconcile(main: &WebviewWindow) {
        if main.label() != "main" {
            return;
        }
        if main.is_minimized().unwrap_or(false) {
            return;
        }
        let Ok(expected) = main.inner_size() else {
            return;
        };
        let webview: &tauri::Webview = main.as_ref();
        let Ok(actual) = webview.size() else {
            log::warn!("Could not measure the main WebView2 surface");
            return;
        };
        let factor = main.scale_factor().unwrap_or(1.0);
        // These are physical sizes, not CSS px; DPI can change while a
        // window is moving between monitors. Do not resize the parent window.
        log::debug!(
            "Main window geometry: client={}x{}, webview={}x{}, scale={factor:.3}",
            expected.width, expected.height, actual.width, actual.height
        );
        if !needs_repair(expected, actual) {
            return;
        }
        match webview.set_size(expected) {
            Ok(()) => {
                log::warn!(
                    "Requested WebView2 surface repair: client={}x{}, previous-webview={}x{}, scale={factor:.3}",
                    expected.width, expected.height, actual.width, actual.height
                );
            }
            Err(error) => {
                log::warn!(
                    "WebView2 surface repair failed: client={}x{}, previous-webview={}x{}, reason={error}",
                    expected.width, expected.height, actual.width, actual.height
                );
            }
        }
    }

    /// Native event callbacks must return promptly. Post a small bounded
    /// worker that measures the *latest* geometry after WebView2's own resize
    /// notifications settle; only the final requested size is applied.
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
            reconcile(&main);
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
    windows::enable_auto_resize(main);
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
