# Looking Glass — Agent Instructions

## Project Context
**Looking Glass** is a Rust reimplementation of American McGee's Alice (2011) using the Macroquad game framework. The engine is complete and playable (39 visits).

**Current focus:** A functional, Alice-only console launcher (`lg-launch`) with proof-of-concept mod support. No game registry, no multi-platform abstraction, no TUI — just a solid CLI that launches Alice with mods.

The engine (`looking-glass` binary) is the test bed. Mods load into the running engine via the mod system.

---

## Architecture (Simplified)

```
LookingGlass/
├── crates/
│   ├── looking-glass-engine/   # Game binary — MOD HOOKS GO HERE
│   ├── looking-glass-mod/      # Mod manifest, registry, discovery, installation
│   ├── looking-glass-config/   # Config layering (XDG + project-local TOML)
│   ├── looking-glass-cli/      # lg-launch binary (Alice-only)
│   └── button-glyphs/          # Controller glyph assets
├── mods/
│   └── Alice/                  # Per-game mod directory
│       ├── vanilla/            # Built-in: no mods
│       ├── cheats/             # Built-in: cheat mod
│       ├── button-glyphs/      # Built-in: glyph mod
│       └── <user-mods>/        # User-installed mods
├── private/
│   ├── data-path.txt           # Resolved game data path
│   ├── config.toml             # Project-local config override
│   └── saves/                  # Save files
├── tools/                      # Packaging, validation scripts
└── tests/                      # Integration tests (engine + launcher + mods)
```

**Key principles:**
- **Single entry point**: `lg-launch` (or `cargo run --bin lg-launch`)
- **Config-driven**: TOML at XDG + project-local, layered
- **Mod system lives in engine**: Launcher only resolves paths + builds env vars
- **No game registry**: Alice path from config or `private/data-path.txt`
- **No platform abstraction**: Engine runs natively; launcher just execs it

---

## Key Files for Launcher/Mod Work

| File | Purpose |
|------|---------|
| `crates/looking-glass-cli/src/main.rs` | `lg-launch` entry point |
| `crates/looking-glass-cli/src/launcher.rs` | Launch options, mod discovery, command building |
| `crates/looking-glass-cli/src/profiles.rs` | Mod profiles, launcher config |
| `crates/looking-glass-mod/src/` | Mod manifest, registry, discovery, installation |
| `crates/looking-glass-engine/src/mod_integration.rs` | Engine-side mod loading, env var injection |
| `crates/looking-glass-config/src/` | Config layering, XDG paths |
| `private/data-path.txt` | Resolved Alice data path (written by Setup.sh) |

---

## Engine Architecture (Reference for Mod Hooks)

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

## Mod Hook Points (Engine Side)

These are the integration points mods can use. Defined in `mod_integration.rs` and engine modules:

| Hook Point | Location | Purpose |
|------------|----------|---------|
| **Menu injection** | `menu.rs` | Add tabs to main/pause/options menus |
| **HUD overlay** | `hud.rs`, `render.rs` | Render custom HUD elements, debug overlay |
| **Console commands** | `console.rs` | Register `warp`, `give`, `noclip`, etc. |
| **Input handling** | `input.rs` | Custom keybinds, mouse handling in edit mode |
| **Config persistence** | `preferences.rs` | Mod-specific config read/write |
| **Map/level transitions** | `level.rs`, `campaign.rs` | Warp hooks, save state manipulation |
| **Player state** | `character.rs`, `inventory.rs` | God mode, noclip, item spawning |
| **Render hooks** | `render.rs`, `render_fx.rs` | Custom post-process, debug visualization |

**Adding a hook:** Edit the relevant engine module, expose a callback registry in `mod_integration.rs`, document the signature.

---

## Development Workflow

### Building
```bash
# Standard build
cargo build --release --locked

# Run launcher (uses private/data-path.txt or config)
cargo run --bin lg-launch -- --profile vanilla

# Run engine directly (for debugging)
./target/release/looking-glass --data /path/to/alice_202106/Alice1/bin/base --map skool1

# Run validation suite
./target/release/looking-glass --validate-all
```

### Testing
```bash
cargo test --locked                    # All 689+ tests
cargo test --locked --bin lg-launch    # Launcher unit tests
cargo test --locked --test integration # Integration tests (launcher + engine)
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
- Engine: modules declared in `main.rs` (100+ `mod` statements)
- Launcher crates: proper workspace crates with `lib.rs`
- `glam` for Vec2/Vec3/Mat4 with serde support
- `clap` for CLI, `serde` + `toml` for config, `tracing` for logging

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

### Launcher/Mod Testing Pattern
```rust
// Unit tests for core logic
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_layering() { ... }
    
    #[test]
    fn test_mod_discovery() { ... }
    
    #[test]
    fn test_launch_command_build() { ... }
}

// Integration tests in tests/
#[test]
fn test_full_launch_flow() { ... }

#[test]
fn test_mod_load_in_engine() { ... }
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
| `jsonschema` | =0.60 | Manifest/config validation |
| `libloading` | latest | Native mod cdylib loading (new) |

### Launcher (Workspace Dependencies)
| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `directories` | XDG path resolution |
| `toml` | Config parsing |
| `walkdir` | Filesystem scanning |
| `notify` | Filesystem watching |
| `tracing` | Logging |

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
- **Launcher/mod work**: Feature branches per mod or crate

## Pushing Changes to GitHub
- Use `gh` CLI for all GitHub operations (push, PR creation, issue management)
- Push to your fork: `git push myfork <branch>` (not `origin`)
- Create PRs via: `gh pr create --base main --head <branch> --repo looking-glass-project/LookingGlass`
- Check CI status: `gh run list --repo looking-glass-project/LookingGlass`
- **Never push directly to `origin` (upstream)** — you only have write access to your fork (`myfork`)

---

## Resources
- **Website**: https://looking-glass-project.github.io/LookingGlass/
- **Original Repo**: https://github.com/looking-glass-project/LookingGlass
- **Fork**: https://github.com/SaraEatsPickledPeppers/LookingGlass
- **Game data**: https://archive.org/details/alice_202106
- **Macroquad**: https://github.com/not-fl3/macroquad
- **Rodio**: https://github.com/RustAudio/rodio
- **clap**: https://github.com/clap-rs/clap

---

## Agent Notes

- **Read `WHERE_WE_ARE.md` first** for full roadmap and current status
- **Read `PHASE2_PLAN.md`** for detailed task breakdown
- **Preserve engine**: The `looking-glass` game binary must keep working; launcher wraps it
- **Library-first**: Build `looking-glass-config` and `looking-glass-mod` as reusable crates
- **Small agents**: One agent per crate or well-defined feature
- **Test on Linux** — primary target; Windows/macOS via CI
- **No game data in repo** — never commit PK3/BSP/FTX files
- **Check `tools/check_source.py`** before committing media files
- **XDG compliance**: Use `directories` crate for config/data/cache paths
- **Error handling**: `anyhow::Result` with context; user-friendly messages in CLI
- **Logging**: `tracing` with structured JSON to `~/.local/share/looking-glass/logs/`

---

## Agent Deployment Strategy

**Deploy small, focused agents:**

1. **Crate agents** (one per crate): `config`, `mod`, `cli`, `engine` (for mod hooks)
2. **Mod agents** (one per proof-of-concept mod): Options Menu Extender, Debug Menu, HUD Customizer
3. **Integration agents**: Config layering, mod discovery→load pipeline, CLI↔engine env var passing
4. **Test agents**: Unit tests per module, integration tests for launch + mod load flows

**Avoid:** Large agents that try to build entire subsystems — they stall.

**Results target:** Each agent completes in <5 minutes. Phase 2A = ~3 agents, Phase 2B = ~3 agents (one per mod).