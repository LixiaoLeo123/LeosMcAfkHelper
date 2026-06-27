use serde::{Deserialize, Serialize};

/// Account credentials for a fake player.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Account {
    /// Offline/cracked-mode account. MCC uses Password = "-".
    Offline { username: String },
    /// Microsoft online account (device-code login, cached refresh token).
    Microsoft { login: String },
}

impl Account {
    /// The username shown in-game / used for the session.
    pub fn effective_username(&self) -> &str {
        match self {
            Account::Offline { username } => username,
            Account::Microsoft { login } => login,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTarget {
    pub host: String,
    /// None => MCC auto-resolves (SRV / default 25565).
    #[serde(default)]
    pub port: Option<u16>,
}

impl ServerTarget {
    /// "host" or "host:port".
    pub fn to_address(&self) -> String {
        match self.port {
            Some(p) => format!("{}:{}", self.host, p),
            None => self.host.clone(),
        }
    }
}

/// Facing direction the fake player holds. None => keep server default
/// (we simply never call LookAtLocation).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Facing {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceQuitConfig {
    #[serde(default)]
    pub on_totem_pop: bool,
    #[serde(default)]
    pub on_attacked: bool, // health drop while alive
    #[serde(default)]
    pub on_death: bool,
    /// Quit when health drops below this value.
    #[serde(default)]
    pub low_health: Option<f32>,
    /// Quit when a chat/kick line matches this regex.
    #[serde(default)]
    pub chat_regex: Option<String>,
}

impl Default for ForceQuitConfig {
    fn default() -> Self {
        Self {
            on_totem_pop: false,
            on_attacked: false,
            on_death: false,
            low_health: None,
            chat_regex: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReconnectConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_retries")]
    pub retries: i32, // -1 = infinite
    #[serde(default = "default_delay_min")]
    pub delay_min: f32,
    #[serde(default = "default_delay_max")]
    pub delay_max: f32,
    #[serde(default = "default_kick_messages")]
    pub kick_messages: Vec<String>,
}

fn default_retries() -> i32 {
    -1
}
fn default_delay_min() -> f32 {
    8.0
}
fn default_delay_max() -> f32 {
    60.0
}
fn default_kick_messages() -> Vec<String> {
    vec![
        "Connection has been lost".into(),
        "Server is restarting".into(),
        "Server is full".into(),
    ]
}

impl Default for AutoReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retries: default_retries(),
            delay_min: default_delay_min(),
            delay_max: default_delay_max(),
            kick_messages: default_kick_messages(),
        }
    }
}

/// Full configuration for one fake player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub id: String,
    pub name: String,
    pub account: Account,
    pub server: ServerTarget,
    /// None => "auto" (server-ping negotiation).
    #[serde(default)]
    pub version_override: Option<String>,
    #[serde(default)]
    pub on_join_commands: Vec<String>,
    /// Delay in seconds before on-join commands are sent (default 0).
    #[serde(default)]
    pub on_join_delay_secs: f32,
    #[serde(default)]
    pub facing: Option<Facing>,
    #[serde(default)]
    pub behaviors_csharp: String,
    #[serde(default)]
    pub force_quit: ForceQuitConfig,
    #[serde(default)]
    pub auto_totem: bool,
    #[serde(default)]
    pub auto_reconnect: AutoReconnectConfig,
    #[serde(default)]
    pub eat_when_hungry: bool,
}

impl PlayerConfig {
    /// Create a new player with sensible defaults and a starter C# template.
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            account: Account::Offline {
                username: "Steve".into(),
            },
            server: ServerTarget {
                host: "localhost".into(),
                port: Some(25565),
            },
            version_override: None,
            on_join_commands: Vec::new(),
            on_join_delay_secs: 0.0,
            facing: None,
            behaviors_csharp: default_behavior_template(),
            force_quit: ForceQuitConfig::default(),
            auto_totem: false,
            auto_reconnect: AutoReconnectConfig::default(),
            eat_when_hungry: false,
        }
    }
}

/// Live runtime status of a player's MCC subprocess.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PlayerStatus {
    Stopped,
    Starting,
    Connected,
    Reconnecting,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    /// Epoch millis.
    pub ts: i64,
    pub stream: String, // "stdout" | "stderr"
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodePrompt {
    pub code: String,
    pub url: String,
}

/// The default C# behavior script a new player starts with.
pub fn default_behavior_template() -> String {
    r#"//MCCScript 1.0
// Leo's AFK Helper - behavior script. Edit freely. Runs inside MCC at ~20 ticks/sec.
// Config-driven features (facing, on-join commands, auto-totem, force-quit) are applied
// automatically by the generated bot. Override the OnUser* hooks below to add behavior.
MCC.LoadBot(new AfkBot());

//MCCScript Extensions
using System;
using System.Text.RegularExpressions;
using MinecraftClient;
using MinecraftClient.Scripting;
using MinecraftClient.Mapping;
using MinecraftClient.Inventory;

public class AfkBot : ChatBot
{
    // Called once after each server join (after on-join commands and facing are applied).
    public virtual void OnUserJoin() { }

    // Called ~20 times per second. Keep it cheap; never block.
    // Example attack loop (uncomment):
    //   if (!ClientIsMoving())
    //   {
    //       var target = Game.FindNearestEntity(typeFilter: "mob", radius: 4.0);
    //       if (target.Ok() && target.Data.Entity != null)
    //       {
    //           LookAtLocation(target.Data.Entity.Location);
    //           InteractEntity(target.Data.Entity.ID, InteractType.Attack);
    //       }
    //   }
    // Example eat loop (uncomment):
    //   var sel = Game.SelectHotbarItem("minecraft:cooked_beef");
    //   if (sel.Ok()) UseItemInHand();
    public virtual void OnUserUpdate() { }

    // Called when a chat/kick line arrives (verbatim text, color codes stripped).
    // Example (uncomment):
    //   string message = "", sender = "";
    //   if (IsPrivateMessage(text, ref message, ref sender) && message == "!hello")
    //       SendPrivateMessage(sender, "Hi there!");
    public virtual void OnUserChat(string text) { }
}
"#.to_string()
}
