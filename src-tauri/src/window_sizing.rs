//! Keep a Windows WebView2 surface in step with its native client area.
//!
//! This addresses the owner-observed blank band below *all three* research panes.
//! The HTML shell already fills the entire browser viewport; styling individual
//! panes cannot repair a WebView2 controller whose physical bounds are stale.
//! This is limited to the main window. The presentation audience has its own
//! window and is never changed here.

use tauri::PhysicalSize;
#[cfg(target_os = "windows")]
use tauri::WebviewWindow;

const PIXEL_TOLERANCE: u32 = 2;

/// Small platform-independent policy; Rust tests exercise real examples from
/// the owner screenshot without creating a window or touching any user data.
fn needs_repair(expected: PhysicalSize<u32>, actual: PhysicalSize<u32>, force: bool) -> bool {
    expected.width > 0
        && expected.height > 0
        && (force
            || expected.width.abs_diff(actual.width) > PIXEL_TOLERANCE
            || expected.height.abs_diff(actual.height) > PIXEL_TOLERANCE)
}

/// Re-apply physical WebView bounds when its size diverges from the native
/// client area. Focus/initialisation forces one sync even if the reported
/// bounds match: the embedded WebView2 compositor may be stale after DPI or
/// monitor changes. Never resize the native window itself.
#[cfg(target_os = "windows")]
pub(crate) fn reconcile(main: &WebviewWindow, force: bool) {
    if main.label() != "main" {
        return;
    }
    let Ok(expected) = main.inner_size() else {
        return;
    };
    // WebviewWindow::set_size resizes the *native window*; use its AsRef<Webview>
    // instead to resize only the embedded browser surface.
    let webview: &tauri::Webview = main.as_ref();
    let Ok(actual) = webview.size() else {
        return;
    };
    if !needs_repair(expected, actual, force) {
        return;
    }
    match webview.set_size(expected) {
        Ok(()) => {
            if expected.width.abs_diff(actual.width) > PIXEL_TOLERANCE
                || expected.height.abs_diff(actual.height) > PIXEL_TOLERANCE
            {
                log::warn!(
                    "Main WebView2 physical-size reconciliation requested: window={}x{}, previous-webview={}x{}",
                    expected.width,
                    expected.height,
                    actual.width,
                    actual.height
                );
            }
        }
        Err(error) => {
            log::warn!(
                "Main WebView2 physical-size reconciliation failed: window={}x{}, webview={}x{}, reason={error}",
                expected.width,
                expected.height,
                actual.width,
                actual.height
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_owner_shaped_bottom_gap() {
        // 1903x~999 client vs 1903x~806 rendered surface: equal widths,
        // materially shorter browser. Values approximate the screenshot;
        // actual native-client dimensions must be measured on Windows.
        assert!(needs_repair(
            PhysicalSize::new(1903, 999),
            PhysicalSize::new(1903, 806),
            false
        ));
    }

    #[test]
    fn normal_matching_client_is_not_resized_repeatedly() {
        assert!(!needs_repair(
            PhysicalSize::new(1280, 800),
            PhysicalSize::new(1280, 800),
            false
        ));
        assert!(!needs_repair(
            PhysicalSize::new(1280, 800),
            PhysicalSize::new(1279, 799),
            false
        ));
    }

    #[test]
    fn focus_reapplies_bounds_and_minimised_windows_are_not_forced() {
        assert!(needs_repair(
            PhysicalSize::new(1280, 800),
            PhysicalSize::new(1280, 800),
            true
        ));
        assert!(!needs_repair(
            PhysicalSize::new(0, 0),
            PhysicalSize::new(0, 0),
            true
        ));
    }
}
