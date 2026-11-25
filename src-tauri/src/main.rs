// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Some(code) = abiding_word_lib::scripture::pdf_worker::run_if_requested() {
        std::process::exit(code);
    }
    if cfg!(windows)
        && option_env!("ABIDING_WORD_ACCEPTANCE_BUILD") == Some("scripture")
        && !acceptance_profile()
    {
        std::process::exit(78);
    }
    abiding_word_lib::run()
}

fn acceptance_profile() -> bool {
    use std::{env, fs, path::Path};
    let Ok(temp) = env::temp_dir().canonicalize() else {
        return false;
    };
    let profile = |name| env::var_os(name).and_then(|path| Path::new(&path).canonicalize().ok());
    let (Some(data), Some(webview)) = (
        profile("ABIDING_WORD_DATA_DIR"),
        profile("WEBVIEW2_USER_DATA_FOLDER"),
    ) else {
        return false;
    };
    let Some(root) = data.parent() else {
        return false;
    };
    root.parent() == Some(temp.as_path())
        && root.file_name().is_some_and(|name| {
            name.to_string_lossy()
                .starts_with("aw-scripture-acceptance-")
        })
        && data == root.join("app-data")
        && webview == root.join("webview2")
        && fs::read(root.join("acceptance-profile"))
            .is_ok_and(|bytes| bytes == b"Abiding Word Scripture acceptance")
}
