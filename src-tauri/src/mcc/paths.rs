use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Directory holding all app data (players, configs, logs, MCC binary copy).
pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| anyhow!("app data dir: {e}"))
}

/// Root directory for one player's MCC working files.
pub fn player_workdir(app: &AppHandle, player_id: &str) -> Result<PathBuf> {
    Ok(app_data_dir(app)?.join("players").join(player_id))
}

pub fn player_scripts_dir(app: &AppHandle, player_id: &str) -> Result<PathBuf> {
    Ok(player_workdir(app, player_id)?.join("scripts"))
}

/// Resolve the MCC executable path.
///
/// Strategy:
/// 1. If the env var `MCC_BINARY` is set, use it (developer override).
/// 2. Otherwise look for a copied binary in the app data dir (`mcc/MinecraftClient`).
/// 3. Otherwise fall back to the Tauri-bundled resource (platform-specific name).
pub fn resolve_mcc_binary(app: &AppHandle) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("MCC_BINARY") {
        let p = PathBuf::from(p);
        if p.exists() {
            eprintln!("[resolve_mcc_binary] Using env MCC_BINARY: {:?}", p);
            return Ok(p);
        }
        eprintln!("[resolve_mcc_binary] MCC_BINARY env set but file not found: {:?}", p);
    }

    let data = app_data_dir(app)?;
    let copied = data.join("mcc").join(mcc_binary_name());
    if copied.exists() {
        eprintln!("[resolve_mcc_binary] Using cached copy: {:?}", copied);
        return Ok(copied);
    }

    // Try the bundled resource.
    let resource_name = bundled_resource_name();
    if let Some(p) = app
        .path()
        .resource_dir()
        .ok()
        .map(|d| d.join("resources").join(resource_name))
    {
        if p.exists() {
            eprintln!("[resolve_mcc_binary] Copying bundled resource {:?} -> {:?}", p, copied);
            // Copy into app-data on first use so writes/permissions are predictable.
            std::fs::create_dir_all(copied.parent().unwrap())?;
            std::fs::copy(&p, &copied)?;
            make_executable(&copied)?;
            eprintln!("[resolve_mcc_binary] Copied and made executable: {:?}", copied);
            return Ok(copied);
        }
        eprintln!("[resolve_mcc_binary] Resource dir exists but bundled binary not found: {:?}", p);
    } else {
        eprintln!("[resolve_mcc_binary] Could not determine resource_dir");
    }

    eprintln!(
        "[resolve_mcc_binary] FAILED: Could not locate MCC binary. \
         Set the MCC_BINARY environment variable to its path."
    );
    Err(anyhow!(
        "Could not locate the Minecraft Console Client binary. \
         Set the MCC_BINARY environment variable to its path."
    ))
}

fn mcc_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "MinecraftClient.exe"
    } else {
        "MinecraftClient"
    }
}

fn bundled_resource_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "MinecraftClient-windows-x64.exe"
    } else if cfg!(target_os = "macos") {
        "MinecraftClient-osx-x64"
    } else {
        "MinecraftClient-linux-x64"
    }
}

pub fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

/// Ensure the per-player working directory and scripts subdir exist.
pub fn ensure_player_dirs(app: &AppHandle, player_id: &str) -> Result<PathBuf> {
    let workdir = player_workdir(app, player_id)?;
    std::fs::create_dir_all(&workdir)
        .with_context(|| format!("creating workdir {:?}", workdir))?;
    std::fs::create_dir_all(workdir.join("scripts"))?;
    Ok(workdir)
}
