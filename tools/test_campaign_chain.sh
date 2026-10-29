#!/usr/bin/env bash
# Campaign chain validation tests
# Ported from test_campaign_chain.ps1

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

EXECUTABLE=""
DATA_DIR=""
DIFFICULTY="normal"
TO=""
EXPECT=""
PINNED=false
SKIP_BUILD=false
DETERMINISM=false
INSIDE_ANODE_SEAT=false
VERBOSE=false

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    -e, --executable PATH    Path to the looking-glass executable (optional, builds if not provided)
    -d, --data PATH          Path to game data directory (optional)
    --difficulty LEVEL       Difficulty: easy, normal, hard, nightmare (default: normal)
    --to TARGET              Campaign target (visit number or map\$entrance)
    --expect VALUE           Expectation: frontier=<n|map\$entrance> or no-checkpoint
    --pinned                 Run pinned tests (Normal difficulty only)
    --skip-build             Skip building, use provided executable
    --determinism            Run determinism check (repeat run)
    --inside-anode-seat      Run native save chain (opens window)
    -v, --verbose            Enable verbose output
    -h, --help               Show this help

Examples:
    $0
    $0 --pinned --determinism
    $0 -e ./target/release/looking-glass --skip-build --pinned
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
        --difficulty)
            DIFFICULTY="$2"
            shift 2
            ;;
        --to)
            TO="$2"
            shift 2
            ;;
        --expect)
            EXPECT="$2"
            shift 2
            ;;
        --pinned)
            PINNED=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --determinism)
            DETERMINISM=true
            shift
            ;;
        --inside-anode-seat)
            INSIDE_ANODE_SEAT=true
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

# Validate pinned + difficulty
if [[ "$PINNED" == true && "$DIFFICULTY" != "normal" ]]; then
    echo "Error: --pinned records Normal-difficulty outcomes: run it without --difficulty."
    exit 1
fi

# If no executable provided and not skipping build, build the project
if [[ -z "$EXECUTABLE" && "$SKIP_BUILD" == false ]]; then
    echo "Building looking-glass in release mode..."
    cd "$PROJECT_ROOT"
    cargo build --release --locked --target-dir private/campaign-chain-target
    EXECUTABLE="$PROJECT_ROOT/private/campaign-chain-target/release/looking-glass"
elif [[ -z "$EXECUTABLE" && "$SKIP_BUILD" == true ]]; then
    echo "Error: --executable is required when using --skip-build"
    usage
fi

if [[ -n "$EXECUTABLE" ]]; then
    EXECUTABLE=$(realpath "$EXECUTABLE")
fi

# Resolve data directory
if [[ -z "$DATA_DIR" ]]; then
    DATA_FILE="$PROJECT_ROOT/private/data-path.txt"
    if [[ -f "$DATA_FILE" ]]; then
        DATA_DIR=$(cat "$DATA_FILE" | tr -d '[:space:]')
    fi
fi

if [[ -n "$DATA_DIR" ]]; then
    DATA_DIR=$(realpath "$DATA_DIR")
fi

CHAIN_DIR="$PROJECT_ROOT/private/campaign-chain"
mkdir -p "$CHAIN_DIR"

# Clean old checkpoints (this run's output only)
find "$CHAIN_DIR" -maxdepth 1 -name '[0-9][0-9]-*.json' -delete 2>/dev/null || true

# Compute SHA256 of executable
SHA256_SUM=$(sha256sum "$EXECUTABLE" | cut -d' ' -f1)

# Initialize summary JSON
SUMMARY_FILE="$CHAIN_DIR/run-summary.json"
cat > "$SUMMARY_FILE" <<EOF
{
  "started_utc": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "executable": "$EXECUTABLE",
  "sha256": "$SHA256_SUM",
  "data": "$DATA_DIR",
  "difficulty": "$DIFFICULTY",
  "inside_anode_seat": $INSIDE_ANODE_SEAT,
  "status": "running",
  "headless": [],
  "native": [],
  "native_skipped": null
}
EOF

update_summary() {
    if command -v jq &> /dev/null; then
        local tmp=$(mktemp)
        cat "$SUMMARY_FILE" | jq "$1" > "$tmp" && mv "$tmp" "$SUMMARY_FILE"
    fi
}

add_step() {
    local name="$1"
    local args="$2"
    local is_native="$3"
    local work_dir="$4"
    local exit_code="$5"
    local log_file="$6"

    local entry=$(jq -n \
        --arg name "$name" \
        --arg args "$args" \
        --argjson code "$exit_code" \
        --arg log "$log_file" \
        --arg work_dir "$work_dir" \
        '{name: $name, args: $args, exit_code: $code, log: $log, work_dir: (if $work_dir == "" then null else $work_dir end)}')

    if [[ "$is_native" == "true" ]]; then
        update_summary ".native += [$entry]"
    else
        update_summary ".headless += [$entry]"
    fi
}

run_step() {
    local name="$1"
    shift
    local step_args=("$@")
    local is_native="${2:-false}"
    local work_dir="${3:-}"

    local log_file="$CHAIN_DIR/$name.log"
    local all_args=("${step_args[@]}")

    if [[ -n "$DATA_DIR" ]]; then
        all_args+=("--data" "$DATA_DIR")
    fi

    echo "Running $name: ${all_args[*]}"

    local exit_code=0
    if [[ -n "$work_dir" ]]; then
        mkdir -p "$work_dir"
        pushd "$work_dir" > /dev/null
    fi

    if [[ "$VERBOSE" == true ]]; then
        "$EXECUTABLE" "${all_args[@]}" 2>&1 | tee "$log_file"
        exit_code=${PIPESTATUS[0]}
    else
        "$EXECUTABLE" "${all_args[@]}" > "$log_file" 2>&1
        exit_code=$?
    fi

    if [[ -n "$work_dir" ]]; then
        popd > /dev/null
        local actual_log="$work_dir/private/campaign-chain/$name.log"
        if [[ -f "$actual_log" ]]; then
            log_file="$actual_log"
        fi
    fi

    # Verify executable hasn't changed
    CURRENT_SHA256=$(sha256sum "$EXECUTABLE" | cut -d' ' -f1)
    if [[ "$CURRENT_SHA256" != "$SHA256_SUM" ]]; then
        echo "Error: Executable changed during test run!"
        exit 1
    fi

    add_step "$name" "${all_args[*]}" "$is_native" "$work_dir" "$exit_code" "$log_file"

    if [[ $exit_code -eq 0 ]]; then
        echo "PASS $name (exit $exit_code)"
    else
        echo "FAIL $name (exit $exit_code)"
        echo "See log: $log_file"
    fi

    return $exit_code
}

echo "=== Campaign Chain Test Suite ==="
echo "Executable: $EXECUTABLE"
echo "SHA256: $SHA256_SUM"
echo "Data dir: ${DATA_DIR:-auto}"
echo "Difficulty: $DIFFICULTY"
echo "Inside Anode Seat: $INSIDE_ANODE_SEAT"
echo "Pinned: $PINNED"
echo "Determinism: $DETERMINISM"
echo

FAILED_STEPS=()

# Graph check
CHAIN_ARGS=("--campaign-graph-check")
if ! run_step "graph-check" "${CHAIN_ARGS[@]}" false; then
    FAILED_STEPS+=("graph-check")
fi

# Route check
CHAIN_ARGS=("--campaign-route-check" "--campaign-strict" "--difficulty" "$DIFFICULTY" "--campaign-log" "$CHAIN_DIR/route-check.log")
if [[ -n "$TO" ]]; then
    CHAIN_ARGS+=("--campaign-to" "$TO")
fi
if [[ -n "$EXPECT" ]]; then
    CHAIN_ARGS+=("--campaign-expect" "$EXPECT")
fi
if ! run_step "route-check" "${CHAIN_ARGS[@]}" false; then
    FAILED_STEPS+=("route-check")
fi
cp -f "$CHAIN_DIR/report.json" "$CHAIN_DIR/report-$DIFFICULTY.json" 2>/dev/null || true

# Route check with skip cinematics
SKIP_ARGS=("--campaign-route-check" "--campaign-strict" "--campaign-skip-cinematics" "--difficulty" "$DIFFICULTY" "--campaign-log" "$CHAIN_DIR/route-check-skip.log")
if [[ -n "$TO" ]]; then
    SKIP_ARGS+=("--campaign-to" "$TO")
fi
if [[ -n "$EXPECT" ]]; then
    SKIP_ARGS+=("--campaign-expect" "$EXPECT")
fi
if ! run_step "route-check-skip" "${SKIP_ARGS[@]}" false; then
    FAILED_STEPS+=("route-check-skip")
fi
cp -f "$CHAIN_DIR/report.json" "$CHAIN_DIR/report-$DIFFICULTY-skip.json" 2>/dev/null || true

# Determinism check
if [[ "$DETERMINISM" == true ]]; then
    cp -f "$CHAIN_DIR/report-$DIFFICULTY.json" "$CHAIN_DIR/report-first.json"

    AGAIN_ARGS=("--campaign-route-check" "--campaign-strict" "--difficulty" "$DIFFICULTY" "--campaign-log" "$CHAIN_DIR/route-check-repeat.log")
    if [[ -n "$TO" ]]; then
        AGAIN_ARGS+=("--campaign-to" "$TO")
    fi
    if [[ -n "$EXPECT" ]]; then
        AGAIN_ARGS+=("--campaign-expect" "$EXPECT")
    fi
    if ! run_step "route-check-repeat" "${AGAIN_ARGS[@]}" false; then
        FAILED_STEPS+=("route-check-repeat")
    fi

    # Compare digests
    if [[ -f "$CHAIN_DIR/report-first.json" && -f "$CHAIN_DIR/report.json" ]]; then
        FIRST_DIGESTS=$(jq -r '.legs | map("\(.index):\(.entrance_digest):\(.exit_digest)") | join(";")' "$CHAIN_DIR/report-first.json" 2>/dev/null || echo "")
        SECOND_DIGESTS=$(jq -r '.legs | map("\(.index):\(.entrance_digest):\(.exit_digest)") | join(";")' "$CHAIN_DIR/report.json" 2>/dev/null || echo "")
        LEGS_COUNT=$(jq '.legs | length' "$CHAIN_DIR/report-first.json" 2>/dev/null || echo "0")
        EQUAL=false
        if [[ "$FIRST_DIGESTS" == "$SECOND_DIGESTS" ]]; then
            EQUAL=true
        fi

        update_summary ".determinism = {equal: $EQUAL, legs: $LEGS_COUNT}"

        if [[ "$EQUAL" == true ]]; then
            echo "PASS determinism: $LEGS_COUNT legs, digests equal"
        else
            echo "FAIL determinism: $LEGS_COUNT legs, digests differ"
            FAILED_STEPS+=("determinism")
        fi
    fi
fi

# Pinned tests
if [[ "$PINNED" == true ]]; then
    # Gates at visit 4 and 6
    GATES=("4:fortress2\$fortress2_start1" "6:skool1\$skool1_start1")

    for gate in "${GATES[@]}"; do
        IFS=':' read -r n to <<< "$gate"
        GATE_ARGS=("--campaign-route-check" "--campaign-strict" "--campaign-to" "$to" "--campaign-log" "$CHAIN_DIR/gate-visit-$n.log")
        if ! run_step "gate-visit-$n" "${GATE_ARGS[@]}" false; then
            FAILED_STEPS+=("gate-visit-$n")
        fi
        cp -f "$CHAIN_DIR/report.json" "$CHAIN_DIR/report-gate-visit-$n.json" 2>/dev/null || true
    done

    # Gates at Easy and Hard in scratch directories
    for gate in "${GATES[@]}"; do
        IFS=':' read -r n to <<< "$gate"
        for other in easy hard; do
            SCRATCH_GATE="$CHAIN_DIR/scratch-gate-$n-$other"
            rm -rf "$SCRATCH_GATE"
            GATE_OTHER_ARGS=("--campaign-route-check" "--campaign-strict" "--difficulty" "$other" "--campaign-to" "$to" "--campaign-log" "$CHAIN_DIR/gate-visit-$n-$other.log")
            if ! run_step "gate-visit-$n-$other" "${GATE_OTHER_ARGS[@]}" false "$SCRATCH_GATE"; then
                FAILED_STEPS+=("gate-visit-$n-$other")
            fi
            cp -f "$SCRATCH_GATE/private/campaign-chain/report.json" "$CHAIN_DIR/report-gate-visit-$n-$other.json" 2>/dev/null || true
        done
    done

    # Pinned failures
    FRONTIER7="skool2\$skool2_start1"
    RETRY4="skool2\$skool2_start1"

    ONWARD_ARGS=("--campaign-route-check" "--campaign-strict" "--campaign-from" "4" "--campaign-expect" "frontier=$FRONTIER7" "--campaign-log" "$CHAIN_DIR/pinned-frontier-from-4.log")
    if ! run_step "pinned-frontier-from-4" "${ONWARD_ARGS[@]}" false; then
        FAILED_STEPS+=("pinned-frontier-from-4")
    fi
    cp -f "$CHAIN_DIR/report.json" "$CHAIN_DIR/report-pinned-frontier-from-4.json" 2>/dev/null || true

    # Retry from 4 in scratch directory
    SCRATCH_RETRY="$CHAIN_DIR/scratch-retry"
    rm -rf "$SCRATCH_RETRY"
    CHECKPOINTS="$SCRATCH_RETRY/private/campaign-chain"
    mkdir -p "$CHECKPOINTS"
    cp -f "$CHAIN_DIR/04-fortress2-first.json" "$CHECKPOINTS/" 2>/dev/null || true

    RETRY_ARGS=("--campaign-route-check" "--campaign-strict" "--campaign-from" "4" "--campaign-allow-retry" "--campaign-expect" "frontier=$RETRY4" "--campaign-log" "$CHAIN_DIR/pinned-retry-from-4.log")
    if ! run_step "pinned-retry-from-4" "${RETRY_ARGS[@]}" false "$SCRATCH_RETRY"; then
        FAILED_STEPS+=("pinned-retry-from-4")
    fi

    # No checkpoint refusal from visit 6
    EMPTY_DIR="$CHAIN_DIR/scratch-refusal"
    rm -rf "$EMPTY_DIR"
    REFUSE_ARGS=("--campaign-route-check" "--campaign-strict" "--campaign-from" "6" "--campaign-expect" "no-checkpoint" "--campaign-log" "$CHAIN_DIR/pinned-no-checkpoint.log")
    if ! run_step "pinned-no-checkpoint" "${REFUSE_ARGS[@]}" false "$EMPTY_DIR"; then
        FAILED_STEPS+=("pinned-no-checkpoint")
    fi
fi

# Native save chain (requires window)
if [[ "$INSIDE_ANODE_SEAT" == true ]]; then
    if ! run_step "save-chain-write" "--campaign-save-chain-write" "--no-audio" true; then
        FAILED_STEPS+=("save-chain-write")
    fi
    if ! run_step "save-chain-read" "--campaign-save-chain-read" "--no-audio" true; then
        FAILED_STEPS+=("save-chain-read")
    fi
else
    update_summary '.native_skipped = "not invoked inside the Anode seat (--inside-anode-seat): --campaign-save-chain-write/read open a window and were not run"'
fi

# Final status
if [[ ${#FAILED_STEPS[@]} -eq 0 ]]; then
    update_summary '.status = "passed"'
else
    update_summary '.status = "failed" | .failed = $ARGS.positional' --args "${FAILED_STEPS[@]}"
fi

echo
echo "=== Summary ==="
if [[ ${#FAILED_STEPS[@]} -eq 0 ]]; then
    echo "Campaign chain steps passed. Summary: $SUMMARY_FILE"
    exit 0
else
    echo "Campaign chain steps failed: ${FAILED_STEPS[*]}. See $SUMMARY_FILE"
    exit 1
fi