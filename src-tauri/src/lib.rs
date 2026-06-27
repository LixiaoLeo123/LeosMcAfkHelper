pub mod commands;
pub mod domain;
pub mod mcc;
pub mod store;

use commands::LogBuffer;
use mcc::process::{kill_all_players, Runtime};
use store::Store;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Log the app data directory so users can find it.
            match app.path().app_data_dir() {
                Ok(dir) => eprintln!("[setup] App data directory: {:?}", dir),
                Err(e) => eprintln!("[setup] WARNING: could not determine app data directory: {e}"),
            }

            // Load the persistent player store.
            let store = Store::load(app.handle())?;
            eprintln!("[setup] Loaded {} players from store.", store.list().len());
            app.manage(store);

            // Runtime registry for live MCC subprocesses.
            app.manage(Runtime::new());

            // In-memory log ring buffer.
            app.manage(LogBuffer::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_players,
            commands::add_player,
            commands::update_player,
            commands::delete_player,
            commands::start_player_cmd,
            commands::stop_player_cmd,
            commands::send_command_cmd,
            commands::get_player_status,
            commands::get_recent_logs,
            commands::preview_script,
            commands::open_player_workdir,
            commands::resolve_mcc_binary_cmd,
            commands::ensure_player_dirs_cmd,
            commands::write_player_files,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let handle = app_handle.clone();
                tauri::async_runtime::block_on(async move {
                    kill_all_players(&handle).await;
                });
            }
        });
}
