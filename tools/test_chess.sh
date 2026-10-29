#!/usr/bin/env bash
# Chess regression tests
# Ported from test_chess.ps1

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

EXECUTABLE=""
DATA_DIR=""

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    -e, --executable PATH    Path to the looking-glass executable (required)
    -d, --data PATH          Path to game data directory (optional)
    -h, --help               Show this help

Example:
    $0 -e ./target/release/looking-glass
EOF
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--executable)
            EXECUTABLE="$2"
            shift 2
            ;;
        -d|--data)
            DATA_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

if [[ -z "$EXECUTABLE" ]]; then
    if [[ -f "$PROJECT_ROOT/target/release/looking-glass" ]]; then
        EXECUTABLE="$PROJECT_ROOT/target/release/looking-glass"
    elif [[ -f "$PROJECT_ROOT/target/debug/looking-glass" ]]; then
        EXECUTABLE="$PROJECT_ROOT/target/debug/looking-glass"
    else
        echo "Error: --executable is required (not found in default locations)"
        usage
    fi
fi

exec "$SCRIPT_DIR/test_runner.sh" \
    -e "$EXECUTABLE" \
    ${DATA_DIR:+-d "$DATA_DIR"} \
    -n chess \
    -c "chess-check chess-render-check chess-save-write chess-save-read"