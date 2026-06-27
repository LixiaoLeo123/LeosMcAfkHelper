// Shared types mirroring the Rust domain structs in src-tauri/src/domain.rs.

export type Account =
  | { kind: "offline"; username: string }
  | { kind: "microsoft"; login: string };

export interface ServerTarget {
  host: string;
  port: number | null;
}

export interface Facing {
  yaw: number;
  pitch: number;
}

export interface ForceQuitConfig {
  on_totem_pop: boolean;
  on_attacked: boolean;
  on_death: boolean;
  low_health: number | null; // quit when health < N
  chat_regex: string | null; // quit on chat/kick line matching this
}

export interface AutoReconnectConfig {
  enabled: boolean;
  retries: number; // -1 = infinite
  delay_min: number;
  delay_max: number;
  kick_messages: string[];
}

export interface PlayerConfig {
  id: string;
  name: string;
  account: Account;
  server: ServerTarget;
  version_override: string | null; // null = "auto"
  on_join_commands: string[];
  on_join_delay_secs: number;
  facing: Facing | null; // null = keep server default
  behaviors_csharp: string;
  force_quit: ForceQuitConfig;
  auto_totem: boolean;
  auto_reconnect: AutoReconnectConfig;
  eat_when_hungry: boolean;
}

export type PlayerStatus =
  | { kind: "stopped" }
  | { kind: "starting" }
  | { kind: "connected" }
  | { kind: "reconnecting" }
  | { kind: "failed"; reason: string };

export interface LogLine {
  ts: number; // epoch millis
  stream: "stdout" | "stderr";
  text: string;
}

export interface DeviceCodePrompt {
  code: string;
  url: string;
}

export interface PlayerRuntime {
  status: PlayerStatus;
  logs: LogLine[];
}
