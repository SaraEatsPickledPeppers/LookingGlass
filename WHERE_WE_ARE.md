# Where We Are — Looking Glass Console-First Launcher

**Last updated:** 2026-10-28  
**Status:** Core engine complete, pivoting to console-first game & mod launcher

---

## ✅ Completed: Rust Engine Foundation

| Component | Status |
|-----------|--------|
| Rust reimplementation of Alice (2011) | ✅ Complete |
| Macroquad-based rendering & game framework | ✅ Complete |
| BSP parsing, asset loading, campaign system | ✅ Complete |
| Linux port (build, CI, packaging, testing) | ✅ Complete |
| Gamepad support (gilrs, PS4/Xbox) | ✅ Complete |
| Wayland + X11 window focus | ✅ Complete |
| Audio device selection (rodio/cpal) | ✅ Complete |
| Performance profiling (--profile, F11 overlay) | ✅ Complete |
| Structured JSON logging (--verbose/--debug) | ✅ Complete |
| Automated validation suite (689 tests) | ✅ Complete |

---

## 🎯 New Direction: Console-First Game & Mod Launcher

**Goal:** Build a functional-first game & mod launcher as a **console application** before making a GUI launcher.

### Why Console-First?
- Faster iteration: TUI/CLI development is quicker than GUI
- Scriptable/automatable: Works in SSH, CI, headless environments
- Unix philosophy: Composable tools that do one thing well
- Can wrap existing game binaries, Wine/Proton, Flatpak, AppImage, native Linux ports
- Foundation for future GUI (Tauri, egui, or native) without rewriting core logic

---

## 📋 Roadmap

### Phase 1: Core Launcher Infrastructure (Console/TUI)

| Task | Status | Priority |
|------|--------|----------|
| **Game registry**: Scan for installed games (Steam, Heroic, Lutris, Flatpak, AppImage, native) | 🔄 Planned | High |
| **Launch command abstraction**: Unified interface for `steam://`, `flatpak run`, `wine`, `proton`, native binaries | 🔄 Planned | High |
| **Config system**: TOML/JSON config for game entries, launch args, env vars, prefixes | 🔄 Planned | High |
| **TUI interface**: ratatui/crossterm-based interactive game list, search, filters | 🔄 Planned | High |
| **Mod profile system**: Per-game mod lists with enable/disable, load order, profiles | 🔄 Planned | High |
| **Save management**: Backup/restore/sync saves per game/profile | 🔄 Planned | Medium |

### Phase 2: Mod Management

| Task | Status | Priority |
|------|--------|----------|
| **Mod source abstraction**: NexusMods API, Mod.io, local directories, Git repos | 🔄 Planned | High |
| **Mod installation**: Extract archives, handle conflicts, track installed files | 🔄 Planned | High |
| **Dependency resolution**: Mod load order, conflicts, requirements | 🔄 Planned | Medium |
| **Profile switching**: Quick swap between mod sets (vanilla, QoL, overhaul, etc.) | 🔄 Planned | Medium |

### Phase 3: Platform Integration

| Task | Status | Priority |
|------|--------|----------|
| **Steam integration**: Library parsing, Proton version selection, Steam Input configs | 🔄 Planned | Medium |
| **Heroic/Gog integration**: Wine prefix management, GOG Galaxy compatibility | 🔄 Planned | Medium |
| **Flatpak/Steam Deck support**: Deck UI mode, controller navigation | 🔄 Planned | Medium |
| **Desktop entries**: Generate `.desktop` files for launched games with custom icons | 🔄 Planned | Low |

### Phase 4: Advanced Features

| Task | Status | Priority |
|------|--------|----------|
| **Benchmarking**: Built-in FPS/frame time capture, comparison reports | 🔄 Planned | Low |
| **Sync/cloud**: Optional save/mod profile sync via Git/rclone/Syncthing | 🔄 Planned | Low |
| **Plugin system**: Lua/WASM plugins for custom launch logic, UI extensions | 🔄 Planned | Low |
| **GUI prototype**: Tauri/egui frontend sharing core library | 🔄 Planned | Low |

---

## 🏗️ Architecture Vision

```
looking-glass-launcher/
├── src/
│   ├── core/           # Game registry, launch abstraction, config
│   ├── mod/            # Mod management, sources, installation
│   ├── platform/       # Steam, Heroic, Flatpak, Wine/Proton integrations
│   ├── tui/            # ratatui-based console interface
│   ├── save/           # Save backup/restore/sync
│   └── cli/            # CLI argument parsing, subcommands
├── Cargo.toml          # Workspace with core lib + cli binary
└── tests/              # Integration tests
```

**Key principles:**
- **Library-first**: Core logic in `looking-glass-launcher-core` crate, thin CLI/TUI wrappers
- **No game data**: Never bundle game files; only manage launch configuration
- **Cross-platform**: Linux primary, Windows/macOS via conditional compilation
- **Unix-native**: Leverages system tools (flatpak, steam, wine) rather than reimplementing

---

## 🔧 Current Repo State

- **Fork**: `https://github.com/SaraEatsPickledPeppers/LookingGlass`
- **Clean**: No game files, no build artifacts, no test outputs
- **CI**: GitHub Actions for Linux + Windows builds
- **Packaging**: `tools/package_linux.py` for tarball/AppImage releases
- **Docs**: `docs/INSTALL.md`, `README.md`, `docs/KNOWN_ISSUES.md` updated for Linux

---

## 📁 Key Files (Updated for New Direction)

| File | Purpose |
|------|---------|
| `WHERE_WE_ARE.md` | This file — project status & roadmap |
| `agents.md` | Agent instructions for launcher development |
| `Cargo.toml` | Will become workspace root for launcher crates |
| `src/main.rs` | Current game binary — will become `looking-glass` game binary |
| `tools/package_linux.py` | Packaging for launcher releases |
| `.github/workflows/` | CI for launcher + game engine |

---

## 🤖 Agent Deployment Strategy (Updated)

**New approach for launcher development:**
1. **One agent per core module** (registry, launch, mod, platform, tui, save, cli)
2. **Integration agents** for cross-cutting concerns (config, logging, error handling)
3. **Test agents** per module with property-based testing where applicable
4. **Small scope**: Each agent owns one crate or one well-defined feature

**Avoid:** Large agents that try to build entire subsystems — they stall.

---

## 💡 Notes for Next Session

- **Start with Phase 1**: Game registry + launch abstraction + config system
- **Use existing engine** as a test game: the Alice reimplementation validates the launcher
- **TUI library**: `ratatui` + `crossterm` (mature, fast, no GPU needed)
- **Config format**: TOML (human-editable, typed via serde)
- **First milestone**: `lg-launch --list` shows installed games, `lg-launch alice` runs the engine
- **Preserve game engine**: Keep `looking-glass` binary working; launcher wraps it