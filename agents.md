# Looking Glass — Agent Instructions

## Project Context
**Looking Glass** is a Rust reimplementation of American McGee's Alice (2011) using the Macroquad game framework, **now pivoting to a functional-first game & mod launcher as a console application first before making a GUI launcher**.

The existing engine (`looking-glass` binary) serves as the primary test game for the launcher. The launcher will manage this and other games (Steam, Heroic, Flatpak, Wine/Proton, native).

**Key files to understand:**
- `README.md` — Project overview, status, install instructions
- `Cargo.toml` — Dependencies, Rust version (1.86), profile config (will become workspace root)
- `src/main.rs` — Current game entry point (1000+ lines, will remain as game binary)
- `WHERE_WE_ARE.md` — Project status tracker and roadmap (read this first)
- `tools/package_linux.py` — Packaging script for Linux releases

## Architecture Vision

### Launcher Crate Structure (Future Workspace)
```
looking-glass-launcher/
├── looking-glass-core/        # Game registry, launch abstraction, config
├── looking-glass-mod/         # Mod management, sources, installation
├── looking-glass-platform/    # Steam, Heroic, Flatpak, Wine/Proton integrations
├── looking-glass-tui/         # ratatui-based console interface
├── looking-glass-save/        # Save backup/restore/sync
├── looking-glass-cli/         # CLI binary (lg-launch)
└── looking-glass/             # Existing game engine binary (unchanged)
```

### Key Principles
- **Library-first**: Core logic in reusable crates, thin CLI/TUI wrappers
- **No game data**: Never bundle game files; only manage launch configuration
- **Cross-platform**: Linux primary, Windows/macOS via `#[cfg]`
- **Unix-native**: Leverages system tools (flatpak, steam, wine) rather than reimplementing
- **Console-first**: TUI/CLI before GUI; GUI shares core library later

---

## Current Engine Architecture (Reference)

### Core Modules (src/)
| Area | Modules | Purpose |
|------|---------|---------|
| **Engine** | `main.rs`, `viewer.rs`, `render.rs`, `assets.rs`, `bsp.rs` | Window, rendering, asset loading, BSP parsing |
| **Gameplay** | `campaign*.rs`, `character.rs`, `movement.rs`, `combat.rs`, `weapons.rs` | Campaign flow, Alice, locomotion, combat, weapons |
| **World** | `level.rs`, `levels/`, `collision.rs`, `interaction.rs`, `traversal.rs` | Maps, physics, doors, ropes, swimming, ledges |
| **NPCs** | `npc.rs`, `ant.rs`, `duchess.rs`, `chess.rs`, `fire_imp.rs`, etc. | Enemy/entity implementations |
| **Systems** | `audio.rs`, `input.rs`, `menu.rs`, `save.rs`, `inventory.rs` | Audio, input, UI, saves, inventory |
| **Validation** | `*_check.rs`, `*_render.rs`, `campaign_chain.rs` | Automated tests, visual regression, campaign verification |

### Key Patterns (Engine)
- **Mode-based CLI**: `main.rs` dispatches 100+ `--mode` flags to check functions
- **Headless vs Windowed**: Checks run either headless (`Run::Headless`) or with window (`Run::Windowed`)
- **Asset system**: `Assets::open(path)` loads PK3/BSP/FTX from game data folder
- **Campaign chain**: `campaign_chain.rs` runs full 39-visit campaign with save/load boundaries

---

## Launcher Development Tasks

### Phase 1: Core Launcher Infrastructure (Console/TUI)

#### 1.1 Game Registry (`looking-glass-core`)
- [ ] **GameEntry struct**: id, name, executable, args, env, prefix, platform, metadata
- [ ] **Registry scanner**: Steam (libraryfolders.vdf), Heroic (config), Flatpak (flatpak list), Lutris, AppImage, native binaries
- [ ] **Platform detection**: Enum `Platform { Steam, Heroic, Flatpak, Lutris, Wine, Proton, Native, AppImage }`
- [ ] **Persistence**: Serialize registry to TOML (`~/.config/looking-glass/registry.toml`)

#### 1.2 Launch Abstraction (`looking-glass-core`)
- [ ] **LaunchCommand trait**: `build_cmd(&self) -> Command` per platform
- [ ] **Steam**: `steam://rungameid/<id>` or `steam -applaunch <id>`
- [ ] **Flatpak**: `flatpak run <app-id> --args`
- [ ] **Wine/Proton**: `WINEPREFIX=... wine /path/to/game.exe` or `proton run`
- [ ] **Native**: Direct binary execution with LD_LIBRARY_PATH handling
- [ ] **AppImage**: Execute with proper sandboxing
- [ ] **Environment management**: Per-game env vars, LD_PRELOAD (e.g., gamemode, mangohud)

#### 1.3 Config System (`looking-glass-core`)
- [ ] **Config struct**: TOML with `game`, `launcher`, `platform`, `mods`, `save` sections
- [ ] **Config paths**: XDG (`~/.config/looking-glass/config.toml`), project-local (`private/config.toml`)
- [ ] **Layered config**: Defaults → global → project → CLI overrides
- [ ] **Validation**: Serde with `#[serde(deny_unknown_fields)]`, custom validators

#### 1.4 TUI Interface (`looking-glass-tui`)
- [ ] **ratatui + crossterm** setup with alternate screen, raw mode
- [ ] **Game list view**: Scrollable, searchable, filterable (platform, tags, installed)
- [ ] **Game detail view**: Launch args, mods, saves, metadata
- [ ] **Launch action**: Spawn detached, show output in overlay or log view
- [ ] **Keybindings**: vim-style (j/k, /, Enter, q, r refresh, p profile switch)

#### 1.5 Mod Profile System (`looking-glass-mod`)
- [ ] **Mod struct**: id, name, version, source, files, dependencies, load_order
- [ ] **Profile struct**: name, enabled_mods (ordered list), game_id
- [ ] **Mod sources**: Local dir, NexusMods API, Mod.io, Git repo
- [ ] **Installation**: Extract to game mod folder, track manifest for uninstall
- [ ] **Load order**: LOOT-style sorting, conflict detection, user override

#### 1.6 Save Management (`looking-glass-save`)
- [ ] **Save discovery**: Platform-specific paths (Steam cloud, Wine prefix, Flatpak data)
- [ ] **Backup**: Timestamped archives to `~/.local/share/looking-glass/saves/<game>/`
- [ ] **Restore**: List backups, preview, restore with confirmation
- [ ] **Sync**: Optional rclone/Git/Syncthing integration for cloud backup

---

### Phase 2: Mod Management (Detailed)

| Task | Description |
|------|-------------|
| NexusMods API | OAuth, mod search, download, version tracking |
| Mod.io API | Cross-platform mod distribution |
| Archive handling | zip, 7z, rar, tar.gz with streaming extraction |
| Conflict resolution | File-level conflict UI, merge strategies |
| Profile import/export | Share mod lists as JSON/TOML |

---

### Phase 3: Platform Integration

| Task | Description |
|------|-------------|
| Steam library parsing | `libraryfolders.vdf` → installed apps, Proton versions |
| Heroic/Gog | Wine prefix management, GOG Galaxy compatibility |
| Flatpak permissions | `--filesystem`, `--device` overrides per game |
| Steam Deck | Deck UI detection, controller navigation hints |
| Desktop entries | Generate `.desktop` with custom icons, categories |

---

### Phase 4: Advanced Features

| Task | Description |
|------|-------------|
| Benchmarking | FPS capture, frame time graphs, comparison reports |
| Cloud sync | Optional save/mod profile sync via Git/rclone |
| Plugin system | Lua (mlua) or WASM (wasmtime) for custom logic |
| GUI prototype | Tauri + egui frontend sharing `looking-glass-core` |

---

## Development Workflow

### Building (Current Engine)
```bash
# Standard build
cargo build --release --locked

# Run from source (data path auto-detected from private/data-path.txt)
./target/release/looking-glass

# Run with explicit data path
./target/release/looking-glass --data /path/to/alice_202106/Alice1/bin/base --map skool1

# List available maps
./target/release/looking-glass --list

# Run validation suite
./target/release/looking-glass --validate-all
```

### Testing Commands
```bash
cargo test --locked
cargo build --release --locked
python tools/check_source.py --worktree
python -m unittest discover -s tools -p "test_source_package.py"
```

### Key Validation Modes (Engine)
| Mode | Purpose |
|------|---------|
| `--validate-all` | All maps, colliders, FTX images, materials |
| `--campaign-route-check` | Full 39-visit campaign route (headless) |
| `--campaign-legs-check` | Opening 8 route legs only |
| `--campaign-playtest` | Full playable campaign with window |
| `--physics-check` | Deterministic walk/jump on skool1 |
| `--character-check` | Alice mesh + all animation clips |
| `--registry-check` | Fingerprint all 39 campaign entries |

---

## Code Conventions

### Rust Style
- Edition 2021, Rust 1.86 minimum
- `anyhow::Result` for errors, `bail!` for early returns
- Modules declared in `main.rs` (100+ `mod` statements) — **launcher will use proper workspace crates**
- Heavy use of `macroquad::prelude::*` for math/graphics (engine only)
- `glam` for Vec2/Vec3/Mat4 with serde support
- **New crate conventions**: `clap` for CLI, `ratatui`/`crossterm` for TUI, `serde` + `toml` for config, `tracing` for logging

### Testing Pattern (Engine)
```rust
// Headless check returning Result
pub fn check(assets: &mut Assets) -> Result<()> { ... }

// Windowed check using Macroquad
pub async fn render_check(assets: &mut Assets) -> Result<()> { ... }

// In main.rs dispatch:
if mode == "my-check" {
    return my_module::check(&mut assets);
}
```

### Launcher Testing Pattern (New)
```rust
// Unit tests for core logic
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_scan_steam() { ... }
    
    #[test]
    fn test_launch_command_steam() { ... }
    
    #[test]
    fn test_config_layering() { ... }
}

// Integration tests in tests/
// #[test]
// fn test_full_launch_flow() { ... }
```

---

## Platform-Specific Code Locations (Engine - Preserved)

| File | Function | Windows | Linux |
|------|----------|---------|-------|
| `build.rs` | Stack size | `/STACK:8388608` | `-Wl,-z,stack-size=8388608` |
| `src/look.rs` | `window_focused()` | user32.dll | x11rb → xdotool → `true` |
| `src/audio/regression.rs` | Device check | Windows-specific msg | Generic message |
| `src/input.rs` | Gamepad | XInput | gilrs (--features gamepad) |

### Conditional Compilation Pattern
```rust
#[cfg(target_os = "windows")]
pub fn platform_specific() { ... }

#[cfg(target_os = "linux")]
pub fn platform_specific() { ... }

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn platform_specific() { ... } // macOS, etc.
```

---

## Dependencies

### Engine (Pinned - Do Not Upgrade Without Testing)
| Crate | Version | Purpose |
|-------|---------|---------|
| `macroquad` | =0.4.14 | Game framework (miniquad) |
| `rodio` | =0.20.1 | Audio (wav, mp3 via symphonia) |
| `zip` | =0.6.6 | PK3 reading |
| `image` | =0.24.9 | TGA/JPEG/PNG loading |
| `glam` | =0.27.0 | Math (Vec/Mat/Quat) |
| `serde` | latest | Serialization |
| `anyhow` | latest | Errors |

### Launcher (New Dependencies - To Be Added)
| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `ratatui` | TUI framework |
| `crossterm` | Terminal manipulation |
| `toml` | Config parsing |
| `directories` | XDG path resolution |
| `walkdir` | Filesystem scanning |
| `notify` | Filesystem watching |
| `mlua` / `wasmtime` | Plugin system (Phase 4) |
| `reqwest` | HTTP for mod APIs |
| `rmp-serde` / `bincode` | Save serialization |

---

## Game Data Requirements (Engine)
- **Source**: `Alice1_2011_vanilla.7z` from Internet Archive (alice_202106)
- **Required PK3s**: `pak0.pk3`, `pak1_large.pk3`, `pak2.pk3`, `pak3.pk3`, `pak5_mod.pk3`
- **Language packs**: `pak4_english.pk3` + `base_g.pk3` (de), `base_f.pk3` (fr), `base_s.pk3` (es), `base_d.pk3` (nl)
- **Setup copies** to `private/game-data/` — original files untouched
- **Launcher never bundles game data** — only manages paths

---

## Common Issues & Fixes

| Issue | Cause | Fix |
|-------|-------|-----|
| Stack overflow on level transition | Default 1MB stack too small | Set 8MB in `build.rs` for Linux |
| Window focus not detected | `window_focused()` returns `true` on Linux | Implement X11 focus query (done) |
| Audio device not found | Rodio backend selection | Ensure `libasound2`/`pipewire` installed |
| 7z extraction fails | `p7zip` not installed | `apt install p7zip-full` / `dnf install p7zip` |
| GUI dialogs don't appear | No `zenity`/`kdialog`/`yad` | Install or use CLI fallback |
| Macroquad window doesn't open | Missing OpenGL/Wayland libs | `apt install libgl1-mesa-dev libx11-dev libwayland-dev` |

---

## Git Workflow
- Main branch: `main`
- Commits: Conventional (feat/fix/docs/refactor/test/chore)
- PRs: Required for all changes
- CI: Must pass on Windows + Linux before merge
- **Launcher**: Separate feature branches per crate/module

---

## Resources
- **Website**: https://looking-glass-project.github.io/LookingGlass/
- **Original Repo**: https://github.com/looking-glass-project/LookingGlass
- **Fork**: https://github.com/SaraEatsPickledPeppers/LookingGlass
- **Game data**: https://archive.org/details/alice_202106
- **Macroquad**: https://github.com/not-fl3/macroquad
- **Rodio**: https://github.com/RustAudio/rodio
- **ratatui**: https://github.com/ratatui/ratatui
- **clap**: https://github.com/clap-rs/clap

---

## Agent Notes (Updated for Launcher Development)

- **Read `WHERE_WE_ARE.md` first** for full roadmap and current status
- **Preserve engine**: The `looking-glass` game binary must keep working; launcher wraps it
- **Library-first**: Build `looking-glass-core` as a reusable crate, not a binary
- **Small agents**: One agent per crate or well-defined feature (registry, launch, mod, platform, tui, save, cli)
- **Test on Linux** — primary target; Windows/macOS via CI
- **No game data in repo** — never commit PK3/BSP/FTX files
- **Check `tools/check_source.py`** before committing media files
- **XDG compliance**: Use `directories` crate for config/data/cache paths
- **Error handling**: `anyhow::Result` with context; user-friendly messages in TUI/CLI
- **Logging**: `tracing` with structured JSON to `~/.local/share/looking-glass/logs/`

---

## Agent Deployment Strategy (Launcher)

**Deploy many small agents:**
1. **Core crate agents** (one per crate): `core`, `mod`, `platform`, `tui`, `save`, `cli`
2. **Feature agents** within crates: e.g., "Steam scanner", "Flatpak launcher", "Mod installer"
3. **Integration agents**: Config layering, CLI↔TUI shared state, error handling
4. **Test agents**: Unit tests per module, integration tests for launch flows

**Avoid:** Large agents that try to build entire subsystems — they stall (~10 min watchdog).

**Results target:** Each agent completes in <5 minutes. Phase 1 = ~7 agents.