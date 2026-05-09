#!/usr/bin/env bash
# ABG Data Registry — Dataset management with provenance tracking
#
# Manages shared datasets with checksums and manifest files.
# Nucleates sweetGrass provenance replacement — when the provenance trio
# is live, braid URNs replace SHA256 as the integrity mechanism.
#
# Usage:
#   bash abg_data.sh register <path>       — register a dataset directory
#   bash abg_data.sh check [<path>]        — verify checksums (all or specific)
#   bash abg_data.sh search <term>         — search by name, accession, notes
#   bash abg_data.sh list                  — list all registered datasets
#   bash abg_data.sh duplicate <file>      — check if file hash already exists
#
# Examples:
#   bash abg_data.sh register $ABG_SHARED/data/ncbi/SRR12345678
#   bash abg_data.sh check
#   bash abg_data.sh search "soil microbiome"
#   bash abg_data.sh list

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

ABG_DATA="${ABG_SHARED}/data"
ROOT_MANIFEST="$ABG_DATA/manifest.toml"

register_dataset() {
    local dataset_path="$1"

    if [[ ! -d "$dataset_path" ]]; then
        echo "ERROR: '$dataset_path' is not a directory" >&2
        exit 1
    fi

    local abs_path
    abs_path="$(cd "$dataset_path" && pwd)"
    local rel_path="${abs_path#$ABG_DATA/}"
    local dataset_name
    dataset_name="$(basename "$abs_path")"

    if [[ -f "$abs_path/manifest.toml" ]]; then
        echo "Dataset '$dataset_name' already has a manifest. Updating checksums..."
    fi

    local total_size=0
    local file_count=0
    local checksums=""

    while IFS= read -r -d '' file; do
        local fname
        fname="$(basename "$file")"
        local fsize
        fsize=$(stat -c%s "$file")
        local fhash
        fhash=$(sha256sum "$file" | cut -d' ' -f1)
        total_size=$((total_size + fsize))
        file_count=$((file_count + 1))
        checksums="${checksums}
[[files]]
name = \"$fname\"
sha256 = \"$fhash\"
size_bytes = $fsize
"
    done < <(find "$abs_path" -maxdepth 1 -type f ! -name 'manifest.toml' -print0 | sort -z)

    local who
    who="$(whoami)"
    local when
    when="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    cat > "$abs_path/manifest.toml" << EOF
[dataset]
name = "$dataset_name"
path = "$rel_path"
source = ""
type = ""
total_sha256 = ""
size_bytes = $total_size
file_count = $file_count
registered_by = "$who"
registered_at = "$when"
notes = ""

[provenance]
braid_urn = ""
witness_key = ""
$checksums
EOF

    echo "Registered: $dataset_name ($file_count files, $(numfmt --to=iec $total_size))"
    echo "  Manifest: $abs_path/manifest.toml"
    echo "  Edit the manifest to fill in source, type, and notes fields."

    _update_root_manifest
}

check_datasets() {
    local target="${1:-}"
    local errors=0
    local checked=0

    local search_path="$ABG_DATA"
    if [[ -n "$target" ]]; then
        search_path="$target"
    fi

    while IFS= read -r manifest; do
        local dir
        dir="$(dirname "$manifest")"
        local dataset_name
        dataset_name="$(basename "$dir")"

        while IFS= read -r line; do
            local fname fhash
            fname=$(echo "$line" | grep -oP 'name = "\K[^"]+' || true)
            if [[ -z "$fname" ]]; then continue; fi

            fhash=$(echo "$line" | grep -oP 'sha256 = "\K[^"]+' 2>/dev/null || true)
            read -r next_line
            if [[ -z "$fhash" ]]; then
                fhash=$(echo "$next_line" | grep -oP 'sha256 = "\K[^"]+' || true)
            fi

            if [[ -z "$fhash" ]]; then continue; fi

            local filepath="$dir/$fname"
            if [[ ! -f "$filepath" ]]; then
                echo "MISSING: $filepath"
                errors=$((errors + 1))
                continue
            fi

            local actual
            actual=$(sha256sum "$filepath" | cut -d' ' -f1)
            if [[ "$actual" != "$fhash" ]]; then
                echo "CORRUPTED: $filepath (expected ${fhash:0:16}..., got ${actual:0:16}...)"
                errors=$((errors + 1))
            else
                checked=$((checked + 1))
            fi
        done < "$manifest"

    done < <(find "$search_path" -name 'manifest.toml' -not -path "$ROOT_MANIFEST" -print)

    echo ""
    echo "Checked: $checked files, $errors errors"
    if [[ $errors -gt 0 ]]; then
        exit 1
    fi
}

search_datasets() {
    local term="$1"
    echo "Searching for '$term' in data registry..."
    echo ""

    while IFS= read -r manifest; do
        if grep -qil "$term" "$manifest" 2>/dev/null; then
            local dir
            dir="$(dirname "$manifest")"
            local name
            name="$(basename "$dir")"
            local size
            size=$(grep 'size_bytes' "$manifest" | head -1 | grep -oP '\d+' || echo "?")
            local source
            source=$(grep 'source' "$manifest" | head -1 | grep -oP '"[^"]*"' | tr -d '"' || echo "")
            local notes
            notes=$(grep 'notes' "$manifest" | head -1 | grep -oP '"[^"]*"' | tr -d '"' || echo "")

            echo "  $name"
            [[ -n "$source" ]] && echo "    source: $source"
            [[ -n "$notes" ]] && echo "    notes: $notes"
            [[ "$size" != "?" ]] && echo "    size: $(numfmt --to=iec "$size" 2>/dev/null || echo "${size} bytes")"
            echo ""
        fi
    done < <(find "$ABG_DATA" -name 'manifest.toml' -not -path "$ROOT_MANIFEST" -print)
}

list_datasets() {
    echo "ABG Data Registry"
    echo "================="
    echo ""

    local total_size=0
    local total_count=0

    printf "%-30s %-12s %-8s %-20s %s\n" "DATASET" "SIZE" "FILES" "REGISTERED" "TYPE"
    printf "%-30s %-12s %-8s %-20s %s\n" "-------" "----" "-----" "----------" "----"

    while IFS= read -r manifest; do
        local dir
        dir="$(dirname "$manifest")"
        local name
        name="$(basename "$dir")"

        local size file_count reg_at dtype
        size=$(grep '^size_bytes' "$manifest" | head -1 | grep -oP '\d+' || echo "0")
        file_count=$(grep '^file_count' "$manifest" | head -1 | grep -oP '\d+' || echo "?")
        reg_at=$(grep 'registered_at' "$manifest" | grep -oP '"[^"]*"' | tr -d '"' | cut -dT -f1 || echo "?")
        dtype=$(grep '^type' "$manifest" | head -1 | grep -oP '"[^"]*"' | tr -d '"' || echo "")

        local human_size
        human_size=$(numfmt --to=iec "$size" 2>/dev/null || echo "${size}B")

        printf "%-30s %-12s %-8s %-20s %s\n" "$name" "$human_size" "$file_count" "$reg_at" "$dtype"

        total_size=$((total_size + size))
        total_count=$((total_count + 1))
    done < <(find "$ABG_DATA" -name 'manifest.toml' -not -path "$ROOT_MANIFEST" -print | sort)

    echo ""
    echo "Total: $total_count datasets, $(numfmt --to=iec $total_size 2>/dev/null || echo "${total_size} bytes")"
    echo "Disk:  $(df -h "$ABG_DATA" | tail -1 | awk '{print $4}') available"
}

duplicate_check() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo "ERROR: '$file' not found" >&2
        exit 1
    fi

    local fhash
    fhash=$(sha256sum "$file" | cut -d' ' -f1)
    echo "SHA256: $fhash"
    echo ""

    local found=0
    while IFS= read -r manifest; do
        if grep -q "$fhash" "$manifest" 2>/dev/null; then
            local dir
            dir="$(dirname "$manifest")"
            echo "DUPLICATE FOUND: $(basename "$dir")/"
            echo "  Manifest: $manifest"
            found=1
        fi
    done < <(find "$ABG_DATA" -name 'manifest.toml' -print)

    if [[ $found -eq 0 ]]; then
        echo "No duplicates found in registry."
    fi
}

_update_root_manifest() {
    local total_datasets=0
    local total_bytes=0

    local datasets_block=""

    while IFS= read -r manifest; do
        local dir
        dir="$(dirname "$manifest")"
        local name
        name="$(basename "$dir")"
        local rel_path="${dir#$ABG_DATA/}"
        local size
        size=$(grep '^size_bytes' "$manifest" | head -1 | grep -oP '\d+' || echo "0")
        local dtype
        dtype=$(grep '^type' "$manifest" | head -1 | grep -oP '"[^"]*"' | tr -d '"' || echo "")

        local top_hash=""
        local first_file_hash
        first_file_hash=$(grep -A1 '^\[\[files\]\]' "$manifest" 2>/dev/null | grep 'sha256' | head -1 | grep -oP '"[^"]*"' | tr -d '"' || echo "")
        top_hash="${first_file_hash}"

        datasets_block="${datasets_block}
[[datasets]]
name = \"$name\"
path = \"$rel_path\"
sha256 = \"$top_hash\"
size_bytes = $size
type = \"$dtype\"
"
        total_datasets=$((total_datasets + 1))
        total_bytes=$((total_bytes + size))
    done < <(find "$ABG_DATA" -name 'manifest.toml' -not -path "$ROOT_MANIFEST" -print | sort)

    cat > "$ROOT_MANIFEST" << EOF
# ABG Data Registry — Root Manifest (auto-generated by abg_data.sh)
# Do not edit manually — run: bash abg_data.sh register <path>

[registry]
version = 1
updated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
updated_by = "$(whoami)"
total_datasets = $total_datasets
total_bytes = $total_bytes
$datasets_block
EOF
}

case "${1:-}" in
    register)
        [[ -z "${2:-}" ]] && { echo "Usage: $0 register <path>"; exit 1; }
        register_dataset "$2"
        ;;
    check)
        check_datasets "${2:-}"
        ;;
    search)
        [[ -z "${2:-}" ]] && { echo "Usage: $0 search <term>"; exit 1; }
        search_datasets "$2"
        ;;
    list)
        list_datasets
        ;;
    duplicate)
        [[ -z "${2:-}" ]] && { echo "Usage: $0 duplicate <file>"; exit 1; }
        duplicate_check "$2"
        ;;
    *)
        echo "ABG Data Registry — Dataset management with provenance tracking"
        echo ""
        echo "Usage: $0 <command> [args]"
        echo ""
        echo "Commands:"
        echo "  register <path>    Register a dataset directory (computes checksums)"
        echo "  check [<path>]     Verify checksums (all or specific dataset)"
        echo "  search <term>      Search datasets by name, accession, notes"
        echo "  list               List all registered datasets"
        echo "  duplicate <file>   Check if a file's hash exists in registry"
        ;;
esac
