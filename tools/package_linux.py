#!/usr/bin/env python3
"""
Assemble a Linux release package (AppImage or tarball) from explicit reviewed inputs.

This script creates a distributable Linux package containing:
- The looking-glass binary
- Launch.sh and Setup.sh entry points
- tools/linux_setup.sh core setup logic
- Documentation and legal files
- Desktop integration files (for AppImage)

Usage:
    python3 tools/package_linux.py --exe ./target/release/looking-glass \
        --output looking-glass-linux-x86_64.AppImage \
        --format appimage

    python3 tools/package_linux.py --exe ./target/release/looking-glass \
        --output looking-glass-linux-x86_64.tar.gz \
        --format tarball
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
import tarfile

ROOT = Path(__file__).resolve().parents[1]

# Files to include in the package (relative to ROOT)
PLAYER_FILES = (
    'Launch.sh',
    'Setup.sh',
    'tools/linux_setup.sh',
    'LICENSE-MIT',
    'LEGAL.md',
    'THIRD_PARTY_NOTICES.txt',
    'README.md',
    'docs/INSTALL.md',
    'docs/KNOWN_ISSUES.md',
    'docs/CONTROLS.md',
    'docs/SAVES.md',
    'docs/CAMPAIGN_RUN_AUDIT.md',
)

# Quick start guide for Linux
QUICK_START = """LOOKING GLASS - LINUX PREVIEW

1. Extract the package (AppImage: make executable and run; Tarball: extract and run Launch.sh)
2. Run Setup.sh to configure game data (or Launch.sh to auto-run setup)
3. Setup defaults to existing game files. Choose an archive or folder if needed,
   then select Set up. Setup checks and unpacks the files, then plays.

Next time: just run Launch.sh, or the AppImage directly.
To change game files: run Setup.sh.

GAME FILES
Use files you are entitled to use. The engine package includes no original game data.
Setup can fetch Alice1_2011_vanilla.7z from:
https://archive.org/details/alice_202106
Or use --data /path/to/Alice1 for existing English 2011 vanilla files.
Downloading requires confirming that you are entitled to download and use the files.
That download is hosted by a third party, not this project. Its availability
and setup's checks do not establish permission to download or use it.

SAVES AND UPDATES
Saves, settings and imported files are in private/. Keep a backup. Install a new
preview in a separate folder and follow its save compatibility notes before
copying private/. Do not delete your old installation until the new one works.

This is an unsigned, experimental preview. See docs/KNOWN_ISSUES.md and
docs/INSTALL.md for limitations and troubleshooting.

Esc: menu. F5/F9: quick save/load. Controls can be changed in Settings.
Use normal exits for campaign progress; Tab's chapter chooser starts a new visit.

This project is not endorsed by or affiliated with EA or its licensors.
American McGee's Alice is an Electronic Arts property. Original game assets
and trademarks remain with EA and/or their respective rights holders.
Looking Glass claims no ownership of them and grants no licence to them.
See LEGAL.md, LICENSE-MIT and THIRD_PARTY_NOTICES.txt for rights and notices.
"""

# Desktop entry for AppImage
DESKTOP_ENTRY = """[Desktop Entry]
Type=Application
Name=Looking Glass
Comment=Local Rust compatibility research for American McGee's Alice (2011)
Exec=looking-glass
Icon=looking-glass
Terminal=false
Categories=Game;
StartupNotify=true
"""

# AppRun script for AppImage
APPRUN_SCRIPT = """#!/bin/sh
# AppRun - Entry point for Looking Glass AppImage

# Get the directory where this AppImage is mounted
HERE="$(dirname "$(readlink -f "${0}")")"

# Set up library path for bundled libraries
export LD_LIBRARY_PATH="${HERE}/usr/lib:${HERE}/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH}"

# Set up XDG directories
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
export XDG_CONFIG_DIRS="${HERE}/usr/etc/xdg:${XDG_CONFIG_DIRS:-/etc/xdg}"

# Change to the AppImage directory so relative paths work
cd "${HERE}"

# Execute the game binary with all arguments
exec "${HERE}/looking-glass" "$@"
"""


def digest(data: bytes) -> str:
    """Compute SHA256 hash of data."""
    return hashlib.sha256(data).hexdigest()


def read_file_bytes(path: Path) -> bytes:
    """Read file as bytes."""
    return path.read_bytes()


def verify_file_hash(path: Path, expected_hash: str) -> bool:
    """Verify file matches expected SHA256 hash."""
    actual = digest(read_file_bytes(path))
    return actual == expected_hash


def create_appimage_structure(staging_dir: Path, exe_path: Path, build_info: dict) -> None:
    """Create the AppImage directory structure."""
    # AppDir structure
    usr_bin = staging_dir / 'usr' / 'bin'
    usr_lib = staging_dir / 'usr' / 'lib'
    usr_lib64 = staging_dir / 'usr' / 'lib' / 'x86_64-linux-gnu'
    usr_share = staging_dir / 'usr' / 'share'
    usr_share_applications = usr_share / 'applications'
    usr_share_icons = usr_share / 'icons' / 'hicolor' / '256x256' / 'apps'
    usr_share_metainfo = usr_share / 'metainfo'
    usr_etc = staging_dir / 'usr' / 'etc' / 'xdg'

    for d in (usr_bin, usr_lib, usr_lib64, usr_share_applications, usr_share_icons, usr_share_metainfo, usr_etc):
        d.mkdir(parents=True, exist_ok=True)

    # Copy binary
    shutil.copy2(exe_path, usr_bin / 'looking-glass')
    (usr_bin / 'looking-glass').chmod(0o755)

    # Create AppRun
    apprun = staging_dir / 'AppRun'
    apprun.write_text(APPRUN_SCRIPT)
    apprun.chmod(0o755)

    # Create desktop entry
    desktop = usr_share_applications / 'looking-glass.desktop'
    desktop.write_text(DESKTOP_ENTRY)

    # Create a simple placeholder icon (SVG)
    icon_svg = '''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">
  <rect width="256" height="256" fill="#1a1a2e"/>
  <text x="128" y="140" font-family="sans-serif" font-size="80" fill="#e94560" text-anchor="middle">🔍</text>
  <text x="128" y="200" font-family="sans-serif" font-size="16" fill="#ffffff" text-anchor="middle">Looking Glass</text>
</svg>'''
    (usr_share_icons / 'looking-glass.svg').write_text(icon_svg)

    # Copy desktop entry to root for AppImage integration
    shutil.copy2(desktop, staging_dir / 'looking-glass.desktop')
    shutil.copy2(usr_share_icons / 'looking-glass.svg', staging_dir / 'looking-glass.svg')

    # Create AppStream metainfo
    metainfo = f'''<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>looking-glass</id>
  <name>Looking Glass</name>
  <summary>Local Rust compatibility research for American McGee's Alice (2011)</summary>
  <description>
    <p>Looking Glass is a Rust reimplementation of American McGee's Alice (2011) using the Macroquad game framework.</p>
    <p>This is an experimental preview for research purposes. Use game files you are entitled to use.</p>
  </description>
  <launchable type="desktop-id">looking-glass.desktop</launchable>
  <url type="homepage">https://github.com/looking-glass-engine/looking-glass</url>
  <url type="bugtracker">https://github.com/looking-glass-engine/looking-glass/issues</url>
  <project_license>MIT</project_license>
  <developer_name>Looking Glass Contributors</developer_name>
  <releases>
    <release version="{build_info.get('version', '0.32.0-preview.2')}" date="{build_info.get('build_date', '')}"/>
  </releases>
  <categories>
    <category>Game</category>
  </categories>
  <keywords>
    <keyword>alice</keyword>
    <keyword>game</keyword>
    <keyword>rust</keyword>
  </keywords>
</component>'''
    (usr_share_metainfo / 'looking-glass.metainfo.xml').write_text(metainfo)


def copy_player_files(staging_dir: Path, target_subdir: str = '') -> dict:
    """Copy player files to staging directory. Returns dict of filename -> hash."""
    payload = {}
    for name in PLAYER_FILES:
        src = ROOT / name
        if not src.exists():
            print(f"Warning: {name} not found, skipping", file=sys.stderr)
            continue
        data = src.read_bytes()
        payload[name] = digest(data)
        dst = staging_dir / target_subdir / name
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(data)
    return payload


def build_appimage(exe_path: Path, output_path: Path, build_info: dict, verbose: bool = False) -> dict:
    """Build AppImage using appimagetool."""
    with tempfile.TemporaryDirectory() as tmpdir:
        staging = Path(tmpdir) / 'LookingGlass.AppDir'
        staging.mkdir(parents=True)

        if verbose:
            print(f"Creating AppImage structure in {staging}")

        # Create AppDir structure
        create_appimage_structure(staging, exe_path, build_info)

        # Copy player files to usr/share/looking-glass/
        player_dir = staging / 'usr' / 'share' / 'looking-glass'
        player_dir.mkdir(parents=True, exist_ok=True)
        copy_player_files(staging, 'usr/share/looking-glass')

        # Also copy to root for backward compatibility
        copy_player_files(staging, '')

        # Ensure shell scripts are executable
        for sh_file in staging.rglob('*.sh'):
            sh_file.chmod(0o755)
        # AppRun and binary
        (staging / 'AppRun').chmod(0o755)
        (staging / 'usr' / 'bin' / 'looking-glass').chmod(0o755)

        # Create manifest
        manifest = {
            'files': {name: digest((staging / name).read_bytes()) for name in PLAYER_FILES if (staging / name).exists()},
            'binary_sha256': build_info.get('executable_sha256', ''),
            'version': build_info.get('version', '0.32.0-preview.2'),
        }
        (staging / 'MANIFEST.json').write_text(json.dumps(manifest, indent=2) + '\n')

        # Build AppImage
        # Check for appimagetool (prefer system-installed, fallback to downloading)
        appimagetool = shutil.which('appimagetool')
        if not appimagetool:
            # Try to download it
            appimagetool_url = "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
            appimagetool_path = Path(tmpdir) / 'appimagetool.AppImage'
            if verbose:
                print("Downloading appimagetool...")
            subprocess.run(['curl', '-L', '-o', str(appimagetool_path), appimagetool_url], check=True)
            appimagetool_path.chmod(0o755)
            appimagetool = str(appimagetool_path)

        # Set ARCH for appimagetool
        env = os.environ.copy()
        env['ARCH'] = 'x86_64'

        if verbose:
            print(f"Running appimagetool: {appimagetool} {staging} {output_path}")

        try:
            subprocess.run([appimagetool, str(staging), str(output_path)], env=env, check=True)
        except subprocess.CalledProcessError as e:
            # Check if it's a FUSE issue
            if 'libfuse' in str(e).lower() or 'fuse' in str(e).lower():
                raise RuntimeError(
                    "AppImage creation failed: FUSE is required to run appimagetool (which is itself an AppImage). "
                    "Install FUSE (e.g., 'sudo dnf install fuse' or 'sudo apt-get install fuse') "
                    "or use the tarball format instead (--format tarball)."
                ) from e
            raise

        # Verify
        if not output_path.exists():
            raise RuntimeError("AppImage was not created")

        # Compute final hash
        final_hash = digest(output_path.read_bytes())
        size = output_path.stat().st_size

        return {'file': str(output_path), 'sha256': final_hash, 'bytes': size}


def build_tarball(exe_path: Path, output_path: Path, build_info: dict, verbose: bool = False) -> dict:
    """Build a tarball package."""
    with tempfile.TemporaryDirectory() as tmpdir:
        staging = Path(tmpdir) / 'LookingGlass'
        staging.mkdir(parents=True)

        if verbose:
            print(f"Creating tarball structure in {staging}")

        # Copy binary
        bin_dir = staging / 'bin'
        bin_dir.mkdir(parents=True)
        shutil.copy2(exe_path, bin_dir / 'looking-glass')
        (bin_dir / 'looking-glass').chmod(0o755)

        # Create launch wrapper that sets up paths correctly
        launch_wrapper = staging / 'Launch.sh'
        launch_wrapper.write_text("""#!/bin/bash
# Launch.sh - Entry point for Looking Glass (packaged version)

set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Binary is in bin/ next to this script
BINARY="${SCRIPT_DIR}/bin/looking-glass"

# Set up library path if there are bundled libraries
export LD_LIBRARY_PATH="${SCRIPT_DIR}/lib:${LD_LIBRARY_PATH}"

# Execute the binary with all arguments
exec "${BINARY}" "$@"
""")
        launch_wrapper.chmod(0o755)

        # Create setup wrapper
        setup_wrapper = staging / 'Setup.sh'
        setup_wrapper.write_text("""#!/bin/bash
# Setup.sh - Entry point for Looking Glass Setup (packaged version)

set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Core setup script
SETUP_CORE="${SCRIPT_DIR}/tools/linux_setup.sh"

# Export root for the core script
export LG_SETUP_ROOT="${SCRIPT_DIR}"

# Execute core setup script with all arguments
exec "${SETUP_CORE}" "$@"
""")
        setup_wrapper.chmod(0o755)

        # Copy player files
        copy_player_files(staging)

        # Ensure shell scripts are executable
        for sh_file in staging.rglob('*.sh'):
            sh_file.chmod(0o755)

        # Create MANIFEST.json
        manifest = {
            'files': {name: digest((staging / name).read_bytes()) for name in PLAYER_FILES if (staging / name).exists()},
            'binary_sha256': build_info.get('executable_sha256', ''),
            'version': build_info.get('version', '0.32.0-preview.2'),
        }
        (staging / 'MANIFEST.json').write_text(json.dumps(manifest, indent=2) + '\n')

        # Create tarball with proper permissions
        if verbose:
            print(f"Creating tarball: {output_path}")

        def _set_permissions(tarinfo: tarfile.TarInfo) -> tarfile.TarInfo:
            # Preserve executable bit for shell scripts and binaries
            if tarinfo.name.endswith('.sh') or tarinfo.name.endswith('/looking-glass'):
                tarinfo.mode = 0o755
            return tarinfo

        with tarfile.open(output_path, 'w:gz') as tar:
            tar.add(staging, arcname='LookingGlass', filter=_set_permissions)

        # Compute final hash
        final_hash = digest(output_path.read_bytes())
        size = output_path.stat().st_size

        return {'file': str(output_path), 'sha256': final_hash, 'bytes': size}


def package(exe: Path, output: Path, format: str, build_info: Path, verbose: bool = False) -> dict:
    """Main packaging function."""
    # Verify inputs exist
    if not exe.exists():
        raise ValueError(f'Executable not found: {exe}')
    if not build_info.exists():
        raise ValueError(f'Build info not found: {build_info}')
    if output.exists():
        raise ValueError(f'Output already exists: {output}')

    # Load build info
    info = json.loads(build_info.read_text(encoding='utf-8'))

    # Verify executable hash
    exe_data = exe.read_bytes()
    if info.get('executable_sha256') != digest(exe_data):
        raise ValueError('Executable differs from the recorded build.')

    if verbose:
        print(f"Packaging {exe} -> {output} (format: {format})")
        print(f"Build info: {info}")

    # Build based on format
    if format == 'appimage':
        result = build_appimage(exe, output, info, verbose)
    elif format == 'tarball':
        result = build_tarball(exe, output, info, verbose)
    else:
        raise ValueError(f'Unknown format: {format}')

    # Write checksum file
    checksum_path = output.with_suffix(output.suffix + '.sha256')
    checksum_path.write_text(f"{result['sha256']}  {output.name}\n")

    if verbose:
        print(f"Package created: {output}")
        print(f"SHA256: {result['sha256']}")
        print(f"Size: {result['bytes']} bytes")

    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True, help='Path to the looking-glass binary')
    parser.add_argument('--output', type=Path, required=True, help='Output package path')
    parser.add_argument('--format', choices=['appimage', 'tarball'], required=True, help='Package format')
    parser.add_argument('--build-info', type=Path, required=True, help='Path to build info JSON')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')

    args = parser.parse_args()

    try:
        result = package(args.exe, args.output, args.format, args.build_info, args.verbose)
        print(json.dumps(result, indent=2))
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()