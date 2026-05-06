#!/usr/bin/env bash
# ABG Account Management — Tiered Access for Compute Sharing
#
# Creates Linux users in tiered groups with scoped permissions.
# JupyterHub uses PAM auth, so system accounts = JupyterHub accounts.
#
# Tiers:
#   abg-observer  — read-only notebooks, view results, no workload submission
#   abg-compute   — run workloads via ToadStool, submit jobs, view provenance
#   abg-admin     — full access, can manage other users, access raw primal APIs
#
# Usage:
#   sudo bash abg_accounts.sh add <username> <tier>
#   sudo bash abg_accounts.sh list
#   sudo bash abg_accounts.sh remove <username>
#
# Example:
#   sudo bash abg_accounts.sh add jdoe compute
#   sudo bash abg_accounts.sh add msmith observer

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
JUPYTERHUB_CONFIG="/home/irongate/jupyterhub/jupyterhub_config.py"

SHARED_NOTEBOOKS="/home/irongate/Development/ecoPrimals/sporeGarden/projectNUCLEUS/notebooks"
SHARED_DATA="/home/irongate/Development/ecoPrimals/springs/wetSpring/data"
PLASMIDBIN="/home/irongate/Development/ecoPrimals/infra/plasmidBin"

ensure_groups() {
    for grp in abg-observer abg-compute abg-admin; do
        getent group "$grp" > /dev/null 2>&1 || groupadd "$grp"
    done
    echo "Groups: abg-observer, abg-compute, abg-admin"
}

setup_shared_dirs() {
    local user_home="$1"
    local tier="$2"

    mkdir -p "$user_home/notebooks"
    mkdir -p "$user_home/results"

    # Symlink shared notebooks so users see them but can't modify originals
    ln -sf "$SHARED_NOTEBOOKS/abg-wetspring-validation.ipynb" "$user_home/notebooks/" 2>/dev/null || true

    if [[ "$tier" == "compute" || "$tier" == "admin" ]]; then
        # Compute users get symlinks to data and workload definitions
        ln -sf "$PROJECT_ROOT/workloads" "$user_home/workloads" 2>/dev/null || true
        ln -sf "$SHARED_DATA" "$user_home/data" 2>/dev/null || true
    fi

    chown -R "$username:$username" "$user_home/notebooks" "$user_home/results"
}

write_user_env() {
    local user_home="$1"
    local tier="$2"
    local username="$3"

    cat > "$user_home/.nucleus_env" << EOF
# projectNUCLEUS environment for $username (tier: $tier)
export NUCLEUS_TIER="$tier"
export NUCLEUS_ROOT="$PROJECT_ROOT"
export TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED=1
export BEARDOG_PORT=9100
export SONGBIRD_PORT=9200
export TOADSTOOL_PORT=9400
export NESTGATE_PORT=9500
export RHIZOCRYPT_PORT=9601
export LOAMSPINE_PORT=9700
export SWEETGRASS_PORT=9850
EOF

    if [[ "$tier" == "observer" ]]; then
        echo "export NUCLEUS_READONLY=1" >> "$user_home/.nucleus_env"
    fi

    chown "$username:$username" "$user_home/.nucleus_env"
}

add_user() {
    local username="$1"
    local tier="$2"

    case "$tier" in
        observer|compute|admin) ;;
        *) echo "ERROR: Invalid tier '$tier'. Use: observer, compute, admin" >&2; exit 1 ;;
    esac

    if id "$username" &>/dev/null; then
        echo "User '$username' already exists — updating tier to $tier"
    else
        useradd -m -s /bin/bash "$username"
        echo "Created user: $username"
        echo ""
        echo "Set password with: sudo passwd $username"
    fi

    # Assign to tier group (remove from other tiers first)
    for grp in abg-observer abg-compute abg-admin; do
        gpasswd -d "$username" "$grp" 2>/dev/null || true
    done

    usermod -aG "abg-$tier" "$username"

    # Compute and admin tiers also get observer access
    if [[ "$tier" == "compute" || "$tier" == "admin" ]]; then
        usermod -aG "abg-observer" "$username"
    fi
    if [[ "$tier" == "admin" ]]; then
        usermod -aG "abg-compute" "$username"
    fi

    setup_shared_dirs "/home/$username" "$tier"
    write_user_env "/home/$username" "$tier" "$username"

    echo ""
    echo "User: $username"
    echo "Tier: $tier"
    echo "Groups: $(id -nG "$username")"
    echo "Home: /home/$username"
    echo ""
    echo "Capabilities:"
    case "$tier" in
        observer)
            echo "  - View shared notebooks and results"
            echo "  - Health check primals"
            echo "  - Read provenance manifests"
            echo "  - NO workload submission"
            ;;
        compute)
            echo "  - All observer capabilities"
            echo "  - Submit workloads via ToadStool"
            echo "  - Access NCBI data and workload definitions"
            echo "  - View provenance chain (DAG, ledger, braid)"
            echo "  - Run validation notebooks"
            ;;
        admin)
            echo "  - All compute capabilities"
            echo "  - Query raw primal APIs"
            echo "  - View deployment status"
            echo "  - Manage other ABG users"
            ;;
    esac

    echo ""
    echo "Next: Add to JupyterHub config and restart"
    echo "  Then: sudo passwd $username"
}

list_users() {
    echo "=== ABG Users ==="
    echo ""
    for tier in admin compute observer; do
        local members
        members=$(getent group "abg-$tier" 2>/dev/null | cut -d: -f4)
        echo "[$tier]"
        if [[ -n "$members" ]]; then
            echo "  $members" | tr ',' '\n' | while read -r u; do
                [[ -n "$u" ]] && echo "  - $u"
            done
        else
            echo "  (none)"
        fi
        echo ""
    done
}

remove_user() {
    local username="$1"

    if ! id "$username" &>/dev/null; then
        echo "User '$username' does not exist"
        exit 1
    fi

    for grp in abg-observer abg-compute abg-admin; do
        gpasswd -d "$username" "$grp" 2>/dev/null || true
    done

    echo "Removed $username from all ABG groups."
    echo "To fully delete: sudo userdel -r $username"
    echo "To remove from JupyterHub: edit $JUPYTERHUB_CONFIG"
}

generate_jupyterhub_config() {
    local admin_users="'irongate'"
    local allowed_users="'irongate'"

    for tier in admin compute observer; do
        local members
        members=$(getent group "abg-$tier" 2>/dev/null | cut -d: -f4)
        if [[ -n "$members" ]]; then
            for u in $(echo "$members" | tr ',' ' '); do
                allowed_users="$allowed_users, '$u'"
                if [[ "$tier" == "admin" ]]; then
                    admin_users="$admin_users, '$u'"
                fi
            done
        fi
    done

    echo ""
    echo "=== JupyterHub Config Update ==="
    echo "Add these lines to $JUPYTERHUB_CONFIG:"
    echo ""
    echo "c.Authenticator.admin_users = {$admin_users}"
    echo "c.Authenticator.allowed_users = {$allowed_users}"
    echo ""
    echo "Then restart: cd ~/jupyterhub && pkill jupyterhub; bash start.sh &"
}

# --- Main ---

if [[ $# -lt 1 ]]; then
    echo "Usage: sudo bash $0 {add|list|remove|config} [args...]"
    exit 1
fi

ACTION="$1"; shift

ensure_groups

case "$ACTION" in
    add)
        [[ $# -lt 2 ]] && { echo "Usage: sudo bash $0 add <username> <tier>"; exit 1; }
        add_user "$1" "$2"
        generate_jupyterhub_config
        ;;
    list)
        list_users
        ;;
    remove)
        [[ $# -lt 1 ]] && { echo "Usage: sudo bash $0 remove <username>"; exit 1; }
        remove_user "$1"
        generate_jupyterhub_config
        ;;
    config)
        generate_jupyterhub_config
        ;;
    *)
        echo "Unknown action: $ACTION"
        echo "Usage: sudo bash $0 {add|list|remove|config} [args...]"
        exit 1
        ;;
esac
