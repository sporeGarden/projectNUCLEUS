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
#   abg-reviewer  — external PI/HPC admin, showcase/ read-only, no execute
#
# Usage:
#   sudo bash abg_accounts.sh add <username> <tier>
#   sudo bash abg_accounts.sh list
#   sudo bash abg_accounts.sh remove <username>
#   sudo bash abg_accounts.sh create-pilot <name>
#   sudo bash abg_accounts.sh create-project <name>
#   sudo bash abg_accounts.sh setup-env <username>
#
# Example:
#   sudo bash abg_accounts.sh add jdoe compute
#   sudo bash abg_accounts.sh add msmith observer
#   sudo bash abg_accounts.sh add pi-garcia reviewer
#   sudo bash abg_accounts.sh create-pilot scrna-feasibility
#   sudo bash abg_accounts.sh create-project scrna-castleman

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

PROJECT_ROOT="$NUCLEUS_PROJECT_ROOT"
SHARED_NOTEBOOKS="$PROJECT_ROOT/notebooks"
SHARED_DATA="$WETSPRING_DIR/data"
PLASMIDBIN="$PLASMIDBIN_DIR"

ensure_groups() {
    for grp in abg-observer abg-compute abg-admin abg-reviewer; do
        getent group "$grp" > /dev/null 2>&1 || groupadd "$grp"
    done
    echo "Groups: abg-observer, abg-compute, abg-admin, abg-reviewer"
}

ensure_shared_workspace() {
    mkdir -p "$ABG_SHARED"/{commons,pilot,projects,data,templates,showcase,validation}
    chmod 2775 "$ABG_SHARED" "$ABG_SHARED"/{commons,pilot,projects,data,templates,showcase,validation}
    echo "Shared workspace: $ABG_SHARED"
}

setup_shared_dirs() {
    local user_home="$1"
    local tier="$2"
    local username="$3"

    mkdir -p "$user_home/notebooks"
    mkdir -p "$user_home/results"

    ln -sf "$SHARED_NOTEBOOKS/abg-wetspring-validation.ipynb" "$user_home/notebooks/" 2>/dev/null || true

    # Tier-aware shared workspace visibility:
    #   reviewer  -> sees only showcase/ (read-only, no access to internal work)
    #   everyone else -> sees full shared tree
    rm -f "$user_home/notebooks/shared" "$user_home/notebooks/showcase" 2>/dev/null || true
    if [[ "$tier" == "reviewer" ]]; then
        ln -sf "$ABG_SHARED/showcase" "$user_home/notebooks/showcase" 2>/dev/null || true
    else
        ln -sf "$ABG_SHARED" "$user_home/notebooks/shared" 2>/dev/null || true
    fi

    # Per-user scratch space for compute/admin (private workspace, 700)
    if [[ "$tier" == "compute" || "$tier" == "admin" ]]; then
        mkdir -p "$user_home/notebooks/scratch"
        chown "$username:$username" "$user_home/notebooks/scratch"
        chmod 700 "$user_home/notebooks/scratch"
        ln -sf "$PROJECT_ROOT/workloads" "$user_home/workloads" 2>/dev/null || true
        ln -sf "$SHARED_DATA" "$user_home/data" 2>/dev/null || true
    fi

    # Tier-appropriate welcome notebook
    local welcome_src="$ABG_SHARED/templates/welcome-${tier}.ipynb"
    if [[ -f "$welcome_src" ]]; then
        ln -sf "$welcome_src" "$user_home/notebooks/Welcome.ipynb" 2>/dev/null || true
    fi

    chown -R "$username:$username" "$user_home/notebooks" "$user_home/results"
}

setup_user_venv() {
    local username="$1"
    local user_home="/home/$username"
    local venv_dir="$user_home/.venv/bioinfo"
    local bioinfo_python="$ABG_SHARED/envs/bioinfo/bin/python3"

    if [[ ! -x "$bioinfo_python" ]]; then
        echo "ERROR: bioinfo python not found at $bioinfo_python" >&2
        return 1
    fi

    echo "  Creating per-user venv at $venv_dir ..."
    sudo -u "$username" "$bioinfo_python" -m venv --system-site-packages "$venv_dir"

    # Configure pip to use local wheelhouse (no internet needed)
    local pip_conf="$venv_dir/pip.conf"
    cat > "$pip_conf" << PIPEOF
[global]
find-links = $ABG_SHARED/wheelhouse
no-index = true
PIPEOF
    chown "$username:$username" "$pip_conf"

    # Register per-user kernel that runs from their venv
    echo "  Registering per-user Bioinfo kernel ..."
    sudo -u "$username" "$venv_dir/bin/python3" -m ipykernel install \
        --user --name bioinfo --display-name "ABG Bioinfo (Python 3.12)" \
        >/dev/null 2>&1

    echo "  venv ready: $venv_dir"
    echo "  User can now run: %pip install <package>"
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
export BEARDOG_PORT=$BEARDOG_PORT
export SONGBIRD_PORT=$SONGBIRD_PORT
export TOADSTOOL_PORT=$TOADSTOOL_PORT
export NESTGATE_PORT=$NESTGATE_PORT
export RHIZOCRYPT_PORT=$RHIZOCRYPT_PORT
export LOAMSPINE_PORT=$LOAMSPINE_PORT
export SWEETGRASS_PORT=$SWEETGRASS_PORT
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
        observer|compute|admin|reviewer) ;;
        *) echo "ERROR: Invalid tier '$tier'. Use: observer, compute, admin, reviewer" >&2; exit 1 ;;
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
    for grp in abg-observer abg-compute abg-admin abg-reviewer; do
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

    setup_shared_dirs "/home/$username" "$tier" "$username"
    write_user_env "/home/$username" "$tier" "$username"

    # Compute and admin tiers get a per-user venv for pip installs
    if [[ "$tier" == "compute" || "$tier" == "admin" ]]; then
        setup_user_venv "$username"
    fi

    echo ""
    echo "User: $username"
    echo "Tier: $tier"
    echo "Groups: $(id -nG "$username")"
    echo "Home: /home/$username"
    echo ""
    echo "Capabilities:"
    case "$tier" in
        observer)
            echo "  - View shared workspace (all directories)"
            echo "  - Health check primals"
            echo "  - Read provenance manifests"
            echo "  - NO workload submission"
            ;;
        compute)
            echo "  - All observer capabilities"
            echo "  - Submit workloads via ToadStool"
            echo "  - Write to commons/ and assigned projects/"
            echo "  - Access NCBI data and workload definitions"
            echo "  - View provenance chain (DAG, ledger, braid)"
            echo "  - Run validation notebooks"
            ;;
        admin)
            echo "  - All compute capabilities"
            echo "  - Query raw primal APIs"
            echo "  - Create projects, manage showcase/"
            echo "  - View deployment status"
            echo "  - Manage other ABG users"
            ;;
        reviewer)
            echo "  - View showcase/ only (polished work ready for review)"
            echo "  - Run Voila dashboards (server-side, no kernel access)"
            echo "  - NO workload submission"
            echo "  - NO code execution (kernels and terminals disabled)"
            echo "  - Intended for external PIs and HPC admins"
            ;;
    esac

    echo ""
    echo "Next: Add to JupyterHub config and restart"
    echo "  Then: sudo passwd $username"
}

create_pilot() {
    local pilot_name="$1"
    local pilot_dir="$ABG_SHARED/pilot/$pilot_name"

    if [[ -d "$pilot_dir" ]]; then
        echo "Pilot '$pilot_name' already exists at $pilot_dir"
        exit 1
    fi

    mkdir -p "$pilot_dir"/{notebooks,data}
    chmod 2775 "$pilot_dir" "$pilot_dir"/{notebooks,data}

    cat > "$pilot_dir/README.md" << PILOTEOF
# Pilot: $pilot_name

Created: $(date -Iseconds)
Status: active

## Hypothesis

_What are you testing? What question does this answer?_

## Decision Criteria

_What result would promote this to a project? What would kill it?_

## Timeline

_When should this be reviewed? (suggest: 2 weeks from creation)_

## Structure

- notebooks/ — Experiment notebooks
- data/ — Pilot-specific data (symlink to shared/data/ for large files)

## Lifecycle

commons/ (idea) -> **pilot/** (structured experiment) -> projects/ (formal) -> showcase/ (polished)

To promote to a project:
  sudo bash abg_accounts.sh create-project $pilot_name
  cp -r $pilot_dir/notebooks/* \$ABG_SHARED/projects/$pilot_name/notebooks/
PILOTEOF

    echo "Created pilot: $pilot_name"
    echo "  Path: $pilot_dir"
    echo "  Subdirectories: notebooks/, data/"
    echo ""
    echo "Lifecycle: commons (idea) -> pilot (experiment) -> projects (formal) -> showcase (polished)"
}

create_project() {
    local project_name="$1"
    local project_dir="$ABG_SHARED/projects/$project_name"

    if [[ -d "$project_dir" ]]; then
        echo "Project '$project_name' already exists at $project_dir"
        exit 1
    fi

    mkdir -p "$project_dir"/{notebooks,data,results}
    chmod 2775 "$project_dir" "$project_dir"/{notebooks,data,results}

    cat > "$project_dir/README.md" << PEOF
# $project_name

Created: $(date -Iseconds)

## Structure

- notebooks/ — Jupyter notebooks for this project
- data/ — Project-specific data (symlink to NestGate or shared/data/ for large files)
- results/ — Output from workload runs, provenance manifests

## Visibility

All ABG members can see this project. Compute and admin tiers can write to it.
PEOF

    echo "Created project: $project_name"
    echo "  Path: $project_dir"
    echo "  Subdirectories: notebooks/, data/, results/"
    echo ""
    echo "Copy a starter notebook:"
    echo "  cp $ABG_SHARED/templates/abg-wetspring-validation.ipynb $project_dir/notebooks/"
}

list_users() {
    echo "=== ABG Users ==="
    echo ""
    for tier in admin compute observer reviewer; do
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

    for grp in abg-observer abg-compute abg-admin abg-reviewer; do
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
    echo "Usage: sudo bash $0 {add|list|remove|create-pilot|create-project|setup-env|config} [args...]"
    exit 1
fi

ACTION="$1"; shift

ensure_groups
ensure_shared_workspace

case "$ACTION" in
    add)
        [[ $# -lt 2 ]] && { echo "Usage: sudo bash $0 add <username> <tier>"; exit 1; }
        add_user "$1" "$2"
        generate_jupyterhub_config
        ;;
    setup-env)
        [[ $# -lt 1 ]] && { echo "Usage: sudo bash $0 setup-env <username>"; exit 1; }
        if ! id "$1" &>/dev/null; then
            echo "ERROR: User '$1' does not exist" >&2
            exit 1
        fi
        setup_user_venv "$1"
        ;;
    list)
        list_users
        echo ""
        echo "=== Pilots ==="
        if [[ -d "$ABG_SHARED/pilot" ]]; then
            for d in "$ABG_SHARED/pilot"/*/; do
                [[ -d "$d" ]] && echo "  - $(basename "$d")"
            done
        fi
        echo ""
        echo "=== Projects ==="
        if [[ -d "$ABG_SHARED/projects" ]]; then
            for d in "$ABG_SHARED/projects"/*/; do
                [[ -d "$d" ]] && echo "  - $(basename "$d")"
            done
        fi
        ;;
    remove)
        [[ $# -lt 1 ]] && { echo "Usage: sudo bash $0 remove <username>"; exit 1; }
        remove_user "$1"
        generate_jupyterhub_config
        ;;
    create-pilot)
        [[ $# -lt 1 ]] && { echo "Usage: sudo bash $0 create-pilot <name>"; exit 1; }
        create_pilot "$1"
        ;;
    create-project)
        [[ $# -lt 1 ]] && { echo "Usage: sudo bash $0 create-project <name>"; exit 1; }
        create_project "$1"
        ;;
    config)
        generate_jupyterhub_config
        ;;
    *)
        echo "Unknown action: $ACTION"
        echo "Usage: sudo bash $0 {add|list|remove|create-pilot|create-project|setup-env|config} [args...]"
        exit 1
        ;;
esac
