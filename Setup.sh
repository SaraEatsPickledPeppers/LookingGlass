#!/bin/bash
# Setup.sh - Linux entry point for Looking Glass guided setup
# Delegates to tools/linux_setup.sh for core logic
# Uses zenity/kdialog/yad for GUI dialogs, fallback to CLI

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SETUP_CORE="${SCRIPT_DIR}/tools/linux_setup.sh"

# Default values
LANGUAGE=""
DATA_PATH=""
ACTION="setup"
DOWNLOAD=false
VERBOSE=false
DRY_RUN=false
GUI_TOOL=""
LOG_FILE=""

# Exit codes
EXIT_SUCCESS=0
EXIT_ERROR=1
EXIT_CANCELLED=2

# Parse command line arguments
usage() {
    cat <<EOF
Usage: Setup.sh [OPTIONS]

Looking Glass - Local Game Data Setup

Options:
  --language <code>   Set language (en, de, fr, es, nl)
  --data <path>       Path to game data folder or archive
  --download          Download game data from Internet Archive
  --help              Show this help message
  --verbose, -v       Enable debug logging
  --dry-run           Show what would be done without writing files

Exit codes:
  0  Success
  1  Error
  2  User cancelled

Examples:
  Setup.sh                    # Interactive setup
  Setup.sh --language de      # Setup with German language
  Setup.sh --data /path/to/Alice1 --dry-run
  Setup.sh --download         # Download and setup
EOF
}

# Detect available GUI dialog tool
detect_gui_tool() {
    if command -v zenity >/dev/null 2>&1; then
        echo "zenity"
        return
    fi
    if command -v kdialog >/dev/null 2>&1; then
        echo "kdialog"
        return
    fi
    if command -v yad >/dev/null 2>&1; then
        echo "yad"
        return
    fi
    echo "cli"
}

# Setup logging
setup_logging() {
    if [[ "$VERBOSE" == true ]]; then
        local log_dir="${HOME}/.local/share/LookingGlass/logs"
        mkdir -p "$log_dir"
        LOG_FILE="${log_dir}/setup-$(date +%Y%m%d-%H%M%S).log"
        exec 3>&1 4>&2
        exec 1> >(tee -a "$LOG_FILE") 2> >(tee -a "$LOG_FILE" >&2)
        echo "[DEBUG] Logging to $LOG_FILE" >&2
    fi
}

# Main argument parsing
while [[ $# -gt 0 ]]; do
    case "$1" in
        --language)
            LANGUAGE="$2"
            shift 2
            ;;
        --data)
            DATA_PATH="$2"
            shift 2
            ;;
        --download)
            DOWNLOAD=true
            ACTION="download"
            shift
            ;;
        --help|-h)
            usage
            exit $EXIT_SUCCESS
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit $EXIT_ERROR
            ;;
    esac
done

# Setup logging early if verbose
setup_logging

# Validate language code
if [[ -n "$LANGUAGE" ]]; then
    case "$LANGUAGE" in
        en|de|fr|es|nl) ;;
        *)
            echo "Error: Invalid language code '$LANGUAGE'. Valid codes: en, de, fr, es, nl" >&2
            exit $EXIT_ERROR
            ;;
    esac
fi

# Detect GUI tool
GUI_TOOL=$(detect_gui_tool)
if [[ "$VERBOSE" == true ]]; then
    echo "[DEBUG] GUI tool detected: $GUI_TOOL" >&2
fi

# Check if core setup script exists
if [[ ! -f "$SETUP_CORE" ]]; then
    echo "Error: Core setup script not found at $SETUP_CORE" >&2
    exit $EXIT_ERROR
fi

# Build arguments for core script
CORE_ARGS=()
[[ -n "$LANGUAGE" ]] && CORE_ARGS+=(--language "$LANGUAGE")
[[ -n "$DATA_PATH" ]] && CORE_ARGS+=(--data "$DATA_PATH")
[[ "$DOWNLOAD" == true ]] && CORE_ARGS+=(--download)
[[ "$VERBOSE" == true ]] && CORE_ARGS+=(--verbose)
[[ "$DRY_RUN" == true ]] && CORE_ARGS+=(--dry-run)
CORE_ARGS+=(--gui "$GUI_TOOL")

# Export variables for core script
export LG_SETUP_LANGUAGE="${LANGUAGE}"
export LG_SETUP_SOURCE="${DATA_PATH}"
export LG_SETUP_ACTION="${ACTION}"
export LG_SETUP_GUI="${GUI_TOOL}"
export LG_SETUP_VERBOSE="${VERBOSE}"
export LG_SETUP_DRY_RUN="${DRY_RUN}"
export LG_SETUP_ROOT="${SCRIPT_DIR}"

# Execute core setup script
if [[ "$VERBOSE" == true ]]; then
    echo "[DEBUG] Executing: $SETUP_CORE ${CORE_ARGS[*]}" >&2
fi

"$SETUP_CORE" "${CORE_ARGS[@]}"
exit_code=$?

# Handle exit codes
case $exit_code in
    0)
        [[ "$VERBOSE" == true ]] && echo "[DEBUG] Setup completed successfully" >&2
        exit $EXIT_SUCCESS
        ;;
    2)
        [[ "$VERBOSE" == true ]] && echo "[DEBUG] Setup cancelled by user" >&2
        exit $EXIT_CANCELLED
        ;;
    *)
        [[ "$VERBOSE" == true ]] && echo "[DEBUG] Setup failed with exit code $exit_code" >&2
        exit $EXIT_ERROR
        ;;
esac