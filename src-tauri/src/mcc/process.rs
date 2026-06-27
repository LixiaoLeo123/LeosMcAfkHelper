use crate::domain::{LogLine, PlayerConfig, PlayerStatus};
use crate::mcc::config::render_ini;
use crate::mcc::devicecode::parse_device_code;
use crate::mcc::paths::{ensure_player_dirs, make_executable, player_workdir, resolve_mcc_binary};
use crate::mcc::scriptgen::generate_script;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin};
use tokio::sync::Mutex;

/// A live MCC subprocess for one player.
pub struct PlayerHandle {
    pub stdin: Arc<Mutex<Option<ChildStdin>>>,
    pub child: Arc<Mutex<Option<Child>>>,
}

/// Global registry of running players.
pub struct Runtime {
    pub handles: DashMap<String, Arc<PlayerHandle>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            handles: DashMap::new(),
        }
    }
}

/// Write the player's MCC config + script into its workdir, then spawn MCC.
pub async fn start_player(app: &AppHandle, p: &PlayerConfig) -> Result<()> {
    let runtime = app.state::<Runtime>();
    if runtime.handles.contains_key(&p.id) {
        return Err(anyhow!("player {} is already running", p.name));
    }

    // 1. Ensure dirs + write files.
    let workdir = ensure_player_dirs(app, &p.id)
        .map_err(|e| {
            eprintln!("[startPlayer] FAILED ensure_player_dirs for '{}': {e:#}", p.name);
            e
        })?;
    let ini_path = workdir.join("MinecraftClient.ini");
    let script_path = workdir.join("scripts").join("bot.cs");
    eprintln!("[startPlayer] workdir for '{}': {:?}", p.name, workdir);

    let ini = render_ini(p).map_err(|e| {
        eprintln!("[startPlayer] FAILED render_ini for '{}': {e:#}", p.name);
        e
    })?;
    let script = generate_script(p).map_err(|e| {
        eprintln!("[startPlayer] FAILED generate_script for '{}': {e:#}", p.name);
        e
    })?;
    if !script.starts_with("//MCCScript 1.0") {
        let msg = "generated script has invalid header";
        eprintln!("[startPlayer] FAILED for '{}': {msg}", p.name);
        return Err(anyhow!(msg));
    }
    std::fs::write(&ini_path, &ini)
        .with_context(|| format!("writing {:?}", ini_path))
        .map_err(|e| {
            eprintln!("[startPlayer] FAILED write ini for '{}': {e:#}", p.name);
            e
        })?;
    std::fs::write(&script_path, &script)
        .with_context(|| format!("writing {:?}", script_path))
        .map_err(|e| {
            eprintln!("[startPlayer] FAILED write script for '{}': {e:#}", p.name);
            e
        })?;

    // 2. Resolve the MCC binary.
    let mcc = resolve_mcc_binary(app).map_err(|e| {
        eprintln!("[startPlayer] FAILED resolve_mcc_binary for '{}': {e:#}", p.name);
        e
    })?;
    eprintln!("[startPlayer] resolved MCC binary for '{}': {:?}", p.name, mcc);
    make_executable(&mcc).map_err(|e| {
        eprintln!("[startPlayer] FAILED make_executable for '{}': {e:#}", p.name);
        e
    })?;

    // 3. Spawn MCC: <binary> <ini> BasicIO-NoColor, CWD = workdir.
    set_status(app, &p.id, PlayerStatus::Starting);
    eprintln!("[startPlayer] Spawning MCC for '{}': {:?} {:?} BasicIO-NoColor (cwd: {:?})", p.name, mcc, ini_path, workdir);
    emit_log(
        app,
        &p.id,
        "stdout",
        &format!("Starting MCC for '{}' -> {}", p.name, p.server.to_address()),
    );

    let mut command = tokio::process::Command::new(&mcc);
    command
        .arg(&ini_path)
        .arg("BasicIO-NoColor")
        .current_dir(&workdir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    // On Unix, put the child in its own session so we can clean up the whole tree.
    #[cfg(unix)]
    {
        unsafe {
            command.pre_exec(|| {
                setsid_raw();
                Ok(())
            });
        }
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("spawning MCC at {:?}", mcc))
        .map_err(|e| {
            eprintln!("[startPlayer] FAILED spawn for '{}': {e:#}", p.name);
            e
        })?;
    eprintln!("[startPlayer] MCC spawned for '{}', pid: {:?}", p.name, child.id());

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to capture MCC stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture MCC stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture MCC stderr"))?;

    let stdin = Arc::new(Mutex::new(Some(stdin)));
    let child = Arc::new(Mutex::new(Some(child)));

            // Load the behavior script shortly after startup via stdin.
            //
            // MCC's stdin dispatcher only treats a line as an *internal* command when it is
            // prefixed with the configured InternalCmdChar (here "slash" => "/"). A bare
            // "script bot" is instead sent to the server as ordinary chat — which both fails
            // to load the bot AND leaks the text into public chat. So we must send "/script".
            {
                let s = stdin.clone();
                let name = p.name.clone();
                let id_for_log = p.id.clone();
                let app_for_log = app.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    match send_command_raw(&s, "/script bot").await {
                        Ok(_) => emit_log(
                            &app_for_log,
                            &id_for_log,
                            "stdout",
                            &format!("Sent '/script bot' to load behavior for '{}'.", name),
                        ),
                        Err(e) => emit_log(
                            &app_for_log,
                            &id_for_log,
                            "stderr",
                            &format!("Failed to load behavior script: {e}"),
                        ),
                    }
                });
            }

    let handle = Arc::new(PlayerHandle {
        stdin: stdin.clone(),
        child: child.clone(),
    });
    runtime.handles.insert(p.id.clone(), handle);

    spawn_output_reader(app.clone(), p.id.clone(), stdout, "stdout");
    spawn_output_reader(app.clone(), p.id.clone(), stderr, "stderr");
    spawn_status_watcher(app.clone(), p.id.clone(), child);

    Ok(())
}

fn spawn_output_reader<R>(app: AppHandle, player_id: String, out: R, stream: &'static str)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(out);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app, &player_id, stream, &line);
            if let Some(prompt) = parse_device_code(&line) {
                let _ = app.emit(&format!("player:{player_id}:devicecode"), &prompt);
            }
            let lower = line.to_lowercase();
            if lower.contains("successfully joined")
                || lower.contains("logged in")
                || lower.contains("connected to")
            {
                set_status(&app, &player_id, PlayerStatus::Connected);
            } else if lower.contains("retrying") || lower.contains("reconnecting") {
                set_status(&app, &player_id, PlayerStatus::Reconnecting);
            }
        }
    });
}

fn spawn_status_watcher(app: AppHandle, player_id: String, child: Arc<Mutex<Option<Child>>>) {
    tokio::spawn(async move {
        loop {
            let exited = {
                let mut guard = child.lock().await;
                match guard.as_mut() {
                    Some(c) => match c.try_wait() {
                        Ok(Some(_status)) => {
                            *guard = None; // drop the child
                            true
                        }
                        _ => false,
                    },
                    None => true, // already taken/dropped
                }
            };
            if exited {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        let runtime = app.state::<Runtime>();
        runtime.handles.remove(&player_id);
        set_status(&app, &player_id, PlayerStatus::Stopped);
        emit_log(&app, &player_id, "stdout", "MCC process exited.");
    });
}

/// Send a line of text to a running player's MCC stdin.
pub async fn send_command(app: &AppHandle, player_id: &str, command: &str) -> Result<()> {
    let runtime = app.state::<Runtime>();
    let handle = runtime
        .handles
        .get(player_id)
        .ok_or_else(|| anyhow!("player is not running"))?
        .clone();
    send_command_raw(&handle.stdin, command).await
}

async fn send_command_raw(stdin: &Arc<Mutex<Option<ChildStdin>>>, command: &str) -> Result<()> {
    let mut guard = stdin.lock().await;
    if let Some(s) = guard.as_mut() {
        s.write_all(command.as_bytes()).await?;
        s.write_all(b"\n").await?;
        s.flush().await?;
        Ok(())
    } else {
        Err(anyhow!("MCC stdin closed"))
    }
}

/// Stop a running player (graceful `exit`, then kill).
pub async fn stop_player(app: &AppHandle, player_id: &str) -> Result<()> {
    let runtime = app.state::<Runtime>();
    let handle = match runtime.handles.get(player_id).map(|h| h.clone()) {
        Some(h) => h,
        None => return Ok(()),
    };
    // "/exit" (not bare "exit"): MCC's stdin dispatcher only honors an internal command
    // when it carries the InternalCmdChar prefix ("/" here); a bare "exit" is sent to the
    // server as chat and never terminates the process.
    let _ = send_command_raw(&handle.stdin, "/exit").await;
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    {
        let mut guard = handle.child.lock().await;
        if let Some(c) = guard.as_mut() {
            let _ = c.start_kill();
        }
    }
    runtime.handles.remove(player_id);
    set_status(app, player_id, PlayerStatus::Stopped);
    Ok(())
}

/// Is a player currently running?
pub fn is_running(app: &AppHandle, player_id: &str) -> bool {
    let runtime = app.state::<Runtime>();
    runtime.handles.contains_key(player_id)
}

fn set_status(app: &AppHandle, player_id: &str, status: PlayerStatus) {
    let _ = app.emit(&format!("player:{player_id}:status"), &status);
}

fn emit_log(app: &AppHandle, player_id: &str, stream: &str, text: &str) {
    let line = LogLine {
        ts: Utc::now().timestamp_millis(),
        stream: stream.to_string(),
        text: text.to_string(),
    };
    let _ = app.emit(&format!("player:{player_id}:log"), &line);
    // Also push into the in-memory ring buffer for history queries.
    if let Some(buf) = app.try_state::<crate::commands::LogBuffer>() {
        let buf = buf.inner().clone();
        let id = player_id.to_string();
        let line = line.clone();
        tauri::async_runtime::spawn(async move {
            buf.push(&id, line).await;
        });
    }
}

pub fn workdir_for(app: &AppHandle, player_id: &str) -> Result<PathBuf> {
    player_workdir(app, player_id)
}

#[cfg(unix)]
unsafe fn setsid_raw() {
    extern "C" {
        fn setsid() -> i32;
    }
    let _ = setsid();
}

/// Kill all running MCC processes — called during app shutdown.
pub async fn kill_all_players(app: &AppHandle) {
    let runtime = app.state::<Runtime>();
    let ids: Vec<String> = runtime.handles.iter().map(|e| e.key().clone()).collect();
    for id in &ids {
        eprintln!("[shutdown] Killing MCC for player '{id}'");
        if let Some(handle) = runtime.handles.get(id) {
            let mut guard = handle.child.lock().await;
            if let Some(c) = guard.as_mut() {
                let _ = c.start_kill();
            }
        }
        runtime.handles.remove(id);
    }
}
