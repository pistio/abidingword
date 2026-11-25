//! Optional read-only Windows WebView2 geometry diagnostics.
//!
//! ABIDING_WORD_VIEWPORT_TRACE=1 enables dimension-only samples. Without it,
//! scheduling returns before starting a worker. Tauri manages the window and
//! embedded webview sizes; this collector only observes their geometry.
//! Native events schedule work off the event-handler thread to avoid blocking
//! WebView2's Windows message loop. No Scripture or user data is read or logged.

use tauri::{PhysicalSize, WebviewWindow};

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{mpsc, LazyLock};
    use std::thread;
    use std::time::Duration;

    // The only observed window is "main"; coalesce bursts of native events.
    static REQUESTED: AtomicU64 = AtomicU64::new(0);
    static WORKER_ACTIVE: AtomicBool = AtomicBool::new(false);
    // Native measurements are enabled only for explicit diagnostic runs.
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
                // SAFETY: the COM calls run on WebView2's owning UI thread.
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
                "sequence": revision, "phase": "observed",
                "repairEnabled": false,
                "client": [expected.width, expected.height],
                "webview": [actual.width, actual.height],
                "controller": controller, "tauriBounds": bounds,
                "scale": main.scale_factor().ok(),
                "stableClient": current_client == Some(expected)
            })
        );
    }

    fn observe(main: &WebviewWindow, revision: u64) {
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
        sample(main, revision, expected, actual);
    }

    /// Native event callbacks must return promptly. Post a small bounded
    /// worker that measures current geometry after WebView2's own resize
    /// notifications. The 40ms coalescing interval is not a geometry/height fix
    /// and is not taken as evidence that the compositor has settled.
    pub(super) fn schedule(main: WebviewWindow) {
        if !*TRACE || main.label() != "main" {
            return;
        }
        REQUESTED.fetch_add(1, Ordering::AcqRel);
        if WORKER_ACTIVE.swap(true, Ordering::AcqRel) {
            return;
        }
        thread::spawn(move || loop {
            let revision = REQUESTED.load(Ordering::Acquire);
            thread::sleep(Duration::from_millis(40));
            observe(&main, revision);
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
