# Install and run Looking Glass on Windows

Looking Glass is an unfinished, independently written Rust recreation using your own American McGee's Alice game data. All 39 visits have completed automated campaign routes, but original-game fidelity and broader device validation remain unfinished. The current launch opens the main menu; see [the campaign audit](CAMPAIGN_RUN_AUDIT.md) for exact evidence.

## Install the Windows preview

**[Download for Windows — ZIP](https://github.com/looking-glass-project/LookingGlass/releases/download/v0.32.0-preview.2-setup.1/LookingGlass-Windows-x64-v0.32.0-preview.2-setup.1.zip)**

This link downloads the one package you need. The other GitHub release files
are optional developer, verification or video files; you do not need them to play.

1. Right-click the Windows ZIP and choose **Extract All**. Keep the entire folder
   somewhere writable, such as `Documents\LookingGlass`. Do not run it inside the ZIP.
2. Open the extracted folder and double-click **Launch.cmd**.
3. On first launch, the guided window defaults to local game files. It looks for game
   files beside the game, in Downloads, and at a previously saved location.
   If none are found, use **Choose archive** or **Choose folder**. Setup does not
   automatically select or start a download.
4. Choose your **Language**.
5. Select **Set up**. Setup checks and imports
   the files, then opens the game. Choose **New Game**.
   Next time, double-click **Launch.cmd** or the optional desktop shortcut.

You do not need to type commands, install Rust or Python, install 7-Zip separately,
or run as administrator. The Windows ZIP includes the small extractor it needs.
Setup uses the Windows PowerShell and .NET components already supplied with Windows.
It changes no machine-wide execution-policy setting.

Allow **3 GB free space** for the game, its download and saved progress. The
optional download is about **933 MB**, so its duration depends on your connection.
You can cancel from the setup window. If a connection fails, retry or use an
existing file; partial downloads are discarded rather than treated as complete.

## Where to get the game files

Use an existing compatible folder or archive that you are entitled to use.
The supported data is the **English 2011 vanilla** package. Setup's optional download
fetches **Alice1_2011_vanilla.7z** from
[American McGee's Alice (2011) on Internet Archive](https://archive.org/details/alice_202106).
The [file listing](https://archive.org/download/alice_202106) and its archive
metadata were checked on **1 October 2026**. Use files you are entitled to use.
Original game data is downloaded separately and is never bundled in the engine ZIP.

To opt in, select **Download from Internet Archive**, then **Set up**. A separate
confirmation asks whether you are entitled to download and use the files and
want to continue. It defaults to **No**. Declining leaves setup open without
starting a download or changing your existing configuration. Confirmation is
required each time you start setup's download option, including a retry or reuse
of its cached download. Selecting local files does not require this confirmation.

The optional download is hosted by that third-party service, not by Looking Glass.
Its availability and setup's checks do not establish redistribution permission
or your entitlement to use the files. Only download or import data you are
authorised to use. EA and the respective rights holders retain the original
game rights; Looking Glass claims none of them. This project is not endorsed by
or affiliated with EA or its licensors. See the [rights notice](../LEGAL.md).

For a manual download, select that exact `.7z` file under **Show all**. You do not
need the torrent, pictures or metadata files. In setup, use **Choose archive** and
select the downloaded file. You can also drag it onto **Setup.cmd**.

Already have extracted files? Select **Choose folder**, then choose any of:

- `Alice1`, containing `bin\base`;
- its parent folder, containing `Alice1\bin\base`;
- `bin\base` itself, containing the five base PK3 files and a supported language pack.

Setup copies an external folder's PK3 files into `private/game-data`; the source
folder stays unchanged. ZIP archives with the same base-folder layout are also
supported. Compatibility with 2000 retail data and third-party texture mods is
not established.
Looking Glass runs independently; you do not need to edit Steam configuration,
change Alice: Madness Returns settings, or run the original executable.

## Changing files, updates and saves

Open **Setup.cmd** to choose a different archive or folder. Setup validates the
required game and language packs and the map layout before changing the
remembered path.
Downloaded files must also match the expected size and SHA-256 checksum. Local
folders are checked for archive structure and map format, not every asset's bytes.

For an archive, setup imports the required game and language packs to
`private/game-data`. It does not extract or run the original executables. Imports
are isolated, and a validated import can be reused. An interrupted extraction may
leave an unused import folder; the previously configured game and saves are kept.

All local settings, imported files, downloads and saves are under `private/`.
External folders are copied; setup does not modify them. Moving the whole Looking
Glass folder keeps imported data working; desktop shortcuts must be recreated
after a move.

For updates, extract the new release into a **new folder** and keep the old one.
Follow that release's save-compatibility notes before copying your `private/`
folder. Setup never resets your saves. Original-game saves are not compatible.
Removing the Looking Glass folder removes its local saves/imports, so keep a backup;
an external data folder is unaffected.

## Choosing or changing the language

English, German, French, Spanish and Dutch are supported. Dutch uses English
spoken dialogue. Choose **Language** during setup.

To change it later, open **Setup.cmd**, select another language and click
**Apply language**. If its files are unavailable, choose a compatible folder,
archive or **Download**. Your game files, mods and saved progress are kept.
Use **Launch.cmd** to return to the game.

## Troubleshooting

| Problem | What to do |
| --- | --- |
| Nothing starts from the ZIP | Choose **Extract All** first, then open **Launch.cmd** in the extracted folder. |
| No game files found / Set up is disabled | Use **Choose archive** or **Choose folder**. The optional **Download from Internet Archive** requires a separate entitlement confirmation. |
| Download fails or is unavailable | Retry, or download the named archive yourself and choose it in setup. No previous game setting is replaced. |
| Integrity check fails | The download is incomplete or different. Obtain the compatible archive again, or select an existing compatible folder. |
| Missing pack / unsupported map format | Select the English 2011 vanilla archive or its Alice1 folder. |
| Unpacking fails | Check free space and that the download is complete. Retry setup. |
| Access denied | Move the whole folder outside Program Files to a writable location such as Documents. |
| Extractor missing | Extract the complete Windows ZIP again. Source users can install [7-Zip](https://www.7-zip.org/) or select already extracted files. |
| Source folder needs Rust | Use the Windows binary package, or follow the developer steps below. |
| A save fails after changing game data | Return to the previous base data and mods. |

This is an **unsigned preview**. Windows may identify it as an unknown publisher.
Check the release source and checksum; do not disable antivirus or system security.
On a managed computer, ask its administrator if policy blocks PowerShell.
Report launch errors with the exact message, Windows version and release name.
Do not post original game files, private paths or personal saves publicly.

## Build from source

Install a Windows Rust toolchain and the required Microsoft C++ build tools using the [official Rust installation instructions](https://www.rust-lang.org/tools/install). The project's minimum Rust version is declared in Cargo.toml (currently 1.86).

From the project folder:

```powershell
cargo build --release --locked
.\Setup.cmd
.\Launch.cmd
```

The compiler downloads the locked open-source dependencies. It does not fetch Alice game files. Advanced data selection is also available without setup:

```powershell
.\target\release\looking-glass.exe --data "D:\Games\Alice1\bin\base" --map gvillage
```

---

# Install and run Looking Glass on Linux

Looking Glass is an unfinished, independently written Rust recreation using your own American McGee's Alice game data. All 39 visits have completed automated campaign routes, but original-game fidelity and broader device validation remain unfinished. The current launch opens the main menu; see [the campaign audit](CAMPAIGN_RUN_AUDIT.md) for exact evidence.

## System dependencies

Install the following packages before running Looking Glass:

| Distribution | Command |
| --- | --- |
| **Fedora / RHEL / CentOS** | `sudo dnf install mesa-libGL libX11 libXcursor libXi libXrandr libXinerama alsa-lib libxkbcommon libxcb libxcb-icccm libxcb-image libxcb-keysyms libxcb-randr libxcb-render libxcb-shape libxcb-shm libxcb-sync libxcb-util libxcb-xfixes libxcb-xinerama libxcb-xkb libxkbcommon-x11` |
| **Debian / Ubuntu / Mint** | `sudo apt install libgl1 libx11-6 libxcursor1 libxi6 libxrandr2 libxinerama1 libasound2 libxkbcommon0 libxcb1 libxcb-icccm4 libxcb-image0 libxcb-keysyms1 libxcb-randr0 libxcb-render0 libxcb-shape0 libxcb-shm0 libxcb-sync1 libxcb-util1 libxcb-xfixes0 libxcb-xinerama0 libxcb-xkb1 libxkbcommon-x11-0` |
| **Arch / Manjaro** | `sudo pacman -S mesa libx11 libxcursor libxi libxrandr libxinerama alsa-lib libxkbcommon libxcb xcb-util xcb-util-image xcb-util-keysyms xcb-util-renderutil xcb-util-wm libxkbcommon-x11` |
| **openSUSE** | `sudo zypper install Mesa-libGL1 libX11-6 libXcursor1 libXi6 libXrandr2 libXinerama1 libasound2 libxkbcommon0 libxcb1 libxcb-icccm4 libxcb-image0 libxcb-keysyms1 libxcb-randr0 libxcb-render0 libxcb-shape0 libxcb-shm0 libxcb-sync1 libxcb-util1 libxcb-xfixes0 libxcb-xinerama0 libxcb-xkb1 libxkbcommon-x11-0` |
| **Void Linux** | `sudo xbps-install mesa-libs libX11 libXcursor libXi libXrandr libXinerama alsa-lib libxkbcommon libxcb xcb-util xcb-util-image xcb-util-keysyms xcb-util-renderutil xcb-util-wm libxkbcommon-x11` |

**Optional but recommended:**
- `xdotool` — window focus and automation (some window managers)
- `zenity` (GTK) or `kdialog` (KDE) — GUI file dialogs in Setup.sh
- `fuse3` — required for AppImage execution
- `libgtk-3-0` / `libgtk-4-1` — if using GTK file dialogs

## Install the Linux preview

Two distribution formats are available from the [GitHub Releases](https://github.com/looking-glass-project/LookingGlass/releases):

### Option 1: Tarball (`.tar.gz`) — recommended

1. Download `LookingGlass-Linux-x64-<version>.tar.gz`
2. Extract it to a writable location:
   ```bash
   tar -xzf LookingGlass-Linux-x64-<version>.tar.gz
   cd LookingGlass-Linux-x64-<version>
   ```
3. Make scripts executable (if not already):
   ```bash
   chmod +x Setup.sh Launch.sh
   ```
4. Run setup:
   ```bash
   ./Setup.sh
   ```
5. Launch the game:
   ```bash
   ./Launch.sh
   ```

### Option 2: AppImage — portable single file

1. Download `LookingGlass-Linux-x64-<version>.AppImage`
2. Make it executable:
   ```bash
   chmod +x LookingGlass-Linux-x64-<version>.AppImage
   ```
3. Run it directly:
   ```bash
   ./LookingGlass-Linux-x64-<version>.AppImage
   ```
   The AppImage bundles Setup and Launch; on first run it opens the setup wizard.

**AppImage notes:**
- Requires **FUSE 3** (`fuse3` package) for extraction at runtime
- On some distributions, enable AppImage integration or use `--appimage-extract-and-run` if FUSE is unavailable
- The AppImage is self-contained; no separate Setup.sh/Launch.sh files are extracted

## First-time setup

1. **Launch Setup.sh** (or the AppImage). The guided window defaults to local game files. It searches beside the executable, in `~/Downloads`, and at a previously saved location.
2. If no game files are found, use **Choose archive** or **Choose folder**.
3. Choose your **Language** (English, German, French, Spanish, Dutch; Dutch uses English spoken dialogue).
4. Select **Set up**. Setup validates and imports the required files, then opens the game. Choose **New Game**.
5. Subsequent launches: run `./Launch.sh` (tarball) or the AppImage again.

**Game files:** Same as Windows — use an existing compatible folder or archive you are entitled to use (English 2011 vanilla). The optional download fetches `Alice1_2011_vanilla.7z` from [Internet Archive](https://archive.org/details/alice_202106). See the Windows section for full details on file layout and entitlement.

## Changing files, updates and saves

- Open **Setup.sh** to choose a different archive or folder. Setup validates required game and language packs and the map layout before changing the remembered path.
- Downloaded files must match the expected size and SHA-256 checksum. Local folders are checked for archive structure and map format.
- For an archive, setup imports required packs to `private/game-data`. It does not extract or run the original executables. Imports are isolated and reusable.
- All local settings, imported files, downloads and saves are under `private/`. External folders are copied; setup does not modify them.
- Moving the whole Looking Glass folder keeps imported data working.
- For updates, extract the new release into a **new folder** and keep the old one. Follow that release's save-compatibility notes before copying your `private/` folder. Setup never resets your saves. Original-game saves are not compatible.
- Removing the Looking Glass folder removes its local saves/imports; keep a backup. An external data folder is unaffected.

## Choosing or changing the language

English, German, French, Spanish and Dutch are supported. Dutch uses English spoken dialogue. Choose **Language** during setup.

To change it later, open **Setup.sh**, select another language and click **Apply language**. If its files are unavailable, choose a compatible folder, archive or **Download**. Your game files, mods and saved progress are kept. Use **Launch.sh** to return to the game.

## Troubleshooting

| Problem | What to do |
| --- | --- |
| **AppImage won't run** | Install `fuse3` (`sudo dnf install fuse3` / `sudo apt install fuse3` / `sudo pacman -S fuse3`). Or run with `--appimage-extract-and-run`. |
| **Setup.sh / Launch.sh: Permission denied** | Run `chmod +x Setup.sh Launch.sh` in the extracted folder. |
| **Missing libraries (libGL, libX11, alsa, etc.)** | Install the system dependencies listed above for your distribution. |
| **Window doesn't focus / input issues** | Install `xdotool` (`sudo dnf install xdotool` / `sudo apt install xdotool` / `sudo pacman -S xdotool`). Some Wayland compositors need `xdotool` for focus helper. |
| **No file dialog in Setup.sh** | Install `zenity` (GTK) or `kdialog` (KDE): `sudo dnf install zenity` / `sudo apt install zenity` / `sudo pacman -S zenity`. |
| **No game files found / Set up is disabled** | Use **Choose archive** or **Choose folder**. The optional **Download from Internet Archive** requires a separate entitlement confirmation. |
| **Download fails or is unavailable** | Retry, or download the named archive yourself and choose it in setup. No previous game setting is replaced. |
| **Integrity check fails** | The download is incomplete or different. Obtain the compatible archive again, or select an existing compatible folder. |
| **Missing pack / unsupported map format** | Select the English 2011 vanilla archive or its Alice1 folder. |
| **Unpacking fails** | Check free space (3 GB) and that the download is complete. Retry setup. |
| **Access denied** | Move the whole folder to a writable location such as `~/Documents/LookingGlass`. Do not run from `/opt`, `/usr`, or read-only mounts. |
| **Extractor missing (tarball)** | Re-extract the complete tarball. Source users can install `p7zip` (`7z`) or select already extracted files. |
| **Wayland: cursor / scaling issues** | Try `GDK_BACKEND=x11 ./Launch.sh` or set `WINIT_UNIX_BACKEND=x11` to force X11 backend. |
| **Audio not working** | Ensure `alsa-lib` / `pipewire-alsa` is installed. Try `Launch.sh` with `ALSA_DEVICE=default` or your specific device. |

This is an **unsigned preview**. Verify the release source and checksum; do not disable system security. Report launch errors with the exact message, distribution, kernel version, and release name. Do not post original game files, private paths or personal saves publicly.

## Build from source on Linux

Install a Rust toolchain and system dependencies using the [official Rust installation instructions](https://www.rust-lang.org/tools/install). The project's minimum Rust version is declared in `Cargo.toml` (currently 1.86). Also install the system dependencies listed above.

From the project folder:

```bash
cargo build --release --locked
./Setup.sh
./Launch.sh
```

The compiler downloads the locked open-source dependencies. It does not fetch Alice game files. Advanced data selection is also available without setup:

```bash
./target/release/looking-glass --data "/home/user/Games/Alice1/bin/base" --map gvillage
```
