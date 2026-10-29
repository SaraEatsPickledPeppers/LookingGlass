#!/bin/bash
# release.sh - Create a distributable Linux package (tarball or AppImage)
# Usage: ./tools/release.sh [tarball|appimage] [version] [--skip-build]
# Example: ./tools/release.sh tarball 0.32.0-preview.2

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${SCRIPT_DIR}/.."
cd "${ROOT_DIR}"

FORMAT="${1:-tarball}"
VERSION="${2:-0.32.0-preview.2}"
SKIP_BUILD="${3:-}"

if [[ "${FORMAT}" != "tarball" && "${FORMAT}" != "appimage" ]]; then
    echo "Error: Format must be 'tarball' or 'appimage'" >&2
    echo "Usage: $0 [tarball|appimage] [version] [--skip-build]" >&2
    exit 1
fi

EXE_PATH="target/release/looking-glass"

if [[ "${SKIP_BUILD}" != "--skip-build" ]]; then
    echo "Building release binary..."
    cargo build --release --locked
else
    echo "Skipping build (--skip-build specified)..."
fi

if [[ ! -f "${EXE_PATH}" ]]; then
    echo "Error: Binary not found at ${EXE_PATH}" >&2
    echo "Run without --skip-build to build, or ensure a release binary exists." >&2
    exit 1
fi

EXE_SHA256=$(sha256sum "${EXE_PATH}" | cut -d' ' -f1)
BUILD_DATE=$(date -u +%Y-%m-%d)
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
TARGET_TRIPLE="x86_64-unknown-linux-gnu"

BUILD_INFO=$(cat <<EOF
{
  "version": "${VERSION}",
  "build_date": "${BUILD_DATE}",
  "rust_version": "${RUST_VERSION}",
  "target_triple": "${TARGET_TRIPLE}",
  "executable_sha256": "${EXE_SHA256}",
  "engine_source_zip_sha256": "placeholder",
  "extractor_sha256": "placeholder"
}
EOF
)

echo "${BUILD_INFO}" > build_info.json

if [[ "${FORMAT}" == "tarball" ]]; then
    OUTPUT="looking-glass-linux-x86_64.tar.gz"
else
    OUTPUT="looking-glass-linux-x86_64.AppImage"
fi

echo "Creating ${FORMAT} package..."
python3 tools/package_linux.py \
    --exe "${EXE_PATH}" \
    --output "${OUTPUT}" \
    --format "${FORMAT}" \
    --build-info build_info.json \
    --verbose

echo ""
echo "Package created: ${OUTPUT}"
echo "SHA256: $(cat ${OUTPUT}.sha256 | cut -d' ' -f1)"