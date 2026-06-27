# Leo's Minecraft AFK Helper

A desktop app to manage multiple Minecraft "fake players" that idle or farm on your behalf.
Built with **Tauri 2** (Rust backend) and **Svelte + TypeScript** (frontend), it drives the
[Minecraft Console Client (MCC)](https://github.com/MCCTeam/Minecraft-Console-Client) as
headless subprocesses. One app window controls many players at once.

Supports **Minecraft Java Edition 1.4.6 - 26.1**, including **1.21.11** (whatever the bundled
MCC build supports).

## Features

- **Multiple players.** Add as many fake players as you like; each runs as its own MCC
  process in its own working directory.
- **Server target + version.** Set host/port per player. Version defaults to `auto`
  (negotiated from the server ping); override it to pin a specific version.
- **Accounts.** Offline (cracked) usernames, or Microsoft online accounts via device-code
  login. After the first sign-in, reconnects use the cached refresh token silently.
- **On-join commands.** A list of commands sent automatically after joining
  (e.g. `/is`, `/gamemode survival`).
- **Facing direction.** Set yaw/pitch with presets (North/East/South/West/Up/Down). Leave
  blank to keep the server default. When set, the bot re-applies the look each tick.
- **Behaviors (direct C#).** Each player's behavior is a native MCC `//MCCScript` C# ChatBot
  that runs inside MCC at 20 TPS. Edit it directly in the built-in code editor, with
  ready-made snippets (attack loop, item pickup, eat-when-hungry, move-to, chat responder)
  and a live generated-preview pane.
- **Auto totem.** Toggle to automatically move a Totem of Undying to the off-hand (slot 45)
  when the equipped one is consumed (detected via Entity Status 35).
- **Auto reconnect.** Toggle MCC's built-in AutoRelog: retries, reconnect delay range, and
  kick-message filters.
- **Force-quit conditions.** Stop the bot when: the totem pops, it is attacked (health drops
  while alive), it dies, health drops below a threshold, or a chat/kick line matches a regex.
- **Live console.** Stream each player's MCC stdout/stderr and send commands/chat at runtime.
- **English UI.**

## How it works

```
Tauri app (this repo)
  |
  |- Svelte UI  --invoke-->  Rust backend
  |                             |
  |                             |- renders MinecraftClient.ini (TOML) per player
  |                             |- generates scripts/bot.cs (//MCCScript C# ChatBot)
  |                             |- spawns:  MinecraftClient <ini> BasicIO-NoColor
  |                             |           (CWD = per-player workdir)
  |                             |- streams stdout/stderr -> UI console
  |                             |- sends `script bot`, chat, `reco`, `quit` over stdin
  |
  |- MCC (bundled binary)  ->  Minecraft server
```

The rich gameplay API (inventory moves, entity events, totem-pop detection, combat) is only
reachable from inside a C# ChatBot, so each player's behavior compiles to a `//MCCScript`
bot. Config-driven features (facing, on-join, auto-totem, force-quit) are merged into the
generated bot automatically; you add custom behavior by overriding the `OnUserJoin()`,
`OnUserUpdate()`, and `OnUserChat(string)` hooks.

## Development

Prerequisites: Node.js 20+, Rust (stable), and the
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
```

The MCC binary is bundled as a Tauri resource (`src-tauri/resources/MinecraftClient-linux-x64`).
For development you can override its path with the `MCC_BINARY` environment variable.

### Build a distributable

```bash
npm run tauri build
```

## Layout

- `src/` - Svelte + TypeScript frontend
  - `lib/components/` - PlayerList, PlayerEditor, CodeEditor, Console, DeviceCodePrompt
  - `lib/stores/players.ts` - player state, status, logs
  - `lib/mccTemplates.ts` - behavior snippets
  - `lib/api.ts` - typed Tauri command/event wrappers
- `src-tauri/src/`
  - `domain.rs` - data model (PlayerConfig, Account, Facing, ForceQuit, AutoReconnect...)
  - `store.rs` - persistence (players.json)
  - `commands.rs` - Tauri command handlers
  - `mcc/paths.rs` - MCC binary resolution + per-player workdirs
  - `mcc/config.rs` - MinecraftClient.ini (TOML) renderer
  - `mcc/scriptgen.rs` - C# `//MCCScript` bot generator
  - `mcc/process.rs` - subprocess lifecycle + log/status streaming
  - `mcc/devicecode.rs` - Microsoft device-code prompt parser

## Notes

- The MCC source repo (`Minecraft-Console-Client-Main/`) is included for reference only
  and is not shipped with the app.
- Cross-platform MCC binaries (Windows/macOS) can be added under
  `src-tauri/resources/`; the path resolver picks the right one per platform.
