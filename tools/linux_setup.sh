#!/bin/bash

# find_game_data - Locate Alice game data directory containing required .pk3 files
# Searches predefined locations and returns the first one containing all required pak files.

VERBOSE=0

# Required pak files that must all exist in the game data directory
REQUIRED_PAKS=(
    "pak0.pk3"
    "pak1_large.pk3"
    "pak2.pk3"
    "pak3.pk3"
    "pak5_mod.pk3"
)

# Log verbose message to stderr
log_verbose() {
    if [[ $VERBOSE -eq 1 ]]; then
        echo "[VERBOSE] $*" >&2
    fi
}

# Check if a directory contains all required pak files
check_pak_files() {
    local dir="$1"
    local missing=()

    for pak in "${REQUIRED_PAKS[@]}"; do
        if [[ ! -f "$dir/$pak" ]]; then
            missing+=("$pak")
        fi
    done

    if [[ ${#missing[@]} -eq 0 ]]; then
        log_verbose "Found all required pak files in: $dir"
        return 0
    else
        log_verbose "Missing pak files in $dir: ${missing[*]}"
        return 1
    fi
}

# Language packs that are considered valid (at least one required)
LANGUAGE_PAKS=(
    "pak4_english.pk3"
    "pak4_german.pk3"
    "pak4_french.pk3"
    "pak4_spanish.pk3"
    "pak4_dutch.pk3"
)

# Validate base directory contains all required game files
validate_base() {
    local base_dir="$1"
    local require_language="${2:-true}"

    # Check required pak files
    if ! check_pak_files "$base_dir"; then
        echo "Missing required pak files in $base_dir" >&2
        return 1
    fi

    # Check for at least one language pack
    if [[ "$require_language" == "true" ]]; then
        local found_language=0
        for lang_pak in "${LANGUAGE_PAKS[@]}"; do
            if [[ -f "$base_dir/$lang_pak" ]]; then
                log_verbose "Found language pack: $lang_pak"
                found_language=1
                break
            fi
        done
        if [[ $found_language -eq 0 ]]; then
            echo "No valid language pack found in $base_dir (need one of: ${LANGUAGE_PAKS[*]})" >&2
            return 1
        fi
    fi

    # Check for 36 map entries (maps/*.bsp) across all pk3 files
    local map_count=0
    for pak in "${REQUIRED_PAKS[@]}"; do
        if [[ -f "$base_dir/$pak" ]]; then
            local count
            count=$(unzip -l "$base_dir/$pak" 2>/dev/null | grep -c 'maps/.*\.bsp$' || true)
            map_count=$((map_count + count))
        fi
    done
    # Also check language packs for maps
    for lang_pak in "${LANGUAGE_PAKS[@]}"; do
        if [[ -f "$base_dir/$lang_pak" ]]; then
            local count
            count=$(unzip -l "$base_dir/$lang_pak" 2>/dev/null | grep -c 'maps/.*\.bsp$' || true)
            map_count=$((map_count + count))
        fi
    done

    if [[ $map_count -ne 36 ]]; then
        echo "Expected 36 map entries (*.bsp), found $map_count in $base_dir" >&2
        return 1
    fi
    log_verbose "Found $map_count map entries"

    # Check village map (maps/gvillage.bsp) for FAKK version 42
    local village_found=0
    for pak in "${REQUIRED_PAKS[@]}" "${LANGUAGE_PAKS[@]}"; do
        if [[ -f "$base_dir/$pak" ]]; then
            # Check if gvillage.bsp exists in this pak
            if unzip -l "$base_dir/$pak" 2>/dev/null | grep -q 'maps/gvillage\.bsp$'; then
                village_found=1
                # Extract and check the first 8 bytes for FAKK signature + version 42
                local temp_file
                temp_file=$(mktemp)
                if unzip -p "$base_dir/$pak" "maps/gvillage.bsp" > "$temp_file" 2>/dev/null; then
                    # Read first 8 bytes: "FAKK" (4 bytes) + version (4 bytes little-endian)
                    local header
                    header=$(head -c 8 "$temp_file" | od -An -tx1 | tr -d ' \n')
                    # Check: first 4 bytes = FAKK (46 41 4b 4b), next 4 bytes = 42 (2a 00 00 00)
                    if [[ "$header" == "46414b4b2a000000" ]]; then
                        log_verbose "Village map (gvillage.bsp) has correct FAKK version 42"
                        rm -f "$temp_file"
                        break
                    else
                        echo "Village map (gvillage.bsp) has invalid signature/version in $pak (got: $header)" >&2
                        rm -f "$temp_file"
                        return 1
                    fi
                fi
                rm -f "$temp_file"
            fi
        fi
    done

    if [[ $village_found -eq 0 ]]; then
        echo "Village map (maps/gvillage.bsp) not found in any pak file" >&2
        return 1
    fi

    log_verbose "Base validation successful for: $base_dir"
    return 0
}

# Search for game data directory
find_game_data() {
    local search_paths=(
        "./alice_202106/Alice1/bin/base"
        "./Alice1/bin/base"
        "./bin/base"
        "$HOME/Downloads/alice_202106/Alice1/bin/base"
        "${XDG_DATA_HOME:-$HOME/.local/share}/LookingGlass/game-data"
    )

    log_verbose "Searching for game data in ${#search_paths[@]} locations"

    for path in "${search_paths[@]}"; do
        # Expand tilde and variables
        local expanded_path="${path/#\~/$HOME}"
        expanded_path="${expanded_path/\$HOME/$HOME}"
        expanded_path="${expanded_path/\$XDG_DATA_HOME/${XDG_DATA_HOME:-$HOME/.local/share}}"

        log_verbose "Checking: $expanded_path"

        if [[ -d "$expanded_path" ]]; then
            log_verbose "Directory exists: $expanded_path"
            if check_pak_files "$expanded_path"; then
                echo "$expanded_path"
                return 0
            fi
        else
            log_verbose "Directory does not exist: $expanded_path"
        fi
    done

    log_verbose "No valid game data directory found"
    return 1
}

# Get list of entries in an archive (.7z or .zip)
# Usage: get_archive_entries <archive_path>
# Prints entries (one per line) to stdout, returns 0 on success
# Normalizes path separators to forward slashes
get_archive_entries() {
    local archive_path="$1"

    if [[ ! -f "$archive_path" ]]; then
        echo "Error: Archive not found: $archive_path" >&2
        return 1
    fi

    local ext="${archive_path##*.}"
    ext="${ext,,}"  # lowercase

    case "$ext" in
        zip)
            # Try unzip first, fall back to 7z
            if command -v unzip >/dev/null 2>&1; then
                unzip -Z1 "$archive_path" 2>/dev/null | sed 's|\\|/|g'
                return ${PIPESTATUS[0]}
            else
                local extractor
                extractor=$(find_extractor) || return 1
                "$extractor" l -ba -slt "$archive_path" 2>/dev/null | \
                    awk -F' = ' '/^Path = / {gsub(/\\/, "/", $2); print $2}'
                return ${PIPESTATUS[0]}
            fi
            ;;
        7z)
            local extractor
            extractor=$(find_extractor) || return 1
            "$extractor" l -ba -slt "$archive_path" 2>/dev/null | \
                awk -F' = ' '/^Path = / {gsub(/\\/, "/", $2); print $2}'
            return ${PIPESTATUS[0]}
            ;;
        *)
            echo "Error: Unsupported archive format: .$ext" >&2
            return 1
            ;;
    esac
}

# Check for cancellation via LG_SETUP_CANCEL env var file
check_cancel() {
    if [[ -n "$LG_SETUP_CANCEL" && -f "$LG_SETUP_CANCEL" ]]; then
        echo "Operation cancelled" >&2
        return 1
    fi
    return 0
}

# Compute SHA256 hash of a file
# Usage: compute_sha256 <file_path>
# Prints SHA256 hash (lowercase hex) to stdout, returns 0 on success
compute_sha256() {
    local file_path="$1"

    if [[ ! -f "$file_path" ]]; then
        echo "Error: File not found: $file_path" >&2
        return 1
    fi

    # sha256sum reads files in chunks efficiently, handles large files automatically
    # Output format: "hash  filename", we extract just the hash
    sha256sum "$file_path" | awk '{print $1}'
    return ${PIPESTATUS[0]}
}

# Download game data archive from Internet Archive with integrity verification
# Usage: download_game_data <download_dir>
# Prints path to downloaded archive on stdout, returns 0 on success
download_game_data() {
    local download_dir="$1"

    # Configuration
    local url="https://archive.org/download/alice_202106/Alice1_2011_vanilla.7z"
    local expected_size=977811158
    local expected_sha256="b253bb2c9c875f1838a711d0fe4f12f4bc2c6207db3baae6431af962eca8705d"
    local filename="Alice1_2011_vanilla.7z"
    local output_path="${download_dir}/${filename}"
    local partial_path="${output_path}.partial"

    # Ensure download directory exists
    mkdir -p "$download_dir" || {
        echo "Failed to create download directory: $download_dir" >&2
        return 1
    }

    # Check if already downloaded and valid
    if [[ -f "$output_path" ]]; then
        log_verbose "Found existing file: $output_path"
        local actual_size
        actual_size=$(stat -c%s "$output_path" 2>/dev/null || stat -f%z "$output_path" 2>/dev/null)
        if [[ "$actual_size" -eq "$expected_size" ]]; then
            local actual_sha256
            actual_sha256=$(sha256sum "$output_path" 2>/dev/null | awk '{print $1}')
            if [[ "$actual_sha256" == "$expected_sha256" ]]; then
                log_verbose "File already exists and matches expected size and SHA256"
                echo "$output_path"
                return 0
            else
                log_verbose "Existing file has incorrect SHA256 (expected: $expected_sha256, got: $actual_sha256)"
            fi
        else
            log_verbose "Existing file has incorrect size (expected: $expected_size, got: $actual_size)"
        fi
        # Remove invalid file and partial file to start fresh
        rm -f "$output_path" "$partial_path"
    fi

    # Determine resume offset if partial file exists
    local resume_offset=0
    if [[ -f "$partial_path" ]]; then
        resume_offset=$(stat -c%s "$partial_path" 2>/dev/null || stat -f%z "$partial_path" 2>/dev/null)
        log_verbose "Resuming download from offset: $resume_offset"
    fi

    # Download with curl (supports resume and progress)
    # Use --continue-at for resume, --progress-bar for progress to stderr
    local curl_args=(
        "--fail"
        "--location"
        "--continue-at" "-"
        "--output" "$partial_path"
        "--user-agent" "LookingGlass/1.0"
        "--connect-timeout" "30"
        "--max-time" "3600"
        "$url"
    )

    log_verbose "Downloading from: $url"
    log_verbose "Saving to: $partial_path"

    # Track progress - we'll use curl's built-in progress bar to stderr
    # and also report bytes downloaded periodically
    local last_reported=0
    local progress_interval=1048576  # Report every 1MB

    if ! curl "${curl_args[@]}" 2>&1; then
        local curl_exit=$?
        if [[ $curl_exit -eq 28 ]]; then
            echo "Download timed out" >&2
        elif [[ $curl_exit -eq 18 ]]; then
            echo "Partial download (file may be incomplete)" >&2
        else
            echo "Download failed with exit code: $curl_exit" >&2
        fi
        check_cancel || return 1
        return 1
    fi

    check_cancel || return 1

    # Verify file size
    local actual_size
    actual_size=$(stat -c%s "$partial_path" 2>/dev/null || stat -f%z "$partial_path" 2>/dev/null)

    if [[ "$actual_size" -ne "$expected_size" ]]; then
        echo "Size mismatch: expected $expected_size bytes, got $actual_size bytes" >&2
        return 1
    fi

    log_verbose "Download complete: $actual_size bytes"

    # Verify SHA256
    local actual_sha256
    actual_sha256=$(sha256sum "$partial_path" 2>/dev/null | awk '{print $1}')

    if [[ "$actual_sha256" != "$expected_sha256" ]]; then
        echo "SHA256 mismatch: expected $expected_sha256, got $actual_sha256" >&2
        return 1
    fi

    log_verbose "SHA256 verified: $actual_sha256"

    # Atomically move partial to final location
    mv "$partial_path" "$output_path" || {
        echo "Failed to move downloaded file to final location" >&2
        return 1
    }

    echo "$output_path"
    return 0
}

# Extract base pak files from archive
# Usage: extract_base_paks <archive_path> <target_dir> <prefix>
# prefix: e.g., "base/", "bin/base/", "Alice1/bin/base/", "alice_202106/Alice1/bin/base/"
extract_base_paks() {
    local archive_path="$1"
    local target_dir="$2"
    local prefix="$3"

    check_cancel || return 1

    # Ensure target directory exists
    mkdir -p "$target_dir" || {
        echo "Failed to create target directory: $target_dir" >&2
        return 1
    }

    # Get archive entries
    local entries
    entries=$(get_archive_entries "$archive_path") || {
        echo "Failed to read archive entries: $archive_path" >&2
        return 1
    }

    # Known prefixes to search for pak0.pk3
    local known_prefixes=(
        "base/"
        "bin/base/"
        "Alice1/bin/base/"
        "alice_202106/Alice1/bin/base/"
    )

    # Find the correct prefix that contains pak0.pk3
    local found_prefix=""
    for p in "${known_prefixes[@]}"; do
        if echo "$entries" | grep -q "^${p}pak0\.pk3$"; then
            found_prefix="$p"
            break
        fi
    done

    if [[ -z "$found_prefix" ]]; then
        echo "Could not find pak0.pk3 in archive with known prefixes" >&2
        return 1
    fi

    # Build list of wanted files
    # 5 required PK3s
    local wanted_files=(
        "${found_prefix}pak0.pk3"
        "${found_prefix}pak1_large.pk3"
        "${found_prefix}pak2.pk3"
        "${found_prefix}pak3.pk3"
        "${found_prefix}pak5_mod.pk3"
    )

    # 5 language pack files
    local lang_paks=(
        "${found_prefix}pak4_english.pk3"
        "${found_prefix}pak4_german.pk3"
        "${found_prefix}pak4_french.pk3"
        "${found_prefix}pak4_spanish.pk3"
        "${found_prefix}pak4_dutch.pk3"
    )
    wanted_files+=("${lang_paks[@]}")

    # 4 language bundle files (if present in archive)
    local lang_bundles=(
        "${found_prefix}language_english.pk3"
        "${found_prefix}language_german.pk3"
        "${found_prefix}language_french.pk3"
        "${found_prefix}language_spanish.pk3"
    )
    wanted_files+=("${lang_bundles[@]}")

    # Filter to only files that exist in the archive
    local to_extract=()
    for file in "${wanted_files[@]}"; do
        if echo "$entries" | grep -q "^${file}$"; then
            to_extract+=("$file")
        fi
    done

    if [[ ${#to_extract[@]} -eq 0 ]]; then
        echo "No matching files found in archive" >&2
        return 1
    fi

    log_verbose "Extracting ${#to_extract[@]} files from archive: $archive_path"

    check_cancel || return 1

    local ext="${archive_path##*.}"

    if [[ "$ext" == "zip" ]]; then
        # Extract with unzip -j (junk paths, extract flat)
        local unzip_args=("-j" "-q" "-o" "$archive_path" "-d" "$target_dir")
        for file in "${to_extract[@]}"; do
            unzip_args+=("$file")
        done
        if ! unzip "${unzip_args[@]}" 2>/dev/null; then
            echo "Failed to extract files from zip archive" >&2
            return 1
        fi
    elif [[ "$ext" == "7z" ]]; then
        local extractor
        extractor=$(find_extractor) || return 1
        # Extract with 7z e (flat extraction)
        local sevenz_args=("e" "-y" "-bso0" "-bsp0" "-o$target_dir" "$archive_path")
        for file in "${to_extract[@]}"; do
            sevenz_args+=("$file")
        done
        if ! "$extractor" "${sevenz_args[@]}" 2>/dev/null; then
            echo "Failed to extract files from 7z archive" >&2
            return 1
        fi
    else
        echo "Unsupported archive format: $ext" >&2
        return 1
    fi

    check_cancel || return 1

    log_verbose "Extracted ${#to_extract[@]} files to: $target_dir"
    return 0
}

# Find 7z extraction command
# Checks in order: 7z, 7zz, 7zr, then common install paths
# Prints the command name to stdout, returns 0 if found, 1 if not found
find_extractor() {
    local extractor_cmds=("7z" "7zz" "7zr")
    local search_paths=("/usr/bin" "/usr/local/bin" "/opt/homebrew/bin" "/bin" "/sbin" "/usr/sbin")

    # First check if commands are in PATH
    for cmd in "${extractor_cmds[@]}"; do
        if command -v "$cmd" >/dev/null 2>&1; then
            log_verbose "Found extractor in PATH: $cmd"
            echo "$cmd"
            return 0
        fi
    done

    # Then check common install paths directly
    for path in "${search_paths[@]}"; do
        for cmd in "${extractor_cmds[@]}"; do
            if [[ -x "$path/$cmd" ]]; then
                log_verbose "Found extractor at: $path/$cmd"
                echo "$cmd"
                return 0
            fi
        done
    done

    # Not found
    echo "To read .7z, install 7-Zip or extract the archive and choose its folder." >&2
    return 1
}

# Write config file atomically (temp file + rename)
# Usage: write_config <config_path> <data_path>
write_config() {
    local config_path="$1"
    local data_path="$2"

    # Create parent directory if it doesn't exist
    local parent_dir
    parent_dir=$(dirname "$config_path")
    if [[ ! -d "$parent_dir" ]]; then
        mkdir -p "$parent_dir"
    fi

    # Write to temp file atomically
    local temp_file
    temp_file=$(mktemp "${config_path}.tmp.XXXXXX")
    printf '%s\n' "$data_path" > "$temp_file"
    mv "$temp_file" "$config_path"
}

# Write receipt file atomically (temp file + rename)
# Usage: write_receipt <receipt_path> <source> <archive_sha256> <data_path>
write_receipt() {
    local receipt_path="$1"
    local source="$2"
    local archive_sha256="$3"
    local data_path="$4"

    # Create parent directory if it doesn't exist
    local parent_dir
    parent_dir=$(dirname "$receipt_path")
    if [[ ! -d "$parent_dir" ]]; then
        mkdir -p "$parent_dir"
    fi

    # Write to temp file atomically
    local temp_file
    temp_file=$(mktemp "${receipt_path}.tmp.XXXXXX")

    # Use jq if available, otherwise construct JSON manually
    if command -v jq >/dev/null 2>&1; then
        jq -n \
            --arg schema "1" \
            --arg source "$source" \
            --arg archiveSha256 "$archive_sha256" \
            --arg dataPath "$data_path" \
            '{schema: ($schema | tonumber), source: $source, archiveSha256: $archiveSha256, dataPath: $dataPath}' \
            > "$temp_file"
    else
        # Manual JSON construction (properly escaped)
        local escaped_source
        local escaped_sha256
        local escaped_data_path
        # Escape backslashes, quotes, and newlines
        escaped_source=$(printf '%s' "$source" | sed 's/\\/\\\\/g; s/"/\\"/g')
        escaped_sha256=$(printf '%s' "$archive_sha256" | sed 's/\\/\\\\/g; s/"/\\"/g')
        escaped_data_path=$(printf '%s' "$data_path" | sed 's/\\/\\\\/g; s/"/\\"/g')
        printf '{"schema":1,"source":"%s","archiveSha256":"%s","dataPath":"%s"}\n' \
            "$escaped_source" "$escaped_sha256" "$escaped_data_path" \
            > "$temp_file"
    fi

    mv "$temp_file" "$receipt_path"
}

# Language pack files for translated languages (de, fr, es, nl)
# These are the base_*.pk3 files that accompany the translated language packs
LANGUAGE_BUNDLE_PAKS=(
    "base_g.pk3"   # German
    "base_f.pk3"   # French
    "base_s.pk3"   # Spanish
    "base_d.pk3"   # Dutch
)

# Language-specific words for validation
# English: checks dialog/standard.tlk for "step" (case-insensitive, whole word)
# Dutch: checks ui/main.urc for "Terug" (case-sensitive)
# Others: check dialog/standard.tlk for language-specific word
LANGUAGE_WORDS=(
    "en:step"       # English - dialog/standard.tlk, case-insensitive whole word
    "de:Schritt"    # German - dialog/standard.tlk
    "fr:étape"      # French - dialog/standard.tlk
    "es:paso"       # Spanish - dialog/standard.tlk
    "nl:Terug"      # Dutch - ui/main.urc, case-sensitive
)

# test_language - Validates a language pack contains required content
# Usage: test_language <lang_code> <pack_path>
# Returns 0 if valid, 1 if invalid
test_language() {
    local lang_code="$1"
    local pack_path="$2"

    if [[ ! -f "$pack_path" ]]; then
        log_verbose "Language pack not found: $pack_path"
        return 1
    fi

    case "$lang_code" in
        en)
            # English: check dialog/standard.tlk for "step" (case-insensitive, whole word)
            if unzip -p "$pack_path" "dialog/standard.tlk" 2>/dev/null | grep -qiw "step"; then
                log_verbose "English language pack validated: $pack_path"
                return 0
            fi
            ;;
        nl)
            # Dutch: check ui/main.urc for "Terug" (case-sensitive)
            if unzip -p "$pack_path" "ui/main.urc" 2>/dev/null | grep -q "Terug"; then
                log_verbose "Dutch language pack validated: $pack_path"
                return 0
            fi
            ;;
        de|fr|es)
            # German, French, Spanish: check dialog/standard.tlk for language-specific word
            local word=""
            for entry in "${LANGUAGE_WORDS[@]}"; do
                if [[ "$entry" == "${lang_code}:"* ]]; then
                    word="${entry#*:}"
                    break
                fi
            done
            if [[ -n "$word" ]] && unzip -p "$pack_path" "dialog/standard.tlk" 2>/dev/null | grep -q "$word"; then
                log_verbose "${lang_code} language pack validated: $pack_path"
                return 0
            fi
            ;;
    esac

    log_verbose "Language pack validation failed for $lang_code: $pack_path"
    return 1
}

# get_language_paths - Returns paths for language packs in a base directory
# Usage: get_language_paths <lang_code> <base_dir>
# Prints paths (one per line) to stdout
get_language_paths() {
    local lang_code="$1"
    local base_dir="$2"

    # Main language pack
    case "$lang_code" in
        en)
            echo "$base_dir/pak4_english.pk3"
            ;;
        de)
            echo "$base_dir/pak4_german.pk3"
            ;;
        fr)
            echo "$base_dir/pak4_french.pk3"
            ;;
        es)
            echo "$base_dir/pak4_spanish.pk3"
            ;;
        nl)
            echo "$base_dir/pak4_dutch.pk3"
            ;;
        *)
            return 1
            ;;
    esac

    # For translated languages, also return bundle paths
    case "$lang_code" in
        de)
            echo "$base_dir/base_g.pk3"
            ;;
        fr)
            echo "$base_dir/base_f.pk3"
            ;;
        es)
            echo "$base_dir/base_s.pk3"
            ;;
        nl)
            echo "$base_dir/base_d.pk3"
            ;;
    esac
}

# get_active_language - Determines active language in a base directory
# Usage: get_active_language <base_dir>
# Prints language code to stdout, returns 0 if found, 1 if none
get_active_language() {
    local base_dir="$1"

    # Priority: translated languages (de/fr/es/nl) over English
    for lang_code in de fr es nl en; do
        local pak_file=""
        case "$lang_code" in
            en) pak_file="pak4_english.pk3" ;;
            de) pak_file="pak4_german.pk3" ;;
            fr) pak_file="pak4_french.pk3" ;;
            es) pak_file="pak4_spanish.pk3" ;;
            nl) pak_file="pak4_dutch.pk3" ;;
        esac

        if [[ -f "$base_dir/$pak_file" ]]; then
            # Validate the pack
            if test_language "$lang_code" "$base_dir/$pak_file"; then
                echo "$lang_code"
                return 0
            fi
        fi
    done

    return 1
}

# get_saved_language_pack - Checks private/game-data/languages/ for saved pack
# Usage: get_saved_language_pack <lang_code> <private_dir>
# Prints path to stdout if found and valid, empty otherwise
get_saved_language_pack() {
    local lang_code="$1"
    local private_dir="$2"

    local lang_dir="$private_dir/game-data/languages/$lang_code"
    if [[ ! -d "$lang_dir" ]]; then
        return 1
    fi

    local pak_file=""
    case "$lang_code" in
        en) pak_file="pak4_english.pk3" ;;
        de) pak_file="pak4_german.pk3" ;;
        fr) pak_file="pak4_french.pk3" ;;
        es) pak_file="pak4_spanish.pk3" ;;
        nl) pak_file="pak4_dutch.pk3" ;;
        *) return 1 ;;
    esac

    local pack_path="$lang_dir/$pak_file"
    if [[ -f "$pack_path" ]] && test_language "$lang_code" "$pack_path"; then
        echo "$pack_path"
        return 0
    fi

    return 1
}

# import_languages - Copies language packs from source to language cache
# Usage: import_languages <source_dir> <private_dir>
# Source can be folder or already extracted base
# Destination: private/game-data/languages/
# Only copies if not already in cache
import_languages() {
    local source_dir="$1"
    local private_dir="$2"

    local cache_dir="$private_dir/game-data/languages"
    mkdir -p "$cache_dir"

    # Check each language pack in source
    for lang_pak in "${LANGUAGE_PAKS[@]}"; do
        local src_path="$source_dir/$lang_pak"
        if [[ ! -f "$src_path" ]]; then
            continue
        fi

        # Determine language code from filename
        local lang_code=""
        case "$lang_pak" in
            pak4_english.pk3) lang_code="en" ;;
            pak4_german.pk3) lang_code="de" ;;
            pak4_french.pk3) lang_code="fr" ;;
            pak4_spanish.pk3) lang_code="es" ;;
            pak4_dutch.pk3) lang_code="nl" ;;
        esac

        if [[ -z "$lang_code" ]]; then
            continue
        fi

        local dest_dir="$cache_dir/$lang_code"
        local dest_path="$dest_dir/$lang_pak"

        # Skip if already in cache and valid
        if [[ -f "$dest_path" ]] && test_language "$lang_code" "$dest_path"; then
            log_verbose "Language pack already cached: $lang_code"
            continue
        fi

        # Validate source pack
        if ! test_language "$lang_code" "$src_path"; then
            log_verbose "Source language pack invalid, skipping: $lang_code"
            continue
        fi

        # Copy to cache
        mkdir -p "$dest_dir"
        cp "$src_path" "$dest_path"
        log_verbose "Imported language pack to cache: $lang_code"
    done

    # Also import bundle packs for translated languages
    for bundle_pak in "${LANGUAGE_BUNDLE_PAKS[@]}"; do
        local src_path="$source_dir/$bundle_pak"
        if [[ ! -f "$src_path" ]]; then
            continue
        fi

        local lang_code=""
        case "$bundle_pak" in
            base_g.pk3) lang_code="de" ;;
            base_f.pk3) lang_code="fr" ;;
            base_s.pk3) lang_code="es" ;;
            base_d.pk3) lang_code="nl" ;;
        esac

        if [[ -z "$lang_code" ]]; then
            continue
        fi

        local dest_dir="$cache_dir/$lang_code"
        local dest_path="$dest_dir/$bundle_pak"

        if [[ -f "$dest_path" ]]; then
            log_verbose "Bundle pack already cached: $bundle_pak"
            continue
        fi

        mkdir -p "$dest_dir"
        cp "$src_path" "$dest_path"
        log_verbose "Imported bundle pack to cache: $bundle_pak"
    done
}

# import_selected_language - Extracts specific language pack from archive
# Usage: import_selected_language <lang_code> <archive_path> <private_dir>
# Uses get_archive_entries and extractor to extract single entry
# Saves to language cache
import_selected_language() {
    local lang_code="$1"
    local archive_path="$2"
    local private_dir="$3"

    local cache_dir="$private_dir/game-data/languages"
    mkdir -p "$cache_dir"

    local pak_file=""
    case "$lang_code" in
        en) pak_file="pak4_english.pk3" ;;
        de) pak_file="pak4_german.pk3" ;;
        fr) pak_file="pak4_french.pk3" ;;
        es) pak_file="pak4_spanish.pk3" ;;
        nl) pak_file="pak4_dutch.pk3" ;;
        *) return 1 ;;
    esac

    local dest_dir="$cache_dir/$lang_code"
    local dest_path="$dest_dir/$pak_file"

    # Skip if already in cache and valid
    if [[ -f "$dest_path" ]] && test_language "$lang_code" "$dest_path"; then
        log_verbose "Language pack already cached: $lang_code"
        return 0
    fi

    # Get archive entries
    local entries
    entries=$(get_archive_entries "$archive_path") || {
        echo "Failed to read archive entries: $archive_path" >&2
        return 1
    }

    # Find the correct prefix
    local known_prefixes=(
        "base/"
        "bin/base/"
        "Alice1/bin/base/"
        "alice_202106/Alice1/bin/base/"
    )

    local found_prefix=""
    for p in "${known_prefixes[@]}"; do
        if echo "$entries" | grep -q "^${p}${pak_file}$"; then
            found_prefix="$p"
            break
        fi
    done

    if [[ -z "$found_prefix" ]]; then
        echo "Could not find $pak_file in archive" >&2
        return 1
    fi

    local archive_entry="${found_prefix}${pak_file}"

    # Extract using appropriate extractor
    local ext="${archive_path##*.}"
    ext="${ext,,}"

    mkdir -p "$dest_dir"

    if [[ "$ext" == "zip" ]]; then
        if ! unzip -j -q -o "$archive_path" "$archive_entry" -d "$dest_dir" 2>/dev/null; then
            echo "Failed to extract $pak_file from zip archive" >&2
            return 1
        fi
    elif [[ "$ext" == "7z" ]]; then
        local extractor
        extractor=$(find_extractor) || return 1
        if ! "$extractor" e -y -bso0 -bsp0 "-o$dest_dir" "$archive_path" "$archive_entry" 2>/dev/null; then
            echo "Failed to extract $pak_file from 7z archive" >&2
            return 1
        fi
    else
        echo "Unsupported archive format: $ext" >&2
        return 1
    fi

    # Validate extracted pack
    if ! test_language "$lang_code" "$dest_path"; then
        echo "Extracted language pack failed validation: $lang_code" >&2
        rm -f "$dest_path"
        return 1
    fi

    log_verbose "Extracted and validated language pack: $lang_code"

    # Also extract bundle pack for translated languages
    local bundle_pak=""
    case "$lang_code" in
        de) bundle_pak="language_german.pk3" ;;
        fr) bundle_pak="language_french.pk3" ;;
        es) bundle_pak="language_spanish.pk3" ;;
        nl) bundle_pak="" ;;  # Dutch doesn't have a language_*.pk3 bundle in the archive
    esac

    if [[ -n "$bundle_pak" ]]; then
        local bundle_archive_entry="${found_prefix}${bundle_pak}"
        local bundle_dest_path="$dest_dir/base_${lang_code}.pk3"

        if echo "$entries" | grep -q "^${bundle_archive_entry}$"; then
            if [[ "$ext" == "zip" ]]; then
                unzip -j -q -o "$archive_path" "$bundle_archive_entry" -d "$dest_dir" 2>/dev/null
            else
                local extractor
                extractor=$(find_extractor) || return 1
                "$extractor" e -y -bso0 -bsp0 "-o$dest_dir" "$archive_path" "$bundle_archive_entry" 2>/dev/null
            fi
            # Rename to base_*.pk3 format
            if [[ -f "$dest_dir/$bundle_pak" ]]; then
                mv "$dest_dir/$bundle_pak" "$bundle_dest_path"
                log_verbose "Extracted bundle pack: $bundle_pak -> base_${lang_code}.pk3"
            fi
        fi
    fi

    return 0
}

# prepare_language - Prepares required languages for installation
# Usage: prepare_language <lang_code> <source> <private_dir>
# For 'nl' (Dutch), also requires 'en' (English)
# Calls import_selected_language for each required language
prepare_language() {
    local lang_code="$1"
    local source="$2"
    local private_dir="$3"

    local required_langs=("$lang_code")

    # Dutch requires English as fallback
    if [[ "$lang_code" == "nl" ]]; then
        required_langs=("en" "nl")
    fi

    for req_lang in "${required_langs[@]}"; do
        if [[ -d "$source" ]]; then
            # Source is a directory, use import_languages
            import_languages "$source" "$private_dir"
        else
            # Source is an archive
            import_selected_language "$req_lang" "$source" "$private_dir" || {
                echo "Failed to prepare language: $req_lang" >&2
                return 1
            }
        fi
    done
}

# apply_language - Applies language packs to base directory
# Usage: apply_language <lang_code> <base_dir> <private_dir>
# Moves old language packs to temp, copies new ones in
# Handles transaction-style rollback on error
apply_language() {
    local lang_code="$1"
    local base_dir="$2"
    local private_dir="$3"

    local cache_dir="$private_dir/game-data/languages/$lang_code"
    if [[ ! -d "$cache_dir" ]]; then
        echo "Language pack not found in cache: $lang_code" >&2
        return 1
    fi

    # Get list of files to apply
    local files_to_apply=()
    while IFS= read -r path; do
        if [[ -f "$path" ]]; then
            files_to_apply+=("$path")
        fi
    done < <(get_language_paths "$lang_code" "$cache_dir")

    if [[ ${#files_to_apply[@]} -eq 0 ]]; then
        echo "No language files to apply for: $lang_code" >&2
        return 1
    fi

    # Create temp directory for rollback
    local temp_dir
    temp_dir=$(mktemp -d "${base_dir}/.lg_lang_backup.XXXXXX")
    log_verbose "Created backup directory: $temp_dir"

    # Move existing language packs to temp (backup)
    for lang_pak in "${LANGUAGE_PAKS[@]}"; do
        if [[ -f "$base_dir/$lang_pak" ]]; then
            mv "$base_dir/$lang_pak" "$temp_dir/"
            log_verbose "Backed up: $lang_pak"
        fi
    done

    # Move existing bundle packs to temp
    for bundle_pak in "${LANGUAGE_BUNDLE_PAKS[@]}"; do
        if [[ -f "$base_dir/$bundle_pak" ]]; then
            mv "$base_dir/$bundle_pak" "$temp_dir/"
            log_verbose "Backed up bundle: $bundle_pak"
        fi
    done

    # Copy new language packs from cache
    local error_occurred=0
    for src_path in "${files_to_apply[@]}"; do
        local filename=$(basename "$src_path")
        if ! cp "$src_path" "$base_dir/$filename"; then
            echo "Failed to copy $filename to base directory" >&2
            error_occurred=1
            break
        fi
        log_verbose "Applied: $filename"
    done

    if [[ $error_occurred -eq 1 ]]; then
        # Rollback: restore from backup
        log_verbose "Rolling back language packs..."
        for backup_file in "$temp_dir"/*; do
            [[ -f "$backup_file" ]] || continue
            local filename=$(basename "$backup_file")
            mv "$backup_file" "$base_dir/$filename"
            log_verbose "Restored: $filename"
        done
        rmdir "$temp_dir"
        return 1
    fi

    # Success: remove backup directory
    rm -rf "$temp_dir"
    log_verbose "Language applied successfully: $lang_code"
    return 0
}

# switch_installed_language - Switch language on an already configured base
# Usage: switch_installed_language <private_dir> <lang_code>
# Reads base path from private/data-path.txt, validates it's a managed base,
# prepares and applies the new language
switch_installed_language() {
    local private_dir="$1"
    local lang_code="$2"

    local data_path_file="$private_dir/data-path.txt"
    if [[ ! -f "$data_path_file" ]]; then
        echo "No configured game data found. Run setup first." >&2
        return 1
    fi

    local base_dir
    base_dir=$(cat "$data_path_file" 2>/dev/null)
    if [[ -z "$base_dir" || ! -d "$base_dir" ]]; then
        echo "Configured game data path is invalid: $base_dir" >&2
        return 1
    fi

    # Verify it's a managed base (under private/game-data/)
    local managed_prefix="$private_dir/game-data/"
    if [[ "$base_dir" != "$managed_prefix"* ]]; then
        echo "Configured base is not a managed installation. Cannot switch language." >&2
        return 1
    fi

    # Get current active language
    local current_lang
    current_lang=$(get_active_language "$base_dir")
    if [[ -z "$current_lang" ]]; then
        echo "Could not determine current language in: $base_dir" >&2
        return 1
    fi

    log_verbose "Current language: $current_lang, requested: $lang_code"

    if [[ "$current_lang" == "$lang_code" ]]; then
        echo "Language $lang_code is already active" >&2
        return 1
    fi

    # Prepare the new language
    prepare_language "$lang_code" "$base_dir" "$private_dir" || {
        echo "Failed to prepare language: $lang_code" >&2
        return 1
    }

    # Apply the new language
    apply_language "$lang_code" "$base_dir" "$private_dir" || {
        echo "Failed to apply language: $lang_code" >&2
        return 1
    }

    echo "Language switched to: $lang_code"
    return 0
}

# main_setup - Main orchestration function for setup
# Reads environment variables set by Setup.sh and performs the setup
main_setup() {
    # Read environment variables with defaults
    local action="${LG_SETUP_ACTION:-setup}"
    local source="${LG_SETUP_SOURCE:-}"
    local lang_code="${LG_SETUP_LANGUAGE:-}"
    local download="${LG_SETUP_DOWNLOAD:-false}"
    local dry_run="${LG_SETUP_DRY_RUN:-false}"
    local gui_tool="${LG_SETUP_GUI:-cli}"
    local verbose="${LG_SETUP_VERBOSE:-false}"
    local root_dir="${LG_SETUP_ROOT:-$(pwd)}"

    # Set verbose mode
    if [[ "$verbose" == "true" ]]; then
        VERBOSE=1
    fi

    log_verbose "main_setup: action=$action, source=$source, lang=$lang_code, download=$download, dry_run=$dry_run, gui=$gui_tool, root=$root_dir"

    # Handle language switch action
    if [[ "$action" == "language" ]]; then
        if [[ -z "$lang_code" ]]; then
            echo "Error: LG_SETUP_LANGUAGE must be set for language action" >&2
            return 1
        fi
        switch_installed_language "$root_dir/private" "$lang_code"
        return $?
    fi

    # Handle setup action
    if [[ "$action" != "setup" ]]; then
        echo "Error: Invalid action: $action (must be 'setup' or 'language')" >&2
        return 1
    fi

    local private_dir="$root_dir/private"
    local base_dir=""
    local archive_sha256=""

    # Step 1: Determine source
    if [[ -n "$source" ]]; then
        log_verbose "Using provided source: $source"
        # Source is explicitly provided
    else
        # Try find_game_data
        base_dir=$(find_game_data)
        if [[ -n "$base_dir" ]]; then
            log_verbose "Found game data via find_game_data: $base_dir"
        else
            # Check private/data-path.txt for saved config
            local data_path_file="$private_dir/data-path.txt"
            if [[ -f "$data_path_file" ]]; then
                base_dir=$(cat "$data_path_file" 2>/dev/null)
                if [[ -n "$base_dir" && -d "$base_dir" ]]; then
                    log_verbose "Found saved config: $base_dir"
                else
                    base_dir=""
                fi
            fi
        fi

        # Check for Alice1_2011_vanilla.7z in project root or alice_202106/
        if [[ -z "$base_dir" ]]; then
            for archive_path in "$root_dir/Alice1_2011_vanilla.7z" "$root_dir/alice_202106/Alice1_2011_vanilla.7z"; do
                if [[ -f "$archive_path" ]]; then
                    source="$archive_path"
                    log_verbose "Found archive: $source"
                    break
                fi
            done
        fi
    fi

    # Step 2: Handle no source found
    if [[ -z "$base_dir" && -z "$source" ]]; then
        if [[ "$download" == "true" ]]; then
            log_verbose "No source found, downloading..."
            local download_dir="$private_dir/downloads"
            source=$(download_game_data "$download_dir") || {
                echo "Failed to download game data" >&2
                return 1
            }
        else
            echo "No game data found. Provide --data <path>, use --download, or place Alice1_2011_vanilla.7z in project root." >&2
            return 1
        fi
    fi

    # Step 3: Process source (directory or archive)
    if [[ -n "$base_dir" ]]; then
        # Source is already a directory (from find_game_data or config)
        log_verbose "Validating base directory: $base_dir"
        if ! validate_base "$base_dir" true; then
            echo "Configured base directory failed validation: $base_dir" >&2
            return 1
        fi
    elif [[ -f "$source" ]]; then
        # Source is an archive
        local ext="${source##*.}"
        ext="${ext,,}"

        if [[ "$ext" != "7z" && "$ext" != "zip" ]]; then
            echo "Unsupported archive format: $ext" >&2
            return 1
        fi

        # Compute SHA256
        archive_sha256=$(compute_sha256 "$source") || {
            echo "Failed to compute SHA256 of archive" >&2
            return 1
        }
        log_verbose "Archive SHA256: $archive_sha256"

        # Check receipt for existing import with same hash
        local receipt_file="$private_dir/setup.json"
        local existing_base=""
        if [[ -f "$receipt_file" ]]; then
            local existing_sha256=""
            if command -v jq >/dev/null 2>&1; then
                existing_sha256=$(jq -r '.archiveSha256 // empty' "$receipt_file" 2>/dev/null)
            else
                # Simple grep fallback
                existing_sha256=$(grep -o '"archiveSha256"[[:space:]]*:[[:space:]]*"[^"]*"' "$receipt_file" 2>/dev/null | sed 's/.*"archiveSha256"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
            fi
            if [[ "$existing_sha256" == "$archive_sha256" ]]; then
                # Check if the existing path is still valid
                local existing_path=""
                if command -v jq >/dev/null 2>&1; then
                    existing_path=$(jq -r '.dataPath // empty' "$receipt_file" 2>/dev/null)
                else
                    existing_path=$(grep -o '"dataPath"[[:space:]]*:[[:space:]]*"[^"]*"' "$receipt_file" 2>/dev/null | sed 's/.*"dataPath"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
                fi
                if [[ -n "$existing_path" && -d "$existing_path" ]] && validate_base "$existing_path" true; then
                    log_verbose "Found valid existing import with same hash: $existing_path"
                    base_dir="$existing_path"
                fi
            fi
        fi

        if [[ -z "$base_dir" ]]; then
            # Create new managed import location
            local uuid=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || date +%s%N)
            base_dir="$private_dir/game-data/import-$uuid/base"

            if [[ "$dry_run" == "true" ]]; then
                echo "[DRY RUN] Would extract to: $base_dir"
            else
                # Extract base paks
                log_verbose "Extracting base paks to: $base_dir"
                extract_base_paks "$source" "$base_dir" "base/" || {
                    echo "Failed to extract base paks from archive" >&2
                    return 1
                }

                # Validate extracted base
                if ! validate_base "$base_dir" true; then
                    echo "Extracted base failed validation" >&2
                    return 1
                fi
            fi
        fi
    else
        echo "No valid game data source found" >&2
        return 1
    fi

    # Step 4: Determine language
    if [[ -z "$lang_code" ]]; then
        lang_code=$(get_active_language "$base_dir")
        if [[ -z "$lang_code" ]]; then
            echo "Missing language pack in game data" >&2
            return 1
        fi
        log_verbose "Detected language: $lang_code"
    fi

    # Step 5: Prepare language packs
    if [[ "$dry_run" != "true" ]]; then
        prepare_language "$lang_code" "$source" "$private_dir" || {
            echo "Failed to prepare language: $lang_code" >&2
            return 1
        }
    else
        echo "[DRY RUN] Would prepare language: $lang_code"
    fi

    # Step 6: Import additional languages from source
    if [[ "$dry_run" != "true" ]]; then
        if [[ -d "$source" ]]; then
            import_languages "$source" "$private_dir"
        elif [[ -f "$source" ]]; then
            # Import all available languages from archive
            for l in en de fr es nl; do
                if [[ "$l" != "$lang_code" ]]; then
                    import_selected_language "$l" "$source" "$private_dir" || true
                fi
            done
        fi
    else
        echo "[DRY RUN] Would import additional languages"
    fi

    # Step 7: Ensure base is in managed location
    local managed_prefix="$private_dir/game-data/"
    if [[ "$base_dir" != "$managed_prefix"* ]]; then
        local uuid=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || date +%s%N)
        local managed_base="$private_dir/game-data/import-$uuid/base"

        if [[ "$dry_run" == "true" ]]; then
            echo "[DRY RUN] Would copy to managed location: $managed_base"
        else
            log_verbose "Copying base to managed location: $managed_base"
            mkdir -p "$(dirname "$managed_base")"
            cp -r "$base_dir"/* "$managed_base/" || {
                echo "Failed to copy base to managed location" >&2
                return 1
            }
            base_dir="$managed_base"
        fi
    fi

    # Step 8: Apply language
    if [[ "$dry_run" != "true" ]]; then
        apply_language "$lang_code" "$base_dir" "$private_dir" || {
            echo "Failed to apply language: $lang_code" >&2
            return 1
        }
    else
        echo "[DRY RUN] Would apply language: $lang_code"
    fi

    # Step 9: Write receipt and config
    if [[ "$dry_run" != "true" ]]; then
        write_config "$private_dir/data-path.txt" "$base_dir"
        write_receipt "$private_dir/setup.json" "$source" "$archive_sha256" "$base_dir"
    else
        echo "[DRY RUN] Would write receipt and config"
    fi

    echo "Setup complete. Game data at: $base_dir"
    return 0
}

# Parse command line arguments
# Environment variables take precedence over command line for Setup.sh compatibility
ACTION="${LG_SETUP_ACTION:-setup}"
SOURCE="${LG_SETUP_SOURCE:-}"
LANG_CODE="${LG_SETUP_LANGUAGE:-}"
DOWNLOAD="${LG_SETUP_DOWNLOAD:-false}"
DRY_RUN="${LG_SETUP_DRY_RUN:-false}"
GUI_TOOL="${LG_SETUP_GUI:-cli}"
VERBOSE_FLAG="${LG_SETUP_VERBOSE:-false}"
ROOT_DIR="${LG_SETUP_ROOT:-$(pwd)}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --find-data)
            if find_game_data; then
                exit 0
            else
                exit 1
            fi
            ;;
        --language)
            LANG_CODE="$2"
            shift 2
            ;;
        --data)
            SOURCE="$2"
            shift 2
            ;;
        --download)
            DOWNLOAD="true"
            shift
            ;;
        --gui)
            GUI_TOOL="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        -v|--verbose)
            VERBOSE_FLAG="true"
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --find-data           Find and print game data directory"
            echo "  --language <code>     Language code (en, de, fr, es, nl)"
            echo "  --data <path>         Path to game data archive or folder"
            echo "  --download            Download game data from Internet Archive"
            echo "  --gui <tool>          GUI tool: zenity, kdialog, yad, cli"
            echo "  --dry-run             Simulate without making changes"
            echo "  -v, --verbose         Verbose output"
            echo "  --help                Show this help"
            echo ""
            echo "Environment variables (used by Setup.sh):"
            echo "  LG_SETUP_ACTION       setup|language (default: setup)"
            echo "  LG_SETUP_SOURCE       Path to archive or folder"
            echo "  LG_SETUP_LANGUAGE     Language code (en, de, fr, es, nl)"
            echo "  LG_SETUP_DOWNLOAD     true|false (default: false)"
            echo "  LG_SETUP_DRY_RUN      true|false (default: false)"
            echo "  LG_SETUP_GUI          zenity|kdialog|yad|cli (default: cli)"
            echo "  LG_SETUP_VERBOSE      true|false (default: false)"
            echo "  LG_SETUP_ROOT         Project root directory"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Use --help for usage" >&2
            exit 1
            ;;
    esac
done

# Set verbose from flag or env var
if [[ "$VERBOSE_FLAG" == "true" ]]; then
    VERBOSE=1
fi

# Export for main_setup
export LG_SETUP_ACTION="$ACTION"
export LG_SETUP_SOURCE="$SOURCE"
export LG_SETUP_LANGUAGE="$LANG_CODE"
export LG_SETUP_DOWNLOAD="$DOWNLOAD"
export LG_SETUP_DRY_RUN="$DRY_RUN"
export LG_SETUP_GUI="$GUI_TOOL"
export LG_SETUP_VERBOSE="$VERBOSE_FLAG"
export LG_SETUP_ROOT="$ROOT_DIR"

# Call main_setup for setup or language actions
if [[ "$ACTION" == "setup" || "$ACTION" == "language" ]]; then
    main_setup
    exit $?
fi

# If no arguments provided and no action, show usage
if [[ $# -eq 0 && -z "$LG_SETUP_ACTION" ]]; then
    echo "Usage: $0 [--find-data] [--language <code>] [--data <path>] [--download] [--gui <tool>] [--dry-run] [-v|--verbose] [--help]" >&2
    exit 1
fi