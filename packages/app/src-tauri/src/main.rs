// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    kittynode_tauri_lib::run()?;
    Ok(())
}
