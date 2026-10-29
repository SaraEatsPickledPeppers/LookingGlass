# Phase 2 Plan: Alice-Only Console Launcher + Proof-of-Concept Mods

**Duration:** 2-3 weeks  
**Goal:** A working `lg-launch` that launches Alice with mod support, plus 2-3 proof-of-concept mods that exercise the mod system foundations.

---

## Scope Boundaries (Non-Negotiable)

| In Scope | Out of Scope |
|----------|--------------|
| `lg-launch` CLI for Alice only | Game registry / multi-game support |
| Mod discovery, profiles, installation | Steam/Heroic/Flatpak/Wine integration |
| Config (data path, mod dir, default profile) | TUI / GUI interface |
| Environment variable mod injection | Save management / cloud sync |
| 2-3 proof-of-concept mods | Mod sources (NexusMods, Mod.io, Git) |
| Integration tests for launcher + mods | Plugin system (Lua/WASM) |
| Documentation for mod authors | Desktop entries, benchmarking |

---

## Part 1: Launcher Cleanup & Hardening (Week 1)

### 1.1 Simplify `lg-launch` to Alice-Only
- [ ] Remove `game_id` CLI argument — hardcode "Alice"
- [ ] Remove `game_id` from `LaunchOptions`, `LauncherConfig`, `ModProfile`
- [ ] Update help text, defaults, profile logic
- [ ] Verify `cargo build --release --locked` and `cargo test --locked` pass

### 1.2 Config File Support
- [ ] Define `LauncherConfig` TOML schema:
  ```toml
  data_directory = "/path/to/alice_202106/Alice1/bin/base"
  mod_directory = "~/.local/share/looking-glass/mods/Alice"
  default_profile = "vanilla"
  profiles = { vanilla = { mods = [] }, cheats = { mods = ["cheats"] } }
  ```
- [ ] Implement layered loading: defaults → XDG (`~/.config/looking-glass/config.toml`) → project-local (`private/config.toml`) → CLI overrides
- [ ] Add `directories` crate for XDG paths
- [ ] Write config on first run if missing (guided setup)

### 1.3 `--dry-run` Flag
- [ ] Print resolved launch command (exe, args, env, working dir) without executing
- [ ] Useful for debugging mod injection, data path resolution

### 1.4 Mod Validation on Install
- [ ] Run manifest schema validation (`jsonschema` 0.60)
- [ ] Check dependencies exist (by mod ID)
- [ ] Warn on load order conflicts
- [ ] Return structured error for CLI/TUI consumption

### 1.5 Integration Tests
- [ ] Test: `lg-launch --dry-run` prints correct command with vanilla profile
- [ ] Test: `lg-launch --dry-run --profile cheats` includes cheat mod env vars
- [ ] Test: Config layering priority (CLI > project-local > XDG > defaults)
- [ ] Test: Mod discovery finds installed mods in `mods/Alice/`
- [ ] Add to `cargo test --locked` suite

---

## Part 2: Proof-of-Concept Mods (Week 1-2)

### Mod 1: Options Menu Extender
**Purpose:** Add custom settings tabs to the in-game options menu.

**Features:**
- New "Mods" tab in options menu
- FOV slider (60-120, default 90)
- Render scale slider (0.5-2.0, default 1.0)
- Debug toggles: Show FPS, Show Coords, Show Collision Boxes
- Settings persist to mod-specific config file

**Tests launcher/mod system:**
- Mod-provided UI injection into existing menu
- Config persistence (read/write mod config)
- Settings applied at runtime (engine reads mod config)

**Files:**
```
mods/Alice/options-menu-extender/
├── mod.toml          # Manifest: id, name, version, dependencies
├── src/
│   ├── main.lua      # Or Rust if using native mods
│   ├── menu.rs       # Options menu hook
│   └── config.rs     # Mod config schema + persistence
└── assets/           # Optional: icons, fonts
```

### Mod 2: Level Select / Debug Menu
**Purpose:** Developer tools — warp to any map, spawn items, noclip, god mode.

**Features:**
- New "Debug" tab in pause menu (or `F1` overlay)
- Map list: all 39 campaign visits + test maps
- Warp button: loads map, preserves inventory/health
- Cheat toggles: Noclip, God Mode, Infinite Ammo, No Clip Camera
- Item spawner: dropdown of all items/weapons, "Give" button
- Console command registration: `warp <map>`, `give <item>`, `noclip`

**Tests launcher/mod system:**
- Mod command registration (runtime console)
- Engine state manipulation (map load, player state)
- Save/load integration (warp creates valid save state)
- Input handling (keybinds for debug toggles)

**Files:**
```
mods/Alice/debug-menu/
├── mod.toml
├── src/
│   ├── main.rs
│   ├── debug_menu.rs
│   ├── commands.rs      # Console command registration
│   └── warp.rs          # Map transition logic
└── assets/
```

### Mod 3: HUD Customizer
**Purpose:** Toggle/move HUD elements, show debug overlay.

**Features:**
- HUD element toggles: Health, Ammo, Weapon Icon, Compass, Objectives
- Position editor: drag HUD elements in "Edit Mode" (hold key + mouse)
- Debug overlay: FPS, frame time, player coords, camera angles, current map/visit
- Layout profiles: "Minimal", "Classic", "Debug", "Custom"
- Persists layout to mod config

**Tests launcher/mod system:**
- Mod render hooks (overlay rendering)
- Input handling (mouse drag in edit mode)
- Config persistence (layout profiles)
- Integration with button-glyphs mod (show controller glyphs in HUD)

**Files:**
```
mods/Alice/hud-customizer/
├── mod.toml
├── src/
│   ├── main.rs
│   ├── hud_hooks.rs
│   ├── layout.rs
│   └── edit_mode.rs
└── assets/
```

---

## Part 3: Mod System Foundations (Week 2-3)

### 3.1 Mod Manifest Schema (Finalize)
- [ ] Required fields: `id`, `name`, `version`, `description`, `author`, `game_target = "Alice"`
- [ ] Optional: `dependencies` (mod IDs), `load_before`, `load_after`, `conflicts_with`
- [ ] Optional: `entry_point` (for native mods: `cdylib` path; for Lua: `main.lua`)
- [ ] Optional: `ui_injection` points: `main_menu`, `pause_menu`, `options_menu`, `hud`, `console`
- [ ] Version: `semver` with `VersionReq` for dependencies

### 3.2 Native Mod Support (Rust `cdylib`)
- [ ] Define stable ABI for mod entry points (versioned struct with function pointers)
- [ ] Engine loads `cdylib` via `libloading` crate
- [ ] Mod gets: `Assets` reference, `Input` reference, `RenderContext`, `GameState` hooks
- [ ] Safety: Mod runs in same process — document trust model (user-installed only)

### 3.3 Lua Mod Support (Optional, via `mlua`)
- [ ] Sandboxed Lua environment per mod
- [ ] Exposed API: UI, input, render hooks, game state read-only
- [ ] Simpler for quick iteration, no compile step

### 3.4 Mod Installation UX
- [ ] `lg-launch --install-mod <path-to-archive>`
- [ ] Extracts to `mods/Alice/<mod-id>/`
- [ ] Validates manifest, writes to registry
- [ ] `--enable-mod <mod-id> [--profile <name>]`
- [ ] `--disable-mod <mod-id> [--profile <name>]`
- [ ] `--list-mods [--profile <name>]`

---

## Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| `lg-launch` launches Alice with vanilla profile | Run, reach main menu, play skool1 |
| `lg-launch --profile cheats` loads cheat mod | Cheat menu accessible in-game |
| `lg-launch --install-mod ./mod.zip` installs mod | Mod appears in `--list-mods`, loads in game |
| Options Menu Extender mod works | New tab visible, sliders persist, FOV applies |
| Debug Menu mod works | Warp to skool2, noclip toggle, give Vorpal Blade |
| HUD Customizer mod works | Toggle health bar, drag ammo counter, FPS overlay |
| `cargo test --locked` passes (689 + new tests) | CI green |
| Config layering works | XDG config overrides defaults, CLI overrides XDG |

---

## Dependencies to Add

| Crate | Purpose | Where |
|-------|---------|-------|
| `directories` | XDG config paths | `looking-glass-config`, `looking-glass-cli` |
| `libloading` | Native mod `cdylib` loading | `looking-glass-engine` (mod_integration) |
| `toml` | Config parsing | `looking-glass-config` (already in workspace) |

---

## Execution Order

1. **Part 1.1-1.2**: Simplify CLI + config file (unblocks everything)
2. **Part 1.3-1.5**: Dry-run, validation, tests (hardening)
3. **Part 2.1**: Options Menu Extender (first mod, exercises UI + config)
4. **Part 2.2**: Debug Menu (exercises engine state, commands, save)
5. **Part 2.3**: HUD Customizer (exercises render hooks, input, layout)
6. **Part 3**: Mod system foundations (schema, native/Lua support, install UX)

---

## Notes

- **Engine unchanged**: All mod hooks added to `looking-glass-engine` via `mod_integration.rs`
- **Launcher stays thin**: Only resolves paths, builds env vars, execs engine
- **Test harness**: Existing 689 tests + new integration tests in `tests/`
- **Mod trust model**: User-installed only; no auto-download, no sandbox (native mods = full trust)
- **Iterate on Mod 1 first**: It validates the full pipeline (manifest → discovery → load → UI → config → runtime)