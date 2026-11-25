// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Some(code) = abiding_word_lib::scripture::pdf_worker::run_if_requested() {
        std::process::exit(code);
    }
    abiding_word_lib::run()
}
