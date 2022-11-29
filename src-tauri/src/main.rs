#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{Manager, RunEvent};
use tokio::sync::Mutex as AsyncMutex;

struct StateWrapper(AsyncMutex<State>);

struct State {
    unique_open_close_prefix: u32
}

fn main() {

    let unique_open_close_prefix = StateWrapper(AsyncMutex::new(State { unique_open_close_prefix: 0 }));

    tauri::Builder::default()
        .manage(unique_open_close_prefix)
        .invoke_handler(tauri::generate_handler![close_and_reopen_cmd])
        .setup(|app| {
            open_several_windows(&app.handle(), 0)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        // don't quit on the last window so we can launch mor
        .run(|_, e| {
            if let RunEvent::ExitRequested { api, .. } = e {
                api.prevent_exit();
            }
        });
}

#[tauri::command(async)]
async fn close_and_reopen_cmd(app: tauri::AppHandle, state_wrapper: tauri::State<'_, StateWrapper>) -> Result<(), String> {

    // there's no particular reason we need to have an async mutex here, that's just what my app is
    // using IRL for roughly equivalent functionality
    let mut state = state_wrapper.0.lock().await;
    state.unique_open_close_prefix += 1;
    let label_prefix = state.unique_open_close_prefix;

    close_and_reopen(app, label_prefix)?;
    Ok(())
}

fn close_and_reopen(app: tauri::AppHandle, label_prefix: u32) -> Result<(), String> {
    close_all_windows(&app)?;
    open_several_windows(&app, label_prefix)?;
    Ok(())
}

fn open_several_windows(app: &tauri::AppHandle, label_prefix: u32) -> Result<(), String> {
    open_window(app, label_prefix, "a", 0.0, 0.0)?;
    open_window(app, label_prefix, "b", 100.0, 100.0)?;
    open_window(app, label_prefix, "c", 200.0, 200.0)?;
    open_window(app, label_prefix, "d", 300.0, 300.0)?;
    open_window(app, label_prefix, "e", 400.0, 400.0)?;
    open_window(app, label_prefix, "f", 500.0, 500.0)
}

fn open_window(app: &tauri::AppHandle, label_prefix: u32, label: &str, x: f64, y: f64) -> Result<(), String> {

    // make the name unique
    let tauri_id = format!("{}-{}", label_prefix, label);

    let url = tauri::WindowUrl::App("index.html".into());

    tauri::WindowBuilder::new(app, tauri_id, url)
        .inner_size(600.0, 600.0)
        .position(x, y)
        .disable_file_drop_handler()
        .title(format!("{} (attempt {})", label, label_prefix))
        .build()
        .map_err(|_| "Couldn't launch window".to_string())?;

    Ok(())
}


fn close_all_windows(app: &tauri::AppHandle) -> Result<(), String> {
    let tauri_windows = app.windows();
    for tauri_window in tauri_windows.values() {
        tauri_window.close().unwrap();
    }

    Ok(())
}
