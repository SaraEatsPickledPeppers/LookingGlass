# Looking Glass Linux Port Plan

## Project Overview
Looking Glass is a Rust reimplementation of American McGee's Alice (2011) using the Macroquad game framework. Currently Windows-only with Windows-specific setup scripts, build configuration, and some platform-specific code.

## Current State Analysis

### Cross-Platform Ready Components
- **Core engine**: Pure Rust, uses `macroquad` (cross-platform), `rodio` (cross-platform audio), `zip`, `image`, `glam`, `serde`
- **Game logic**: All gameplay systems (combat, traversal, NPCs, campaign, saves, etc.) are platform-agnostic
- **Asset loading**: Reads PK3 archives, BSP maps, FTX textures - pure Rust implementations
- **Dependencies**: All Cargo dependencies support Linux

### Windows-Specific Components Requiring Changes

| Component | Type | Effort | Notes |
|-----------|------|--------|-------|
| `Setup.cmd` / `Launch.cmd` | Batch + PowerShell scripts | High | Need Linux shell script equivalents |
| `tools/windows_setup.ps1` | PowerShell (30KB) | High | Full guided setup UI - needs rewrite |
| `tools/package_windows.py` | Python packaging script | Medium | Linux packaging equivalent needed |
| `build.rs` | Stack size linker args | Low | Add Linux stack size handling |
| `src/look.rs` | Window focus detection | Low | Add Linux X11/Wayland implementation |
| `src/audio/regression.rs` | Audio device check | Low | Fix "Windows output device" message |
| `tools/test_*.ps1` | PowerShell test scripts | Medium | Convert to shell/Python |
| CI/CD | GitHub Actions | Low | Add Linux build/test matrix |

## Implementation Plan

### Phase 1: Core Build Support (Week 1)
- [ ] Update `build.rs` for Linux stack size (8MB like Windows)
- [ ] Add Linux target to CI/CD (GitHub Actions)
- [ ] Verify `cargo build --release --locked` works on Linux
- [ ] Test basic window creation with Macroquad on Linux

### Phase 2: Platform Abstraction (Week 1-2)
- [ ] Implement `window_focused()` for Linux in `src/look.rs`
  - X11: Use Xlib `XGetInputFocus` or `xdotool`
  - Wayland: May need protocol-specific approach or fallback to `true`
- [ ] Fix audio regression test message in `src/audio/regression.rs`
- [ ] Audit all `#[cfg(target_os = "windows")]` usages

### Phase 3: Setup/Launch Scripts (Week 2-3)
- [ ] Create `Setup.sh` - Linux equivalent of `Setup.cmd`
  - Bash script with similar guided flow
  - Use `zenity`/`kdialog`/`yad` for GUI dialogs, or TUI with `dialog`/`whiptail`
  - Fallback to CLI prompts if no GUI tools
- [ ] Create `Launch.sh` - Linux equivalent of `Launch.cmd`
  - Handle build-from-source vs binary execution
  - Pass arguments through to binary
- [ ] Create `tools/linux_setup.sh` - Core setup logic (shared by Setup.sh)
- [ ] Test with actual game data on Linux

### Phase 4: Packaging & Distribution (Week 3)
- [ ] Create `tools/package_linux.py` - Linux packaging script
  - Create AppImage or tarball with binary + assets
  - Include Setup.sh, Launch.sh, README
  - Handle dependencies (libgl, libx11, etc.)
- [ ] Add Linux release workflow to GitHub Actions
- [ ] Test packaged release on clean Linux system

### Phase 5: Testing & Validation (Week 3-4)
- [ ] Run full test suite: `cargo test --locked`
- [ ] Run validation checks: `cargo run -- --validate-all`
- [ ] Run campaign checks: `cargo run -- --campaign-route-check`
- [ ] Manual playthrough testing on Linux
- [ ] Audio testing (rodio with ALSA/PulseAudio/PipeWire)

### Phase 6: Documentation & Polish (Week 4)
- [ ] Update `docs/INSTALL.md` with Linux instructions
- [ ] Add Linux troubleshooting section
- [ ] Update README with Linux download/badge
- [ ] Document any known Linux-specific issues

### Phase 7: Feature Parity & Polish (Post-Launch)
- [ ] Controller support testing (SDL2/Gamepad via Macroquad)
- [ ] Wayland-native window focus improvements (beyond xdotool fallback)
- [ ] Advanced audio configuration (PipeWire/PulseAudio device selection)
- [ ] Performance tuning and profiling on Linux

## Technical Details

### Stack Size (build.rs)
```rust
// Linux typically defaults to 8MB, but explicit is better
#[cfg(all(target_os = "linux", target_env = "gnu"))]
println!("cargo:rustc-link-arg=-Wl,-z,stack-size=8388608");
```

### Window Focus Detection (look.rs)
```rust
#[cfg(target_os = "linux")]
pub fn window_focused() -> bool {
    // Option 1: Try X11 via xlib-sys
    // Option 2: Use `xdotool getwindowfocus` subprocess
    // Option 3: Return true (conservative, matches non-Windows fallback)
    true
}
```

### Setup.sh Architecture
```bash
#!/bin/bash
# Setup.sh - Entry point
# Delegates to tools/linux_setup.sh for core logic
# Uses zenity/kdialog/yad for GUI, fallback to CLI

# Key functions needed:
# - Find game data (same search paths, adapted for Linux)
# - Validate PK3 files and language packs
# - Extract 7z archives (requires p7zip)
# - Write private/data-path.txt
# - Write private/setup.json receipt
```

### Required System Dependencies (Linux)
- `libgl1-mesa-dev` / `libglvnd-dev` (OpenGL)
- `libx11-dev` / `libxcb1-dev` / `libwayland-dev` (Windowing)
- `libasound2-dev` / `libpulse-dev` / `pipewire-dev` (Audio)
- `p7zip-full` (7z extraction for setup)
- `zenity` or `kdialog` or `yad` (GUI dialogs, optional)
- `curl` / `wget` (Internet Archive download)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Macroquad Linux bugs | Medium | High | Test early; file upstream issues |
| Audio backend issues (rodio) | Medium | Medium | Test ALSA/PulseAudio/PipeWire |
| 7z extraction in setup | Low | High | Use `p7zip` CLI, well-tested |
| GUI dialog compatibility | Medium | Medium | Multiple backends + CLI fallback |
| Wayland cursor grab | Medium | Low | Macroquad handles; test focus |
| Stack overflow on Linux | Low | High | Set explicit stack size in build.rs |

## Success Criteria
1. ✅ `cargo build --release --locked` completes on Linux
2. ✅ `cargo test --locked` passes on Linux
3. ✅ `Setup.sh` can find/extract/validate game data
4. ✅ `Launch.sh` starts the game and reaches main menu
5. ✅ Full campaign playable (automated checks pass)
6. ✅ Linux release package creates working AppImage/tarball
7. ✅ CI/CD builds and tests on Linux for every PR

## References
- [Macroquad Linux support](https://github.com/not-fl3/macroquad)
- [Rodio audio backends](https://github.com/RustAudio/rodio)
- [AppImage packaging](https://appimage.org/)
- Original Windows setup: `tools/windows_setup.ps1`, `Setup.cmd`, `Launch.cmd`