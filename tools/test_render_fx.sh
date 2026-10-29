#!/usr/bin/env bash
# Render FX regression tests
# Ported from test_render_fx.ps1

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

EXECUTABLE=""
DATA_DIR=""
VERBOSE=false

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    -e, --executable PATH    Path to the looking-glass executable (required)
    -d, --data PATH          Path to game data directory (optional)
    -v, --verbose            Enable verbose output
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

EXECUTABLE=$(realpath "$EXECUTABLE")

if [[ -n "$DATA_DIR" ]]; then
    DATA_DIR=$(realpath "$DATA_DIR")
fi

REPORT_DIR="private/render-fx"
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
  "checks": []
}
EOF

CHECKS=("billboard-check" "render-fx-check" "fidelity-corpus-check" "visibility-check")
FAILED_CHECKS=()

update_report() {
    local check_name="$1"
    local exit_code="$2"
    local log_file="$3"

    if command -v jq &> /dev/null; then
        local tmp=$(mktemp)
        jq --arg name "$check_name" --argjson code "$exit_code" --arg log "$log_file" \
           '.checks += [{"name": $name, "exit_code": $code, "log": $log}]' \
           "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    fi
}

run_check() {
    local check="$1"
    local log_file="$REPORT_DIR/$check.log"

    echo "Running $check..."

    local args=("--$check" "--no-audio")
    if [[ -n "$DATA_DIR" ]]; then
        args+=("--data" "$DATA_DIR")
    fi

    local exit_code=0
    if [[ "$VERBOSE" == true ]]; then
        "$EXECUTABLE" "${args[@]}" 2>&1 | tee "$log_file"
        exit_code=${PIPESTATUS[0]}
    else
        "$EXECUTABLE" "${args[@]}" > "$log_file" 2>&1
        exit_code=$?
    fi

    update_report "$check" "$exit_code" "$log_file"

    # Verify executable hasn't changed
    CURRENT_SHA256=$(sha256sum "$EXECUTABLE" | cut -d' ' -f1)
    if [[ "$CURRENT_SHA256" != "$SHA256_SUM" ]]; then
        echo "Error: Executable changed during test run!"
        exit 1
    fi

    if [[ $exit_code -eq 0 ]]; then
        echo "PASS $check"
    else
        echo "FAIL $check (exit code: $exit_code)"
        echo "See log: $log_file"
        FAILED_CHECKS+=("$check")
    fi

    return $exit_code
}

echo "=== Render FX Regression Test Suite ==="
echo "Executable: $EXECUTABLE"
echo "SHA256: $SHA256_SUM"
echo "Data dir: ${DATA_DIR:-auto}"
echo "Report dir: $REPORT_DIR"
echo "Checks: ${CHECKS[*]}"
echo

for check in "${CHECKS[@]}"; do
    run_check "$check"
done

echo
echo "=== Summary ==="

if command -v jq &> /dev/null; then
    local tmp=$(mktemp)
    if [[ ${#FAILED_CHECKS[@]} -eq 0 ]]; then
        jq '.status = "passed"' "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    else
        jq --argjson failed "$(printf '%s\n' "${FAILED_CHECKS[@]}" | jq -R . | jq -s .)" \
           '.status = "failed" | .failed = $failed' \
           "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    fi
fi

if [[ ${#FAILED_CHECKS[@]} -eq 0 ]]; then
    echo "All checks passed!"
    exit 0
else
    echo "Failed checks: ${FAILED_CHECKS[*]}"
    exit 1
fi