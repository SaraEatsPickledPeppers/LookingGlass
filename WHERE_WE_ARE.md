# Where We Are — Looking Glass Linux Port

**Last updated:** 2026-10-28  
**Status:** Phase 2 complete, Phase 3 **COMPLETE**, Phase 4 **COMPLETE**, Phase 5 **MOSTLY COMPLETE**, Phase 6 **COMPLETE**

---

## ✅ Phase 1: Core Build Support — COMPLETE

| Task | Status |
|------|--------|
| `build.rs` Linux stack size (8MB) | ✅ Done |
| GitHub Actions CI (ubuntu-latest + windows-latest) | ✅ Done |
| `cargo build --release --locked` on Linux | ✅ Verified working |
| Binary runs (`--list`, `--validate-all`) | ✅ Verified (needs game data) |

**Files changed:** `build.rs`, `.github/workflows/ci.yml`

---

## ✅ Phase 2: Platform Abstraction — COMPLETE

| Part | Task | Status | Key Changes |
|------|------|--------|-------------|
| **Part 1** | Window Focus (`src/look.rs`) | ✅ | 3-tier: x11rb (X11) → xdotool (Wayland) → `true` fallback |
| **Part 2** | Audio Regression Fix | ✅ | `"Windows output device"` → `"audio output device"` |
| **Part 3** | Platform Conditional Audit | ✅ | Found 5 sites; 2 need work (focus, gamepad); gamepad → Tier 5 |
| **Part 4** | CI Hardening | ✅ | xdotool, xvfb, xvfb-run wrapping |

**Files changed:**
- `Cargo.toml` — Added `x11rb = "=0.14.0"` (optional, feature `"x11"`)
- `src/look.rs` — Three `#[cfg]` implementations of `window_focused()`
- `src/audio/regression.rs` — Platform-agnostic error message
- `.github/workflows/ci.yml` — Added xdotool, xvfb, xvfb-run
- `PHASE2_PLAN.md`, `LINUX_PORT_PLAN.md` — Gamepad moved to Tier 5/Phase 7

**Build verified:**
```bash
cargo build              # default (xdotool fallback) ✓
cargo build --features x11   # x11rb implementation ✓
cargo test --locked      # passes ✓
```

---

## ✅ Phase 3: Setup/Launch Scripts — COMPLETE

### What was completed:

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| **tools/linux_setup.sh** | Core setup logic: find game data, validate PK3s, extract 7z, write config | High | ✅ **COMPLETE** |
| **Setup.sh** | Entry point: arg parsing, GUI dialog detection (zenity/kdialog/yad), calls linux_setup.sh | High | ✅ **COMPLETE** |
| **Launch.sh** | Entry point: binary discovery order, auto cargo build, passes args through | High | ✅ **COMPLETE** |
| **Rust logging** | Add `--verbose/--debug` flags, structured logging to `~/.local/share/LookingGlass/logs/` | Medium | ✅ **COMPLETE** |

### What was completed:
- **tools/linux_setup.sh**: All core functions implemented:
  - `find_game_data` - Searches 5 locations for game data
  - `validate_base` - Validates 5 required PK3s, 36 maps, village map FAKK v42
  - `extract_base_paks` - Extracts from .7z/.zip with prefix detection
  - `download_game_data` - Downloads from Internet Archive with SHA256 verification
  - `write_config` / `write_receipt` - Atomic writes to private/data-path.txt and private/setup.json
  - Language handling: `test_language`, `import_languages`, `apply_language`, `switch_installed_language`
  - `main_setup` - Full orchestration matching Windows Setup.cmd logic
  - `find_extractor`, `get_archive_entries`, `compute_sha256`, `check_cancel`
- **Setup.sh**: Updated to delegate to linux_setup.sh with all arguments (--language, --data, --download, --gui, --dry-run, --verbose)
- **Launch.sh**: Binary discovery working (pre-built, cargo release, private playtest dirs)
- **Rust logging** (`src/logging.rs`): 
  - `--verbose` / `-v` flag for INFO level logging
  - `--debug` flag for DEBUG level logging
  - Structured JSON logging to `~/.local/share/LookingGlass/logs/looking-glass-YYYYMMDD-HHMMSS.log`
  - Non-blocking file appender with rotation via `tracing-appender`
  - Dependencies added: `chrono`, `tracing`, `tracing-subscriber`, `tracing-appender`

### What was attempted (agents stalled):
- Multiple agents spawned for the above tasks
- All stalled after ~10 minutes (watchdog timeout)
- **Fixed**: Re-deployed as many small agents (one per function) - all completed successfully

---

## 📋 Phase 4+: Future Phases (from LINUX_PORT_PLAN.md)

| Phase | Focus | Status |
|-------|-------|--------|
| **Phase 4** | Packaging (AppImage/tarball, `tools/package_linux.py`) | ✅ **COMPLETE** |
| **Phase 5** | Testing & Validation (full test suite, campaign checks, manual playthrough) | ✅ **MOSTLY COMPLETE** (689 tests pass, validation passes, campaign has known slime timeout) |
| **Phase 6** | Documentation & Polish (INSTALL.md Linux section, README badge) | ✅ **COMPLETE** |
| **Phase 7** | Feature Parity & Polish (gamepad via gilrs, Wayland-native focus, audio devices) | 🔄 **IN PROGRESS** (6 agents deployed) |

---

## 🎯 Immediate Next Steps

1. **Install required system packages (user needs to run):**
   ```bash
   sudo dnf install -y zenity xdotool  # for GUI dialogs and window focus fallback
   ```

2. **Test current build with game data** once available
   - Need `Alice1_2011_vanilla.7z` from Internet Archive
   - Run `Setup.sh --download` or `Setup.sh --data /path/to/Alice1`
   - Run `Launch.sh` to start game
   - Test `--verbose` / `--debug` flags for logging

3. **Phase 5: Testing & Validation** - ✅ **MOSTLY COMPLETE**
   - `cargo test --locked` — 689 tests, 687 passed, 2 ignored, 0 failed
   - `cargo run -- --validate-all` — All 36 maps validated
   - `cargo run -- --campaign-route-check` — Fails at pandemonium (known slime timeout)
   - Game launches and runs (village level tested)

4. **Phase 7: Feature Parity & Polish** - Next major phase
   - Gamepad support via gilrs/SDL3
   - Wayland-native window focus improvements
   - Advanced audio configuration

---

## 📁 Key Files to Reference

| File | Purpose |
|------|---------|
| `LINUX_PORT_PLAN.md` | Full 7-phase plan |
| `PHASE2_PLAN.md` | Phase 2 detailed breakdown |
| `agents.md` | Agent instructions for this project |
| `WHERE_WE_ARE.md` | This file |
| `build.rs` | Stack size for Windows + Linux |
| `src/look.rs` | Window focus detection (3-tier Linux impl) |
| `.github/workflows/ci.yml` | CI with Linux + Windows matrix |

---

## 🤖 Agent Deployment Strategy (Revised - EXECUTED SUCCESSFULLY)

**Previous issue:** Large tasks caused agent stalls.

**New approach:** Deploy many small agents:
1. One agent per shell function in `tools/linux_setup.sh` ✅ **COMPLETED** (10 agents)
2. One agent for `Setup.sh` wrapper ✅ **ALREADY EXISTS** (no agent needed)
3. One agent for `Launch.sh` wrapper ✅ **ALREADY EXISTS** (no agent needed)
4. One agent for Rust logging module ✅ **COMPLETED**
5. One agent to wire logging into `main.rs` argument parsing ✅ **COMPLETED** (integrated in logging.rs agent)

**Results:** All 11 agents completed successfully in <5 minutes each. Phase 3 **FULLY COMPLETE**.

---

## 💡 Notes for Next Session

- Game data installed — extracted from `Alice1_2011_vanilla.7z` (already in project root)
- `xdotool` installed, `zenity` already present (Fedora default)
- All Rust builds pass; binary runs with game data configured
- Windows support fully preserved via `#[cfg]` — no breaking changes
- **Phase 3 COMPLETE**: Setup.sh → linux_setup.sh integration tested and working (dry-run verified)
- **Rust logging complete**: --verbose/-v and --debug flags added, structured JSON logging to ~/.local/share/LookingGlass/logs/
- **Phase 4 COMPLETE**: 
  - `tools/package_linux.py` created for tarball and AppImage packaging
  - Tarball format working (tested and verified)
  - AppImage format supported (requires FUSE at build time)
  - `.github/workflows/release.yml` created for automated Linux releases on tag push
- **Phase 5 MOSTLY COMPLETE**:
  - `cargo test --locked` — 689 tests, 687 passed, 2 ignored, 0 failed
  - `--validate-all` — All 36 maps validated
  - `--campaign-route-check` — Fails at pandemonium (known slime timeout, not Linux-specific)
  - Game launches and runs (village level tested)
- **Phase 6 COMPLETE**:
  - `docs/INSTALL.md` — Comprehensive Linux instructions (5 distro families, tarball/AppImage, troubleshooting)
  - `README.md` — Linux support section with system requirements, quick start, tested distros
  - `docs/KNOWN_ISSUES.md` — Linux-specific known issues section
- Ready for Phase 7: Feature Parity & Polish (gamepad, Wayland-native focus, audio)