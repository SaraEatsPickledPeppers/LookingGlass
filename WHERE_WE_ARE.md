# Where We Are — Looking Glass Console Launcher for Alice

**Last updated:** 2026-10-29  
**Status:** Engine complete. Building a focused, Alice-only console launcher with mod support.

---

## ✅ Completed: Rust Engine Foundation

| Component | Status |
|-----------|--------|
| Rust reimplementation of Alice (2011) | ✅ Complete — 39 visits playable |
| Macroquad-based rendering & game framework | ✅ Complete |
| BSP parsing, asset loading, campaign system | ✅ Complete |
| Linux port (build, CI, packaging, testing) | ✅ Complete |
| Gamepad support (gilrs, PS4/Xbox) | ✅ Complete |
| Wayland + X11 window focus | ✅ Complete |
| Audio device selection (rodio/cpal) | ✅ Complete |
| Performance profiling (`--profile`, F11 overlay) | ✅ Complete |
| Structured JSON logging (`--verbose/--debug`) | ✅ Complete |
| Automated validation suite (689 tests) | ✅ Complete |
| Mod system infrastructure (`looking-glass-mod` crate) | ✅ Core complete |
| CLI launcher (`lg-launch`) with mod profiles | ✅ Working |

---

## 🎯 Current Direction: Alice-Only Console Launcher

**Goal:** A single-purpose, functional console launcher (`lg-launch`) that:
1. Launches Alice with the correct data path
2. Loads proof-of-concept mods via the existing mod system
3. Provides a clean foundation for future mod development
4. Has zero scope beyond Alice — no game registry, no multi-platform abstraction

### Why This Scope?
- The engine only runs Alice. A general launcher is dead code.
- Proof-of-concept mods need a stable, simple integration point first.
- Strong foundation > broad features. We add breadth after depth works.

---

## 📋 Roadmap (Small, Concrete)

### Phase 2A: Launcher Polish (Current)

| Task | Status | Notes |
|------|--------|-------|
| `lg-launch` CLI: data path config, mod profile selection | ✅ Working | `data_directory` added in latest commit |
| Mod discovery: fix base-dir logic | ✅ Fixed | Parent of `mods/` now passed correctly |
| Environment variable mod injection (`LOOKING_GLASS_MODS`) | ✅ Added | For launcher→engine mod passing |
| Mod load order via `LOOKING_GLASS_MOD_ORDER` | ✅ Added | |
| Remove `looking-glass-core`, `looking-glass-launcher`, `looking-glass-tui` from workspace | ✅ Done | Cargo.toml cleaned |
| Simplify `lg-launch` to Alice-only (remove `game_id` arg) | ✅ Done | Hardcoded "Alice" everywhere |
| Config file support (layered: defaults → XDG → local → CLI) | ✅ Done | `ConfigManager` with layered config |
| `--dry-run` flag to print resolved launch command | ✅ Done | Added in CLI |
| Mod validation on install (schema, dependencies) | 🔄 Next | Planned |
| Integration test: `cargo test --locked` covers launcher + mod load | ✅ Done | All 689 tests passing |
| Fix mod enable/disable test (clear shared registries on reload) | ✅ Done | Fixed in commit 2029040 |

### Phase 2B: First Proof-of-Concept Mods

| Mod Idea | Purpose | Tests Launcher Feature |
|----------|---------|------------------------|
| **Options Menu Extender** | Add custom settings tabs (FOV, render scale, debug toggles) | Mod UI injection, config persistence |
| **Level Select / Debug Menu** | Warp to any map, spawn items, noclip, god mode | Mod command registration, runtime console |
| **HUD Customizer** | Toggle/move HUD elements, show FPS, show coords | Mod render hooks, settings UI |
| **Campaign Route Visualizer** | Show visit graph, mark completed, allow jump | Mod save integration, campaign data access |
| **Input Remapper UI** | In-game rebind menu with glyph preview | Mod input hooks, button-glyphs integration |

### Phase 2C: Launcher Hardening

| Task | Status |
|------|--------|
| Config file (`~/.config/looking-glass/config.toml`) for data path, mod dir, default profile | ✅ Done |
| `--dry-run` flag to print resolved launch command | ✅ Done |
| Mod validation on install (schema, dependencies) | 🔄 Planned |
| Integration test: `cargo test --locked` covers launcher + mod load | ✅ Done |

---

## 🏗️ Architecture (Simplified)

```
LookingGlass/
├── crates/
│   ├── looking-glass-engine/   # Game binary (unchanged)
│   ├── looking-glass-mod/      # Mod system (manifest, registry, discovery)
│   ├── looking-glass-config/   # Config layering (XDG + project-local)
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
└── tests/                      # Integration tests (engine + launcher)
```

**Key principles:**
- **Single binary entry point**: `lg-launch` (or `cargo run --bin lg-launch`)
- **Config-driven**: TOML at XDG + project-local, layered
- **Mod system lives in engine**: Launcher only resolves paths + env vars
- **No game registry**: Alice path from config or `private/data-path.txt`
- **No platform abstraction**: Engine runs natively; launcher just execs it

---

## 🔧 Current Repo State

- **Fork**: `https://github.com/SaraEatsPickledPeppers/LookingGlass`
- **Clean**: No game files, no build artifacts committed
- **CI**: GitHub Actions for Linux + Windows builds
- **Packaging**: `tools/package_linux.py` for tarball/AppImage
- **Latest commit**: Fixed mod enable/disable test by clearing shared registries on reload; added ConfigManager per-mod config methods; --dry-run flag; Alice-only scope

---

## 📁 Key Files for Launcher Work

| File | Purpose |
|------|---------|
| `crates/looking-glass-cli/src/main.rs` | `lg-launch` entry point |
| `crates/looking-glass-cli/src/launcher.rs` | Launch options, mod discovery, command building |
| `crates/looking-glass-cli/src/profiles.rs` | Mod profiles, launcher config |
| `crates/looking-glass-mod/src/` | Mod manifest, registry, discovery, installation |
| `crates/looking-glass-engine/src/mod_integration.rs` | Engine-side mod loading, env var injection |
| `private/data-path.txt` | Resolved Alice data path (written by Setup.sh) |

---

## 💡 Notes for Next Session

- **Start with Phase 2A cleanup**: Hardcode Alice in `lg-launch`, remove `game_id`
- **Engine is the test bed**: Every mod loads into the running Alice engine
- **Testing harness**: `cargo test --locked` runs 689 tests; add launcher/mod tests there
- **First mod target**: Options Menu Extender — exercises config, UI hooks, persistence

## 📝 GitHub Push Policy
**Every major change gets pushed to the fork (`myfork`).**

```bash
# After completing a task:
git add -A
git commit -m "type: description"
git push myfork <branch>
# Then optionally: gh pr create --base main --head <branch> --repo looking-glass-project/LookingGlass
```

- Push to `myfork` (your fork), NOT `origin` (upstream)
- Use `gh` CLI for PRs and CI monitoring
- One commit per logical change; conventional commit format