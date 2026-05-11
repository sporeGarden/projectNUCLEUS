"""Snapshot publisher and quarantine for pappusCast."""

import json
import os
import shutil
import subprocess
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

from .config import (
    ABG_SHARED, PUBLIC_ROOT, JUPYTER_BIN, VOILA_USER,
    QUARANTINE_DIR, HUB_API, STATE_DIR,
    BASE_MINUTES, MAX_MINUTES, log,
)
from .state import FileState, PublishState, file_sha256


def publish_file(rel_path: str, state: PublishState, tier: str) -> bool:
    """Copy a validated file from shared workspace to public/ snapshot."""
    src = ABG_SHARED / rel_path
    dst = PUBLIC_ROOT / rel_path

    if not src.exists():
        prev = state.files.pop(rel_path, None)
        if prev and dst.exists():
            dst.unlink()
            log.info("Removed deleted file: %s", rel_path)
        return True

    dst.parent.mkdir(parents=True, exist_ok=True)
    try:
        if dst.exists():
            dst.chmod(0o644)
            dst.unlink()
        shutil.copy2(str(src), str(dst))
        os.chmod(str(dst), 0o444)
    except OSError as e:
        log.error("Failed to copy %s: %s", rel_path, e)
        return False

    if dst.suffix == ".ipynb":
        subprocess.run(
            ["sudo", "-u", VOILA_USER, f"{JUPYTER_BIN}/jupyter",
             "trust", str(dst)],
            capture_output=True, timeout=15,
        )

    now = time.time()
    sha = file_sha256(src)
    fs = state.files.get(rel_path)
    if fs is None:
        fs = FileState(rel_path=rel_path, sha256=sha, mtime=src.stat().st_mtime)
        state.files[rel_path] = fs
    fs.sha256 = sha
    fs.mtime = src.stat().st_mtime
    fs.quarantined = False
    fs.quarantine_reason = ""

    if tier == "light":
        fs.light_ok = True
    elif tier == "medium":
        fs.light_ok = True
        fs.medium_ok = True
        fs.medium_ts = now
    elif tier == "heavy":
        fs.light_ok = True
        fs.medium_ok = True
        fs.medium_ts = now
        fs.heavy_ts = now

    log.info("Published [%s]: %s", tier, rel_path)
    return True


def quarantine_file(rel_path: str, reason: str, state: PublishState):
    """Record a file that failed validation without publishing it."""
    QUARANTINE_DIR.mkdir(parents=True, exist_ok=True)
    entry = {
        "rel_path": rel_path,
        "reason": reason,
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }
    qfile = QUARANTINE_DIR / f"{rel_path.replace('/', '_')}.json"
    qfile.write_text(json.dumps(entry, indent=2))

    fs = state.files.get(rel_path)
    if fs:
        fs.quarantined = True
        fs.quarantine_reason = reason
    else:
        state.files[rel_path] = FileState(
            rel_path=rel_path, sha256="", mtime=0,
            quarantined=True, quarantine_reason=reason,
        )
    log.warning("Quarantined: %s — %s", rel_path, reason)


def get_active_users() -> int:
    """Query JupyterHub API for count of running single-user servers."""
    token = os.environ.get("JUPYTERHUB_API_TOKEN", "")
    if not token:
        token_file = STATE_DIR / "hub_token"
        if token_file.exists():
            token = token_file.read_text().strip()
    if not token:
        return 1

    try:
        req = urllib.request.Request(
            f"{HUB_API}/users",
            headers={"Authorization": f"token {token}"},
        )
        with urllib.request.urlopen(req, timeout=5) as resp:
            users = json.loads(resp.read().decode())
            return sum(
                1 for u in users
                if u.get("servers", {}).get("", {}).get("ready")
            )
    except (urllib.error.URLError, json.JSONDecodeError, OSError, ValueError):
        return 1


def compute_interval() -> float:
    """Compute publish interval in seconds based on active users."""
    active = get_active_users()
    minutes = min(BASE_MINUTES * max(1, active), MAX_MINUTES)
    log.debug("Active users: %d, publish interval: %d min", active, minutes)
    return minutes * 60
