# Phase 2: Platform Abstraction — Detailed Plan

**Duration:** Week 1-2  
**Goal:** Replace Windows-specific platform code with Linux equivalents while preserving Windows support

---

## Part 1: Window Focus Detection (`src/look.rs`)

### Task 1.1: Audit Current Implementation
- [ ] Read `src/look.rs` lines 66-84 to understand Windows implementation
- [ ] Identify exact signature: `pub fn window_focused() -> bool`
- [ ] Note: Non-Windows fallback returns `true` (line 83)

### Task 1.2: Implement X11 Support
- [ ] Add `x11` feature to Cargo.toml (optional dependency on `x11rb` or `xlib-sys`)
- [ ] Implement `#[cfg(all(target_os = "linux", not(target_env = "musl")))]` version using Xlib:
  ```rust
  // Option A: x11rb crate (pure Rust, safer)
  // Option B: xlib-sys (bindings to Xlib, closer to Windows user32 approach)
  ```
- [ ] Test on X11 session (most common on Fedora with GNOME/Xorg)

### Task 1.3: Implement Wayland Support
- [ ] Research Wayland focus detection (no direct equivalent to XGetInputFocus)
- [ ] Options:
  - Return `true` (conservative fallback, matches current non-Windows behavior)
  - Use `xdotool getwindowfocus` subprocess (works on both X11/Wayland with XWayland)
  - Use `libwayland-client` for native Wayland (complex, may not be needed)
- [ ] Recommendation: Start with `xdotool` subprocess — works on both, simple

### Task 1.4: Unified Linux Implementation
- [ ] Combine X11 + Wayland with fallback chain:
  1. Try X11 via `x11rb` if available
  2. Fallback to `xdotool` subprocess
  3. Final fallback: `true`
- [ ] Add `xdotool` to system dependencies in CI and docs

### Task 1.5: Test & Verify
- [ ] Run on X11 session → focus detection works
- [ ] Run on Wayland session → focus detection works (via xdotool)
- [ ] Verify Windows build still compiles (`cargo check --target x86_64-pc-windows-gnu` if cross-compiling)

---

## Part 2: Audio Regression Test Fix (`src/audio/regression.rs`)

### Task 2.1: Locate & Understand the Issue
- [ ] Read `src/audio/regression.rs` line 77: `ensure!(audio.playback.is_some(), "No usable Windows output device");`
- [ ] Understand `Audio::new()` return value — why `playback` is `None` on Linux

### Task 2.2: Fix Error Message
- [ ] Change message to platform-agnostic: `"No usable audio output device"`
- [ ] Or make it conditional:
  ```rust
  let msg = if cfg!(target_os = "windows") {
      "No usable Windows output device"
  } else {
      "No usable audio output device"
  };
  ```

### Task 2.3: Investigate Root Cause (if needed)
- [ ] Check if rodio actually fails to find device on Linux
- [ ] Rodio supports ALSA, PulseAudio, PipeWire — should work
- [ ] May be a headless CI issue (no audio device) — add `continue-on-error` or mock

### Task 2.4: Test
- [ ] Run `cargo test --locked audio::regression` on Linux
- [ ] Verify test passes with real audio device
- [ ] Verify CI handles headless case gracefully

---

## Part 3: Platform Conditional Audit

> **Note:** Gamepad/controller support is now classified as **Tier 5 (Polish)** — not a Phase 2 blocker. It will be addressed in post-Phase 2 polish work.

### Task 3.1: Search for All Windows-Specific Code
- [ ] Run: `grep -rn 'target_os = "windows"' src/ --include="*.rs"`
- [ ] Run: `grep -rn 'cfg.*windows' src/ --include="*.rs"`
- [ ] Run: `grep -rn 'target_os.*windows' tools/ --include="*.py" --include="*.ps1"`

### Task 3.2: Categorize Each Usage
| File | Line | Type | Linux Equivalent Needed? |
|------|------|------|--------------------------|
| `look.rs` | 68-80 | Window focus | ✅ Part 1 |
| `audio/regression.rs` | 77 | Error message | ✅ Part 2 |
| `build.rs` | 7-17 | Linker args | ✅ Phase 1 (done) |
| *others* | *...* | *...* | *...* |

### Task 3.3: Add Linux Equivalents
- [ ] For each finding, add `#[cfg(target_os = "linux")]` branch
- [ ] Common patterns:
  - File paths: use `std::path::PathBuf` (already cross-platform)
  - Process spawning: `std::process::Command` works on both
  - Env vars: `std::env::var` works on both
  - FFI: Use `#[cfg]` to select correct library

### Task 3.4: Verify No Regressions
- [ ] `cargo check --locked` on Linux
- [ ] `cargo check --locked` on Windows (if cross-compiling setup exists)
- [ ] `cargo test --locked` passes on Linux

---

## Part 4: CI/CD Hardening (Optional but Recommended)

### Task 4.1: Add Audio Test to CI
- [ ] Install PipeWire/ALSA virtual device in CI for headless audio tests
- [ ] Or mark audio regression test as `#[ignore]` in CI, run locally

### Task 4.2: Add Window Focus Test
- [ ] Add simple test that calls `window_focused()` and asserts it returns `bool`
- [ ] Run in CI with `xvfb-run` (X virtual framebuffer)

---

## Dependencies to Add

### Cargo.toml (optional features)
```toml
[dependencies]
# For X11 window focus detection (choose one):
# x11rb = { version = "0.11", optional = true, features = ["xlib_xcb"] }
# xlib-sys = { version = "0.1", optional = true }

[features]
default = []
x11 = ["dep:x11rb"]
```

### System Dependencies (already installed from Phase 1)
- `wayland-devel` — already installed
- `libx11-devel` — already installed
- `xdotool` — **need to install**: `sudo dnf install -y xdotool`
- `x11rb` crate needs `libxcb-devel` — already covered by `libxcb-devel`

---

## Acceptance Criteria for Phase 2

| Criterion | Verification |
|-----------|--------------|
| `window_focused()` returns correct focus state on X11 | Manual test: focus/unfocus window, check behavior |
| `window_focused()` works on Wayland (via xdotool) | Manual test on Wayland session |
| Audio regression test passes on Linux | `cargo test --locked audio::regression` |
| No `target_os = "windows"` without Linux equivalent | `grep` audit shows all covered |
| CI passes on Linux | GitHub Actions green |
| Windows build still works | Cross-compile or verify no breaking changes |

---

## Suggested Execution Order

1. **Start with Part 2** (easiest — just error message fix)
2. **Do Part 3** (audit — quick, finds any hidden issues)
3. **Do Part 1** (most complex — window focus, needs testing)
4. **Do Part 4** (CI hardening — after core works)

---

## Post-Phase 2 / Polish (Tier 5)

The following items are **not Phase 2 blockers** and should be deferred to post-Phase 2 polish work:

- Gamepad/controller support (SDL2 via Macroquad)
- Wayland-native window focus (beyond xdotool fallback)
- Advanced CI hardening (headless audio, xvfb-run for focus tests)

---

## Notes

- **Don't break Windows** — always use `#[cfg]` not removal
- **Test on both X11 and Wayland** if possible (Fedora defaults to Wayland)
- **`xdotool` is the pragmatic choice** — works everywhere with XWayland, simple subprocess call
- **Rodio should work on Linux** — the error message fix may be all that's needed