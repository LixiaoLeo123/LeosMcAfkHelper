use crate::domain::{LogLine, PlayerConfig, PlayerStatus};
use crate::mcc::paths::{ensure_player_dirs, resolve_mcc_binary};
use crate::mcc::process::{
    is_running, send_command, start_player, stop_player, workdir_for,
};
use crate::mcc::scriptgen::generate_script;
use crate::store::Store;
use std::fs;
use tauri::{AppHandle, State};
use tokio::sync::Mutex as AsyncMutex;

/// A small in-memory log ring buffer per player so the UI can fetch recent history.
#[derive(Clone)]
pub struct LogBuffer {
    pub buffers: std::sync::Arc<dashmap::DashMap<String, AsyncMutex<Vec<LogLine>>>>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            buffers: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }
    pub async fn push(&self, id: &str, line: LogLine) {
        let entry = self
            .buffers
            .entry(id.to_string())
            .or_insert_with(|| AsyncMutex::new(Vec::new()));
        let mut guard = entry.lock().await;
        guard.push(line);
        if guard.len() > 2000 {
            let drop_n = guard.len() - 2000;
            guard.drain(0..drop_n);
        }
    }
    pub async fn recent(&self, id: &str) -> Vec<LogLine> {
        match self.buffers.get(id) {
            Some(e) => e.lock().await.clone(),
            None => Vec::new(),
        }
    }
}

#[tauri::command]
pub fn list_players(store: State<'_, Store>) -> Vec<PlayerConfig> {
    store.list()
}

#[tauri::command]
pub fn add_player(player: PlayerConfig, store: State<'_, Store>) -> Result<(), String> {
    store.add(player).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_player(player: PlayerConfig, store: State<'_, Store>) -> Result<(), String> {
    store.upsert(player).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_player(
    id: String,
    store: State<'_, Store>,
    app: AppHandle,
) -> Result<(), String> {
    // Stop if running, then remove from store. Player workdir files are kept on disk
    // (session cache, etc.) but removed from the saved list.
    let app2 = app.clone();
    let id2 = id.clone();
    tauri::async_runtime::block_on(async move {
        let _ = stop_player(&app2, &id2).await;
    });
    store.remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_player_cmd(
    id: String,
    store: State<'_, Store>,
    app: AppHandle,
) -> Result<(), String> {
    let player = store
        .get(&id)
        .ok_or_else(|| {
            let msg = format!("player not found: {id}");
            eprintln!("[start_player_cmd] FAILED: {msg}");
            msg
        })?;
    eprintln!("[start_player_cmd] Starting player '{}' ({}), server: {}, account: {:?}",
        player.name, player.id, player.server.to_address(), player.account);
    start_player(&app, &player)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            eprintln!("[start_player_cmd] FAILED for '{}': {msg}", player.name);
            // Also print the full error chain for debugging.
            eprintln!("[start_player_cmd] Full error chain: {e:#}");
            msg
        })
}

#[tauri::command]
pub async fn stop_player_cmd(id: String, app: AppHandle) -> Result<(), String> {
    stop_player(&app, &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_command_cmd(
    id: String,
    command: String,
    app: AppHandle,
) -> Result<(), String> {
    send_command(&app, &id, &command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_player_status(id: String, app: AppHandle) -> PlayerStatus {
    if is_running(&app, &id) {
        PlayerStatus::Connected // best-effort; refined by events
    } else {
        PlayerStatus::Stopped
    }
}

#[tauri::command]
pub async fn get_recent_logs(
    id: String,
    log_buffer: State<'_, LogBuffer>,
) -> Result<Vec<LogLine>, String> {
    Ok(log_buffer.recent(&id).await)
}

#[tauri::command]
pub fn preview_script(player: PlayerConfig) -> Result<String, String> {
    generate_script(&player).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_player_workdir(id: String, app: AppHandle) -> Result<(), String> {
    let dir = workdir_for(&app, &id).map_err(|e| e.to_string())?;
    open_path(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_mcc_binary_cmd(app: AppHandle) -> Result<String, String> {
    resolve_mcc_binary(&app)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ensure_player_dirs_cmd(id: String, app: AppHandle) -> Result<String, String> {
    ensure_player_dirs(&app, &id)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_player_files(
    id: String,
    player: PlayerConfig,
    app: AppHandle,
) -> Result<String, String> {
    // For debugging/preview: write the generated ini + script and return the workdir.
    use crate::mcc::config::render_ini;
    let workdir = ensure_player_dirs(&app, &id).map_err(|e| e.to_string())?;
    let ini = render_ini(&player).map_err(|e| e.to_string())?;
    let script = generate_script(&player).map_err(|e| e.to_string())?;
    fs::write(workdir.join("MinecraftClient.ini"), ini).map_err(|e| e.to_string())?;
    fs::write(workdir.join("scripts").join("bot.cs"), script).map_err(|e| e.to_string())?;
    Ok(workdir.to_string_lossy().to_string())
}

fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path).spawn()?;
    }
    Ok(())
}

pub fn build_handler() -> Box<dyn Fn(&AppHandle) + Send + Sync + 'static> {
    // We register commands via the invoke_handler in lib.rs; this is unused but kept
    // to expose a stable registration point if needed.
    Box::new(|_| {})
}
