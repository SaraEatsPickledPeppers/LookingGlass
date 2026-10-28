#!/bin/bash
# Launch.sh - Linux entry point for Looking Glass
# Mirrors Launch.cmd behavior for Linux

set -euo pipefail

# Parse dev flags
VERBOSE=false
DEV_MODE=false
NO_BUILD=false
ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --dev)
            DEV_MODE=true
            shift
            ;;
        --no-build)
            NO_BUILD=true
            shift
            ;;
        --help|-h)
            cat <<EOF
Usage: ./Launch.sh [OPTIONS] [--] [GAME_ARGS...]

Options:
  --verbose, -v    Log which binary is being used and arguments passed
  --dev            Force cargo build even if binary exists (for development)
  --no-build       Skip auto-build, fail if binary missing
  --help, -h       Show this help message

All remaining arguments are passed through to the game binary.
EOF
            exit 0
            ;;
        --)
            shift
            ARGS+=("$@")
            break
            ;;
        *)
            ARGS+=("$1")
            shift
            ;;
    esac
done

# Change to script directory (equivalent to pushd "%~dp0")
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Verbose logging function
log_verbose() {
    if [[ "$VERBOSE" == true ]]; then
        echo "[Launch.sh] $*"
    fi
}

# Function to run the binary with proper exit handling
run_binary() {
    local binary="$1"
    shift
    log_verbose "Using binary: $binary"
    log_verbose "Arguments: $*"
    "$binary" "$@"
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        log_verbose "Binary exited with code: $exit_code"
    fi
    exit $exit_code
}

# Check for binaries in priority order
# 1. ./looking-glass (pre-built binary in project root)
if [[ -f "./looking-glass" && -x "./looking-glass" && "$DEV_MODE" == false ]]; then
    log_verbose "Found pre-built binary at ./looking-glass"
    run_binary "./looking-glass" "${ARGS[@]}"
fi

# 2. ./private/heart-of-darkness/looking-glass (if exists)
if [[ -f "./private/heart-of-darkness/looking-glass" && -x "./private/heart-of-darkness/looking-glass" && "$DEV_MODE" == false ]]; then
    log_verbose "Found binary at ./private/heart-of-darkness/looking-glass"
    run_binary "./private/heart-of-darkness/looking-glass" --save-dir "$SCRIPT_DIR/private/playtest-saves" "${ARGS[@]}"
fi

# 3. ./private/playtest/looking-glass (if exists)
if [[ -f "./private/playtest/looking-glass" && -x "./private/playtest/looking-glass" && "$DEV_MODE" == false ]]; then
    log_verbose "Found binary at ./private/playtest/looking-glass"
    run_binary "./private/playtest/looking-glass" --save-dir "$SCRIPT_DIR/private/playtest-saves" "${ARGS[@]}"
fi

# 4. ./target/release/looking-glass (cargo build --release output)
if [[ -f "./target/release/looking-glass" && -x "./target/release/looking-glass" && "$DEV_MODE" == false ]]; then
    log_verbose "Found cargo release binary at ./target/release/looking-glass"
    run_binary "./target/release/looking-glass" "${ARGS[@]}"
fi

# If --no-build is set, fail here
if [[ "$NO_BUILD" == true ]]; then
    echo "Error: No binary found and --no-build specified" >&2
    echo "Checked locations:" >&2
    echo "  ./looking-glass" >&2
    echo "  ./private/heart-of-darkness/looking-glass" >&2
    echo "  ./private/playtest/looking-glass" >&2
    echo "  ./target/release/looking-glass" >&2
    exit 1
fi

# Check for cargo and build if needed
if command -v cargo >/dev/null 2>&1; then
    log_verbose "Building with cargo build --release --locked"
    cargo build --release --locked
    if [[ $? -ne 0 ]]; then
        echo "Error: Cargo build failed" >&2
        exit 1
    fi

    if [[ -f "./target/release/looking-glass" && -x "./target/release/looking-glass" ]]; then
        log_verbose "Build successful, running ./target/release/looking-glass"
        run_binary "./target/release/looking-glass" "${ARGS[@]}"
    else
        echo "Error: Build succeeded but binary not found at ./target/release/looking-glass" >&2
        exit 1
    fi
else
    echo "Error: No binary found and cargo not available to build" >&2
    echo "Checked locations:" >&2
    echo "  ./looking-glass" >&2
    echo "  ./private/heart-of-darkness/looking-glass" >&2
    echo "  ./private/playtest/looking-glass" >&2
    echo "  ./target/release/looking-glass" >&2
    echo "" >&2
    echo "Install Rust from https://rustup.rs/ or download a pre-built release." >&2
    exit 1
fi