use crate::domain::PlayerConfig;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Persists the list of players to `players.json` in the app data dir.
pub struct Store {
    pub dir: PathBuf,
    pub players: Mutex<Vec<PlayerConfig>>,
}

impl Store {
    pub fn load(app: &AppHandle) -> Result<Self> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("app data dir: {e}"))?;
        std::fs::create_dir_all(&dir)?;
        let file = dir.join("players.json");
        let players = if file.exists() {
            let data = std::fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(Self {
            dir,
            players: Mutex::new(players),
        })
    }

    pub fn list(&self) -> Vec<PlayerConfig> {
        self.players.lock().unwrap().clone()
    }

    pub fn get(&self, id: &str) -> Option<PlayerConfig> {
        self.players
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.id == id)
            .cloned()
    }

    pub fn upsert(&self, player: PlayerConfig) -> Result<()> {
        {
            let mut guard = self.players.lock().unwrap();
            if let Some(existing) = guard.iter_mut().find(|p| p.id == player.id) {
                *existing = player;
            } else {
                guard.push(player);
            }
        }
        self.persist()
    }

    pub fn add(&self, player: PlayerConfig) -> Result<()> {
        self.players.lock().unwrap().push(player);
        self.persist()
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        self.players.lock().unwrap().retain(|p| p.id != id);
        self.persist()
    }

    fn persist(&self) -> Result<()> {
        let file = self.dir.join("players.json");
        let guard = self.players.lock().unwrap();
        let data = serde_json::to_string_pretty(&*guard)?;
        std::fs::write(&file, data)
            .with_context(|| format!("writing {:?}", file))?;
        Ok(())
    }
}
