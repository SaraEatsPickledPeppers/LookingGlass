#!/usr/bin/env bash
# Visibility regression tests
# Ported from test_visibility.ps1

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

EXECUTABLE=""
DATA_DIR=""
SKIP_BUILD=false
VERBOSE=false

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    -e, --executable PATH    Path to the looking-glass executable (optional, builds if not provided)
    -d, --data PATH          Path to game data directory (optional)
    --skip-build             Skip building, use provided executable
    -v, --verbose            Enable verbose output
    -h, --help               Show this help

Example:
    $0
    $0 -e ./target/release/looking-glass --skip-build
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
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
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

# If no executable provided and not skipping build, build the project
if [[ -z "$EXECUTABLE" && "$SKIP_BUILD" == false ]]; then
    echo "Building looking-glass in release mode..."
    cd "$PROJECT_ROOT"
    cargo build --release --locked --target-dir private/visibility-test-target
    EXECUTABLE="$PROJECT_ROOT/private/visibility-test-target/release/looking-glass"
elif [[ -z "$EXECUTABLE" && "$SKIP_BUILD" == true ]]; then
    echo "Error: --executable is required when using --skip-build"
    usage
fi

if [[ -n "$EXECUTABLE" ]]; then
    EXECUTABLE=$(realpath "$EXECUTABLE")
fi

if [[ -n "$DATA_DIR" ]]; then
    DATA_DIR=$(realpath "$DATA_DIR")
fi

REPORT_DIR="private/visibility"
mkdir -p "$REPORT_DIR"

# Compute SHA256 of executable
SHA256_SUM=$(sha256sum "$EXECUTABLE" | cut -d' ' -f1)

# Initialize report JSON
REPORT_FILE="$REPORT_DIR/run-summary.json"
cat > "$REPORT_FILE" <<EOF
{
  "started_utc": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "executable": "$EXECUTABLE",
  "sha256": "$SHA256_SUM",
  "data_dir": "$DATA_DIR",
  "status": "running",
  "exit_code": null
}
EOF

echo "=== Visibility Regression Test Suite ==="
echo "Executable: $EXECUTABLE"
echo "SHA256: $SHA256_SUM"
echo "Data dir: ${DATA_DIR:-auto}"
echo "Report dir: $REPORT_DIR"
echo

LOG_FILE="$REPORT_DIR/run.log"

# Build command arguments
ARGS=("--visibility-check" "--no-audio")
if [[ -n "$DATA_DIR" ]]; then
    ARGS+=("--data" "$DATA_DIR")
fi

echo "Running visibility-check..."

EXIT_CODE=0
if [[ "$VERBOSE" == true ]]; then
    "$EXECUTABLE" "${ARGS[@]}" 2>&1 | tee "$LOG_FILE"
    EXIT_CODE=${PIPESTATUS[0]}
else
    "$EXECUTABLE" "${ARGS[@]}" > "$LOG_FILE" 2>&1
    EXIT_CODE=$?
fi

# Update report
if command -v jq &> /dev/null; then
    local tmp=$(mktemp)
    if [[ $EXIT_CODE -eq 0 ]]; then
        jq '.status = "passed" | .exit_code = 0' "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    else
        jq '.status = "failed" | .exit_code = '$EXIT_CODE "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    fi
fi

echo
echo "=== Summary ==="
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "Visibility checks passed. Report: private/visibility/village-report.json"
    exit 0
else
    echo "Visibility regression failed ($EXIT_CODE). See $LOG_FILE"
    exit 1
fi