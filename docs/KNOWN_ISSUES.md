# Prerelease limitations

These limitations apply to **v0.32.0-preview.2**, the experimental Windows public preview.

- Five automated campaign lineages complete all 39 visits; only one is an
  uninterrupted fresh run. This is route evidence, not a full visual/audio
  comparison. See [the campaign audit](CAMPAIGN_RUN_AUDIT.md).
- A reported waterside rope may be difficult to grab while swimming. The likely
  Dry Landing rope matches its authored length; exact location and native
  emergence/contact behavior remain unresolved.
- Material diagnostics include empty identifiers in fortress, school, temple
  and red-chess maps, plus `textures/notexture` in Funhouse and Hatter1. Their
  visible surfaces still need individual review.
- Camera framing, acting, facial synchronization, particle effects and enemy
  behavior retain documented fidelity gaps in their subsystem notes.
- Automated native tests used disabled audio. They do not certify audible
  mixing, clean-machine installation or every supported display/device.
- Walking uses a brisk 104 units/s pace. Walking jumps cover less horizontal
  distance than in preview.1; use the run modifier for longer jumps.
- Steam Deck / Proton has not been verified for this update.
- Current validation targets Windows x64 and the English 2011 vanilla data.
  Other editions, language/texture modifications and other platforms are not
  certified.

Use normal level exits to continue a campaign. The chapter chooser starts a new
visit. Keep backups of important saves before trying development updates.

---

## Linux-Specific Known Issues

These issues apply to the Linux build and are tracked for future phases.

- **AppImage requires FUSE at build time (not runtime)** — install `libfuse2` if building the AppImage locally. The resulting AppImage bundles FUSE and does not require it on the target system.
- **Window focus detection falls back to `xdotool` on Wayland** — install `xdotool` for the best experience. Native Wayland focus detection is not yet implemented.
- **GUI dialogs need `zenity`, `kdialog`, or `yad`** — install one of these for graphical setup dialogs (file pickers, confirmations, etc.). Without one, dialogs fall back to terminal prompts.
- **Gamepad support not yet implemented on Linux (Phase 7)** — keyboard/mouse only for now.
- **Wayland-native window focus not yet implemented (Phase 7)** — currently uses X11-compatible fallback via `xdotool`.
