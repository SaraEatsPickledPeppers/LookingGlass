# Looking Glass — Agent Instructions

## Project Context
**Looking Glass** is a Rust reimplementation of American McGee's Alice (2011) using the Macroquad game framework. Currently Windows-only; this project ports it to Linux.

**Key files to understand:**
- `README.md` — Project overview, status, install instructions
- `Cargo.toml` — Dependencies, Rust version (1.86), profile config
- `src/main.rs` — Entry point, CLI options, mode dispatch (1000+ lines)
- `build.rs` — Windows stack size linker args (needs Linux equivalent)
- `LINUX_PORT_PLAN.md` — Detailed port plan (read this first)

## Architecture Overview

### Core Modules (src/)
| Area | Modules | Purpose |
|------|---------|---------|
| **Engine** | `main.rs`, `viewer.rs`, `render.rs`, `assets.rs`, `bsp.rs` | Window, rendering, asset loading, BSP parsing |
| **Gameplay** | `campaign*.rs`, `character.rs`, `movement.rs`, `combat.rs`, `weapons.rs` | Campaign flow, Alice, locomotion, combat, weapons |
| **World** | `level.rs`, `levels/`, `collision.rs`, `interaction.rs`, `traversal.rs` | Maps, physics, doors, ropes, swimming, ledges |
| **NPCs** | `npc.rs`, `ant.rs`, `duchess.rs`, `chess.rs`, `fire_imp.rs`, etc. | Enemy/entity implementations |
| **Systems** | `audio.rs`, `input.rs`, `menu.rs`, `save.rs`, `inventory.rs` | Audio, input, UI, saves, inventory |
| **Validation** | `*_check.rs`, `*_render.rs`, `campaign_chain.rs` | Automated tests, visual regression, campaign verification |

### Key Patterns
- **Mode-based CLI**: `main.rs` dispatches 100+ `--mode` flags to check functions
- **Headless vs Windowed**: Checks run either headless (`Run::Headless`) or with window (`Run::Windowed`)
- **Asset system**: `Assets::open(path)` loads PK3/BSP/FTX from game data folder
- **Campaign chain**: `campaign_chain.rs` runs full 39-visit campaign with save/load boundaries

## Linux Port Tasks

### Priority 1: Build & Run
- [ ] **build.rs**: Add Linux stack size linker arg (8MB like Windows)
- [ ] **CI/CD**: Add Linux matrix to GitHub Actions (ubuntu-latest)
- [ ] **Verify**: `cargo build --release --locked` and `cargo test --locked` pass

### Priority 2: Platform Abstraction
- [ ] **src/look.rs**: Implement `window_focused()` for Linux (X11/Wayland)
- [ ] **src/audio/regression.rs**: Fix "Windows output device" error message
- [ ] **Audit**: Search for `target_os = "windows"` and add Linux equivalents

### Priority 3: Setup/Launch Scripts
- [ ] **Setup.sh**: Linux equivalent of `Setup.cmd` + `tools/windows_setup.ps1`
  - GUI dialogs via `zenity`/`kdialog`/`yad` with CLI fallback
  - Game data discovery (same search paths, Linux-adapted)
  - 7z extraction via `p7zip` CLI
  - Write `private/data-path.txt` and `private/setup.json`
- [ ] **Launch.sh**: Linux equivalent of `Launch.cmd`
  - Build from source or run binary
  - Pass args through to `looking-glass` binary
- [ ] **tools/linux_setup.sh**: Shared core logic for Setup.sh

### Priority 4: Packaging
- [ ] **tools/package_linux.py**: Create AppImage or distributable tarball
- [ ] **GitHub Actions**: Linux release workflow (build + package + upload)

### Priority 5: Testing
- [ ] Run all validation modes: `--validate-all`, `--campaign-route-check`, etc.
- [ ] Manual playthrough: main menu → New Game → first levels
- [ ] Audio: test rodio with ALSA/PulseAudio/PipeWire
- [ ] Controller: test Xbox/Gamepad via Macroquad/SDL2

## Development Workflow

### Building
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

### Testing Commands (from CONTRIBUTING.md)
```bash
cargo test --locked
cargo build --release --locked
python tools/check_source.py --worktree
python -m unittest discover -s tools -p "test_source_package.py"
```

### Key Validation Modes
| Mode | Purpose |
|------|---------|
| `--validate-all` | All maps, colliders, FTX images, materials |
| `--campaign-route-check` | Full 39-visit campaign route (headless) |
| `--campaign-legs-check` | Opening 8 route legs only |
| `--campaign-playtest` | Full playable campaign with window |
| `--physics-check` | Deterministic walk/jump on skool1 |
| `--character-check` | Alice mesh + all animation clips |
| `--registry-check` | Fingerprint all 39 campaign entries |

## Code Conventions

### Rust Style
- Edition 2021, Rust 1.86 minimum
- `anyhow::Result` for errors, `bail!` for early returns
- Modules declared in `main.rs` (100+ `mod` statements)
- Heavy use of `macroquad::prelude::*` for math/graphics
- `glam` for Vec2/Vec3/Mat4 with serde support

### Testing Pattern
```rust
// Headless check returning Result
pub fn check(assets: &mut Assets) -> Result<()> { ... }

// Windowed check using Macroquad
pub async fn render_check(assets: &mut Assets) -> Result<()> { ... }

// In main.rs dispatch:
if mode == "my-check" {
    return my_module::check(&mut assets);
}
if mode == "my-render" {
    macroquad::Window::from_config(viewer::config(), async move {
        if let Err(e) = my_module::render_check(&mut assets).await {
            eprintln!("My render failed: {e:#}");
            std::process::exit(1);
        }
    });
    return Ok(());
}
```

### Asset Loading
```rust
let mut assets = Assets::open(&data_path)?;
// Read raw bytes from PK3
let bsp_bytes = assets.read("maps/skool1.bsp")?;
// Parse BSP
let map = Bsp::parse(&bsp_bytes)?;
// Load texture
let ftx_bytes = assets.read("textures/wall.ftx")?;
let image = texture::decode_ftx(&ftx_bytes)?;
```

## Platform-Specific Code Locations

### Windows-Only Code to Port
| File | Function | Windows Implementation | Linux Needed |
|------|----------|------------------------|--------------|
| `build.rs` | Stack size | `/STACK:8388608` (MSVC) / `-Wl,--stack,8388608` (GNU) | `-Wl,-z,stack-size=8388608` |
| `src/look.rs` | `window_focused()` | `GetActiveWindow`/`GetForegroundWindow` via user32.dll | X11 `XGetInputFocus` or fallback `true` |
| `src/audio/regression.rs` | Device check | `"No usable Windows output device"` | Generic message |
| `Setup.cmd` | Launcher | Batch → PowerShell | `Setup.sh` → `tools/linux_setup.sh` |
| `Launch.cmd` | Launcher | Batch + cargo build fallback | `Launch.sh` + cargo build fallback |
| `tools/windows_setup.ps1` | Guided setup UI | PowerShell + .NET ZipFile | Bash + `p7zip` + `zenity`/`dialog` |
| `tools/package_windows.py` | Windows ZIP | PyInstaller-style bundling | AppImage or tarball |

### Conditional Compilation Pattern
```rust
#[cfg(target_os = "windows")]
pub fn platform_specific() { ... }

#[cfg(target_os = "linux")]
pub fn platform_specific() { ... }

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn platform_specific() { ... } // macOS, etc.
```

## Dependencies (Cargo.toml)
| Crate | Purpose | Linux Support |
|-------|---------|---------------|
| `macroquad = "=0.4.14"` | Game framework (miniquad) | ✅ Full |
| `rodio = "=0.20.1"` | Audio (wav, mp3 via symphonia) | ✅ ALSA/Pulse/PipeWire |
| `zip = "=0.6.6"` | PK3 reading | ✅ |
| `image = "=0.24.9"` | TGA/JPEG/PNG loading | ✅ |
| `glam = "=0.27.0"` | Math (Vec/Mat/Quat) | ✅ |
| `serde` | Serialization | ✅ |
| `anyhow` | Errors | ✅ |

**Pinned versions** — Do not upgrade without testing; Macroquad 0.4.14 requires specific fontdue version.

## Game Data Requirements
- **Source**: `Alice1_2011_vanilla.7z` from Internet Archive (alice_202106)
- **Required PK3s**: `pak0.pk3`, `pak1_large.pk3`, `pak2.pk3`, `pak3.pk3`, `pak5_mod.pk3`
- **Language packs**: `pak4_english.pk3` + `base_g.pk3` (de), `base_f.pk3` (fr), `base_s.pk3` (es), `base_d.pk3` (nl)
- **Setup copies** to `private/game-data/` — original files untouched

## Common Issues & Fixes

| Issue | Cause | Fix |
|-------|-------|-----|
| Stack overflow on level transition | Default 1MB stack too small | Set 8MB in `build.rs` for Linux |
| Window focus not detected | `window_focused()` returns `true` on Linux | Implement X11 focus query |
| Audio device not found | Rodio backend selection | Ensure `libasound2`/`pipewire` installed |
| 7z extraction fails | `p7zip` not installed | `apt install p7zip-full` |
| GUI dialogs don't appear | No `zenity`/`kdialog`/`yad` | Install or use CLI fallback |
| Macroquad window doesn't open | Missing OpenGL/Wayland libs | `apt install libgl1-mesa-dev libx11-dev libwayland-dev` |

## Git Workflow
- Main branch: `main`
- Commits: Conventional (feat/fix/docs/refactor/test/chore)
- PRs: Required for all changes
- CI: Must pass on Windows + Linux before merge

## Resources
- **Website**: https://looking-glass-project.github.io/LookingGlass/
- **Repo**: https://github.com/looking-glass-project/LookingGlass
- **Game data**: https://archive.org/details/alice_202106
- **Macroquad**: https://github.com/not-fl3/macroquad
- **Rodio**: https://github.com/RustAudio/rodio

## Agent Notes
- **Read `LINUX_PORT_PLAN.md` first** for full task breakdown
- **Test on Linux** — don't assume Windows behavior translates
- **Preserve Windows support** — use `#[cfg]` not removal
- **Keep changes focused** — one platform abstraction at a time
- **Run validation modes** after any engine change
- **Game data is external** — never commit PK3/BSP/FTX files
- **Check `tools/check_source.py`** before committing media files