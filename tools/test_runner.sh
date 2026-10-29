#!/usr/bin/env bash
# Unified test runner for Looking Glass regression tests
# Ported from PowerShell test scripts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
EXECUTABLE=""
DATA_DIR=""
TEST_NAME=""
CHECKS=()
REPORT_DIR=""
SAVE_DIR=""
VERBOSE=false
NO_AUDIO=true
SHA256_SUM=""

# Print usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    -e, --executable PATH    Path to the looking-glass executable (required)
    -d, --data PATH          Path to game data directory (optional, reads from private/data-path.txt)
    -n, --name NAME          Test suite name (e.g., ants, burrow, chess) (required)
    -c, --checks LIST        Space-separated list of checks to run (required)
    -r, --report-dir DIR     Report output directory (default: private/<name>)
    -s, --save-dir DIR       Save directory (default: private/saves)
    -v, --verbose            Enable verbose output
    --no-audio               Disable audio (default: true)
    -h, --help               Show this help

Examples:
    $0 -e ./target/release/looking-glass -n ants -c "ant-check ant-render-check ant-save-write ant-save-read pool-check potears2-check"
    $0 -e ./target/release/looking-glass -n chess -c "chess-check chess-render-check chess-save-write chess-save-read"
EOF
    exit 1
}

# Parse arguments
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
        -n|--name)
            TEST_NAME="$2"
            shift 2
            ;;
        -c|--checks)
            IFS=' ' read -ra CHECKS <<< "$2"
            shift 2
            ;;
        -r|--report-dir)
            REPORT_DIR="$2"
            shift 2
            ;;
        -s|--save-dir)
            SAVE_DIR="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --no-audio)
            NO_AUDIO=true
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

# Validate required arguments
if [[ -z "$EXECUTABLE" ]]; then
    echo -e "${RED}Error: --executable is required${NC}"
    usage
fi

if [[ -z "$TEST_NAME" ]]; then
    echo -e "${RED}Error: --name is required${NC}"
    usage
fi

if [[ ${#CHECKS[@]} -eq 0 ]]; then
    echo -e "${RED}Error: --checks is required${NC}"
    usage
fi

if [[ ! -f "$EXECUTABLE" ]]; then
    echo -e "${RED}Error: Executable not found: $EXECUTABLE${NC}"
    exit 1
fi

# Resolve executable path
EXECUTABLE=$(realpath "$EXECUTABLE")

# Set default report directory
if [[ -z "$REPORT_DIR" ]]; then
    REPORT_DIR="private/$TEST_NAME"
fi

# Set default save directory
if [[ -z "$SAVE_DIR" ]]; then
    SAVE_DIR="private/saves"
fi

# Resolve data directory if provided
if [[ -n "$DATA_DIR" ]]; then
    DATA_DIR=$(realpath "$DATA_DIR")
fi

# Create report directory
mkdir -p "$REPORT_DIR"

# Compute SHA256 of executable
SHA256_SUM=$(sha256sum "$EXECUTABLE" | cut -d' ' -f1)

# Initialize report JSON
REPORT_FILE="$REPORT_DIR/report.json"
cat > "$REPORT_FILE" <<EOF
{
  "executable": "$EXECUTABLE",
  "sha256": "$SHA256_SUM",
  "data_dir": "$DATA_DIR",
  "save_dir": "$SAVE_DIR",
  "no_audio": $NO_AUDIO,
  "checks": []
}
EOF

# Function to update report JSON with check result
update_report() {
    local check_name="$1"
    local exit_code="$2"
    local log_file="$3"

    # Use jq to update the JSON report
    if command -v jq &> /dev/null; then
        local tmp=$(mktemp)
        jq --arg name "$check_name" --argjson code "$exit_code" --arg log "$log_file" \
           '.checks += [{"name": $name, "exit_code": $code, "log": $log}]' \
           "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    else
        # Fallback: append to a simple text report
        echo "$check_name: exit_code=$exit_code log=$log_file" >> "$REPORT_DIR/report.txt"
    fi
}

# Function to run a single check
run_check() {
    local check="$1"
    local log_file="$REPORT_DIR/$check.log"

    echo -e "${YELLOW}Running $check...${NC}"

    # Build command arguments
    local args=("--$check")
    if [[ "$NO_AUDIO" == true ]]; then
        args+=("--no-audio")
    fi
    if [[ -n "$DATA_DIR" ]]; then
        args+=("--data" "$DATA_DIR")
    fi
    if [[ -n "$SAVE_DIR" ]]; then
        args+=("--save-dir" "$SAVE_DIR")
    fi

    # Run the check, capturing both stdout and stderr to log file
    local exit_code=0
    if [[ "$VERBOSE" == true ]]; then
        "$EXECUTABLE" "${args[@]}" 2>&1 | tee "$log_file"
        exit_code=${PIPESTATUS[0]}
    else
        "$EXECUTABLE" "${args[@]}" > "$log_file" 2>&1
        exit_code=$?
    fi

    # Update report
    update_report "$check" "$exit_code" "$log_file"

    if [[ $exit_code -eq 0 ]]; then
        echo -e "${GREEN}PASS $check${NC}"
    else
        echo -e "${RED}FAIL $check (exit code: $exit_code)${NC}"
        echo -e "${RED}See log: $log_file${NC}"
    fi

    return $exit_code
}

# Main execution
echo "=== $TEST_NAME Regression Test Suite ==="
echo "Executable: $EXECUTABLE"
echo "SHA256: $SHA256_SUM"
echo "Data dir: ${DATA_DIR:-auto}"
echo "Save dir: $SAVE_DIR"
echo "Report dir: $REPORT_DIR"
echo "No audio: $NO_AUDIO"
echo "Checks: ${CHECKS[*]}"
echo

FAILED_CHECKS=()

for check in "${CHECKS[@]}"; do
    if ! run_check "$check"; then
        FAILED_CHECKS+=("$check")
    fi

    # Verify executable hasn't changed
    CURRENT_SHA256=$(sha256sum "$EXECUTABLE" | cut -d' ' -f1)
    if [[ "$CURRENT_SHA256" != "$SHA256_SUM" ]]; then
        echo -e "${RED}Error: Executable changed during test run!${NC}"
        exit 1
    fi
done

echo
echo "=== Summary ==="
if [[ ${#FAILED_CHECKS[@]} -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
    # Update report status
    if command -v jq &> /dev/null; then
        local tmp=$(mktemp)
        jq '.status = "passed"' "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    fi
    exit 0
else
    echo -e "${RED}Failed checks: ${FAILED_CHECKS[*]}${NC}"
    if command -v jq &> /dev/null; then
        local tmp=$(mktemp)
        jq --argjson failed "$(printf '%s\n' "${FAILED_CHECKS[@]}" | jq -R . | jq -s .)" \
           '.status = "failed" | .failed = $failed' \
           "$REPORT_FILE" > "$tmp" && mv "$tmp" "$REPORT_FILE"
    fi
    exit 1
fi