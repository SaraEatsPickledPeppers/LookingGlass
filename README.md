# Looking Glass — American McGee's Alice Re-Remaster (Linux Rewrite)

**Play the whole game, from the opening to the final boss, ending and credits.**
All **39 campaign visits**, including return visits, are playable in this
experimental **Linux-first** preview.

Looking Glass is an unofficial **re-remaster of American McGee's Alice**: it
brings the **2011 rerelease's game data** into a newly written, open-source Rust
engine with guided setup, configurable controller support and modern display
options. It uses your own compatible game files and runs independently of the
original game executable, gameplay DLL and Alice: Madness Returns.

This project is a **complete Linux rewrite** of the original Windows-only engine.
The game engine now runs natively on Linux (Wayland/X11) with full feature parity:
gamepad support, audio device selection, performance profiling, structured logging,
and automated validation (689 tests passing).

[Visit the Looking Glass website](https://looking-glass-project.github.io/LookingGlass/) ·
**[Download for Linux](https://github.com/looking-glass-project/LookingGlass/releases)** ·
[Windows download](https://github.com/looking-glass-project/LookingGlass/releases/download/v0.32.0-preview.2-setup.1/LookingGlass-Windows-x64-v0.32.0-preview.2-setup.1.zip) ·
[Watch the 78-second story showcase](#gameplay-showcase) ·
[Report a bug](https://github.com/looking-glass-project/LookingGlass/issues/new/choose)

**One download, three steps (Linux):**
1. Download the Linux tarball from [releases](https://github.com/looking-glass-project/LookingGlass/releases)
2. Extract: `tar -xzf looking-glass-linux-x86_64.tar.gz && cd LookingGlass`
3. Run `./Launch.sh` — guided setup handles game files, then starts the game

**Or build from source:**
```bash
git clone https://github.com/SaraEatsPickledPeppers/LookingGlass
cd LookingGlass
cargo build --release --locked
./Setup.sh --download   # or ./Setup.sh --data /path/to/Alice1
./Launch.sh
```

This is an experimental preview, with remaining fidelity and device-testing gaps.
The engine download includes no original game data. You need compatible files
you are entitled to use. **American McGee's Alice belongs to EA; original game
content remains with EA and its respective rights holders.** Looking Glass claims
no ownership of that content. [Rights and distribution notice](LEGAL.md).

---

## 🎮 Gameplay Showcase

[![Watch the story and gameplay showcase: Cheshire, the Rabbit and Pandemonium's minecart](docs/media/gameplay/story-preview.gif)](https://looking-glass-project.github.io/LookingGlass/#showcase)

**[Watch the new 78-second showcase in your browser](https://looking-glass-project.github.io/LookingGlass/#showcase)** ·
[Download the video](https://github.com/looking-glass-project/LookingGlass/releases/download/v0.32.0-preview.1/LookingGlass-Story-Showcase.mp4)

Cutscenes from the **first two chapters — Dementia and Pandemonium** lead into
combat and exploration. Meet the Rabbit and Cheshire Cat, collect the Vorpal
Blade, hear the Gnome's warning through subtitles, and take the minecart ride.
Actual in-engine footage at normal speed, edited with titles and transitions;
720p, 30 fps. **The recordings are silent; dialogue subtitles are enabled.**

Want more of the opening story? Two separate one-minute edits:

- **[Chapter 1: Dementia](https://github.com/looking-glass-project/LookingGlass/releases/download/v0.32.0-preview.1/LookingGlass-Chapter-1-Dementia.mp4)** — Alice's arrival, Cheshire, the Blade and the little door.
- **[Chapter 2: Pandemonium](https://github.com/looking-glass-project/LookingGlass/releases/download/v0.32.0-preview.1/LookingGlass-Chapter-2-Pandemonium.mp4)** — the Gnome's warning and the minecart sequence.

[Recording and editing details](docs/media/gameplay/README.md#story-and-opening-chapter-videos).

---

## ✨ Why Try Looking Glass?

The practical benefits are easier setup, flexible controls and a game engine
that can be inspected and improved. These conveniences are built into this
project:

| Feature | What it means for you |
| --- | --- |
| **Full playable campaign** | Start a new game and play all 39 story visits through the final boss, ending and credits, with campaign progress and saves carried between levels. |
| **Guided installation** | One setup script finds, optionally downloads, checks and unpacks compatible game files, then starts the game. No compiler, separate extractor or changes to the original executable are needed. |
| **Built-in controller support** | Play and navigate menus with a PS4/Xbox/controller. Rebind buttons, adjust stick sensitivity and deadzones, and invert the camera. An active controller disconnect pauses play. |
| **Modern display and camera options** | Choose window resolutions including 1080p, 1440p and 4K, or borderless fullscreen at your desktop resolution. Adjust third-person camera distance, mouse sensitivity and look inversion in the settings. |
| **Easy chapter replay** | Browse all 39 story visits, including return visits, and choose a difficulty from the chapter menu. Each selection starts a fresh visit; normal exits carry your campaign forward. |
| **Save protection and recovery** | Four manual slots, quick save and a separate autosave have picture previews. Each slot keeps a previous validated copy, which the loader can recover if the current file is damaged. |
| **Source that can be improved** | The newly written engine code is available in this project under MIT. Contributors can inspect behavior, fix bugs and extend it, with automated gameplay and save checks to help catch regressions. |
| **Linux-native** | Runs natively on Wayland and X11 with full feature parity. No Wine/Proton needed for the engine itself. |

Looking Glass retains the original artwork, models, music and voices from your
game files. Its focus is rebuilding the engine and gameplay with these practical
conveniences; visual and behavioral fidelity are still being refined.

---

## 🏗️ Recreated Gameplay

- **Campaign progression:** story-ordered levels and return visits, quest items,
  puzzles, moving machinery and persistent world state.
- **Combat:** the weapon and power-up systems, enemy encounters and boss fights.
- **Traversal:** rope climbing and swinging, ledge grabs and pull-ups, swimming,
  currents and breath management.
- **Story and atmosphere:** scripted cameras and character performances,
  conversations, Cheshire hints, music, voices and positional sound effects.

These systems are implemented with remaining gaps documented below. They are
recreations of the original game's features, alongside the conveniences above.

---

## 📊 Current Status

**The full campaign is playable; this re-remaster is still an experimental preview.**

All 39 campaign visits have been completed in automated native playthroughs:
three Easy and two Normal runs, including one uninterrupted fresh campaign.
The other four used earned checkpoints while fixes were developed. The ending
film and credits were also checked in the viewer. These results do not establish
complete visual, audio or original-game fidelity. See the
[campaign audit](docs/CAMPAIGN_RUN_AUDIT.md) and
[known issues](docs/KNOWN_ISSUES.md) for exact coverage and remaining work.

**Platform support:**
- **Linux (primary):** x86_64, Wayland + X11, Vulkan 1.2+, tested on Ubuntu 22.04+, Fedora 38+, Arch, Steam Deck (Desktop mode)
- **Windows:** x64, same feature set via conditional compilation
- **Game data:** 2011 vanilla data set in English, German, French, Spanish, Dutch
- **Saves:** Persistent campaign saves, own format (original saves not importable)

Expect remaining differences in cutscene staging, effects, materials and enemy
behavior, plus a reported waterside rope-grab issue. Broader PC and audible
playthrough validation remain unfinished.

---

## 🎬 Gameplay Videos

Short clips recorded in the Linux preview, at normal speed. These recordings
are **silent**. Select a preview to watch or download the full video.

| Dry Landing | Pool of Tears | Pale Realm |
| --- | --- | --- |
| [![Cards combat against an ant guard](docs/media/gameplay/dry-landing.gif)](docs/media/gameplay/dry-landing.mp4) | [![Exploring the waterfall path](docs/media/gameplay/pool-of-tears.gif)](docs/media/gameplay/pool-of-tears.mp4) | [![Walking through the chess town](docs/media/gameplay/pale-realm.gif)](docs/media/gameplay/pale-realm.mp4) |
| Cards combat · [Watch video, 11 seconds](docs/media/gameplay/dry-landing.mp4) | Waterfall exploration · [Watch video, 10 seconds](docs/media/gameplay/pool-of-tears.mp4) | Chess town exploration · [Watch video, 14 seconds](docs/media/gameplay/pale-realm.mp4) |

Full videos are 720p. [Recording details](docs/media/gameplay/README.md).

---

## 📥 Install and Play (Linux)

### Option 1: Pre-built Release (Recommended)

1. **Download** the Linux tarball from the latest release:
   [`looking-glass-linux-x86_64.tar.gz`](https://github.com/looking-glass-project/LookingGlass/releases)
2. **Extract** it:
   ```bash
   tar -xzf looking-glass-linux-x86_64.tar.gz
   cd LookingGlass
   ```
3. **Run the launcher:**
   ```bash
   ./Launch.sh
   ```
   The guided setup will find or download compatible game files, then start the game.
4. **Next time**, just run `./Launch.sh` again (or create a desktop shortcut from the extracted folder).

### Option 2: Build from Source

```bash
# Prerequisites (Fedora/RHEL)
sudo dnf install -y gcc clang libglvnd-devel libx11-devel libasound2-devel \
    pkg-config p7zip zenity xdotool

# Prerequisites (Ubuntu/Debian)
sudo apt-get install -y build-essential libgl1-mesa-dev libx11-dev libasound2-dev \
    pkg-config p7zip-full zenity xdotool

# Build
git clone https://github.com/SaraEatsPickledPeppers/LookingGlass
cd LookingGlass
cargo build --release --locked

# Setup game data (downloads from Internet Archive or uses local files)
./Setup.sh --download   # or ./Setup.sh --data /path/to/Alice1

# Launch
./Launch.sh
```

### System Requirements (Linux)

| Requirement | Minimum |
| --- | --- |
| **Architecture** | x86_64 |
| **Kernel** | Linux 5.15+ |
| **Graphics** | Vulkan 1.2+ capable GPU (Intel/AMD/NVIDIA with proprietary or Mesa drivers) |
| **Display** | Wayland or X11 |
| **Memory** | 4 GB RAM (8 GB recommended) |
| **Storage** | 3 GB free space (game data ~933 MB additional if downloading) |
| **Audio** | PipeWire or PulseAudio |

Tested on: Ubuntu 22.04+, Fedora 38+, Arch Linux, Steam Deck (Desktop mode).

**Note:** The Linux build uses the same save format and game-data detection as Windows.
If you have the Windows package extracted, you can copy the `GameData` folder to the
Linux extract location to avoid re-downloading.

### Game Files

The supported file is **Alice1_2011_vanilla.7z** from
[American McGee's Alice (2011) on Internet Archive](https://archive.org/details/alice_202106).
Use **Choose archive** / **Choose folder** for files you already have. To download
instead, select **Download from Internet Archive**, select **Set up**, and confirm
your entitlement. The confirmation defaults to **No**; setup never switches to a
download because local files are missing. Use game files you are entitled to use. Original game
data is not included in the Looking Glass release.

That download is hosted by a third party, not this project. Its availability
and setup's integrity checks do not establish permission to download or use it.
See the [game-data and media notice](LEGAL.md#what-the-project-distributes).

See [installation and troubleshooting](docs/INSTALL.md), including updates and
building from source. [Release notes and checksums](https://github.com/looking-glass-project/LookingGlass/releases/tag/v0.32.0-preview.2-setup.1)
describe the exact package and its verification limits.

Use the in-game settings to view or change controls. **Esc** opens the menu,
**F5/F9** quick-save/load, and **Tab** opens the chapter chooser. Choosing a
chapter starts a new visit; use normal exits to carry campaign progress.

---

## 🔧 Build and Contribute

```bash
# Run all tests (689 tests)
cargo test --locked

# Build release binary
cargo build --release --locked

# Run with game data configured
./Launch.sh
```

The minimum Rust version is declared in [Cargo.toml](Cargo.toml). The dependency
versions are locked. Building does not fetch original game data.

[Contribution guidance](CONTRIBUTING.md) explains the source boundary, testing
and bug-report requirements. Development shortcuts are under
[tools/launchers](tools/launchers/README.md); ordinary play uses root `Launch.sh`.

### Creating a Distributable Package

```bash
# After building release
cargo build --release --locked

# Create build info (CI does this automatically)
EXE_SHA256=$(sha256sum target/release/looking-glass | cut -d' ' -f1)
cat > build_info.json <<EOF
{
  "version": "0.32.0-preview.2",
  "build_date": "$(date -u +%Y-%m-%d)",
  "rust_version": "1.86",
  "target_triple": "x86_64-unknown-linux-gnu",
  "executable_sha256": "$EXE_SHA256",
  "engine_source_zip_sha256": "placeholder",
  "extractor_sha256": "placeholder"
}
EOF

# Create tarball package
python3 tools/package_linux.py \
  --exe target/release/looking-glass \
  --output looking-glass-linux-x86_64.tar.gz \
  --format tarball \
  --build-info build_info.json \
  --verbose

# Or create AppImage (requires FUSE)
python3 tools/package_linux.py \
  --exe target/release/looking-glass \
  --output looking-glass-linux-x86_64.AppImage \
  --format appimage \
  --build-info build_info.json \
  --verbose
```

The `tools/package_linux.py` script creates a complete package with:
- The `looking-glass` binary
- `Launch.sh` and `Setup.sh` entry points
- `tools/linux_setup.sh` core setup logic
- Documentation and legal files
- Desktop integration (for AppImage)

Automated releases are configured in `.github/workflows/release.yml` — push a tag
like `v0.32.0` to trigger a Linux build and release upload.

---

## 📚 Documentation

- [Installation](docs/INSTALL.md) and [controls](docs/CONTROLS.md)
- [Campaign route audit](docs/CAMPAIGN_RUN_AUDIT.md) and [known issues](docs/KNOWN_ISSUES.md)
- [Validation](docs/VALIDATION.md) and [save behavior](docs/SAVES.md)
- [Release preparation](docs/RELEASE_PLAN.md)
- [Code signing policy and privacy](docs/CODE_SIGNING.md) — application awaiting review; current preview unsigned
- [Earlier development notes](docs/DEVELOPMENT_HISTORY.md)
- [Linux performance guide](docs/PERFORMANCE_LINUX.md)

---

## 🚀 Future: Console-First Game & Mod Launcher

The engine is complete and stable. The next phase is building a **functional-first
game & mod launcher as a console application** (TUI/CLI) before making a GUI launcher.

**Planned launcher features:**
- Game registry: scan Steam, Heroic, Lutris, Flatpak, AppImage, native binaries
- Unified launch abstraction: `steam://`, `flatpak run`, Wine/Proton, native
- Mod management: NexusMods/Mod.io integration, load ordering, profiles
- Save management: backup/restore/sync per game/profile
- TUI interface: `ratatui`/`crossterm` based, vim-style keybindings
- Plugin system: Lua/WASM for custom launch logic

See [WHERE_WE_ARE.md](WHERE_WE_ARE.md) for the full roadmap.

---

## ⚖️ License and Independence

Newly authored source and documentation use the [MIT license](LICENSE-MIT).
Original game assets, gameplay footage and third-party components are excluded
from that license.
**This project is not endorsed by or affiliated with EA or its licensors.**
EA and the respective rights holders retain all rights in the original game,
its assets and trademarks. The project hosts no playable original game-data
packages; published gameplay footage is described separately in the
[rights, ownership and distribution notice](LEGAL.md).