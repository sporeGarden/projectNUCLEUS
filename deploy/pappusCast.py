#!/usr/bin/env python3
"""
pappusCast — Tiered auto-propagation from shared workspace to observer surface.

Named for the dandelion pappus: the parachute structure that carries seeds to
new ground. Each validated notebook is a seed; pappusCast disperses them from
the compute workspace to the public observer surface.

Self-pollination (auto-validation within workspace) and cross-pollination
(propagation to public surface) mirror the dandelion's reproductive strategy.

Three validation tiers run at different cadences:

  Light  (on-change):  JSON valid, kernel available, title present
  Medium (periodic):   Light + execute as voila user, check for cell errors
  Heavy  (~6h):        Medium + diff, changelog, full regression

Temporal lag scales with active JupyterHub users to avoid overloading during
collaborative sessions.

Usage:
    python3 pappusCast.py [--once] [--force] [--json]
    python3 pappusCast.py status
    python3 pappusCast.py health

Evolution: Python (now) -> Rust binary -> pappusCast primal
"""

import argparse
import hashlib
import json
import logging
import os
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

ABG_SHARED = Path(os.environ.get("ABG_SHARED", "/home/irongate/shared/abg"))
PUBLIC_ROOT = ABG_SHARED / "public"
STATE_DIR = PUBLIC_ROOT / ".pappusCast"
QUARANTINE_DIR = STATE_DIR / "quarantine"
STATE_FILE = STATE_DIR / "last_publish.json"
CHANGELOG_FILE = STATE_DIR / "changelog.jsonl"
LOG_DIR = Path(__file__).resolve().parent / "tier_test_results"

JUPYTER_BIN = "/home/irongate/miniforge3/envs/jupyterhub/bin"
VOILA_USER = "voila"
HUB_API = "http://127.0.0.1:8000/hub/api"

WATCHED_DIRS = ["commons", "showcase", "data", "pilot", "validation"]
SKIP_NAMES = {".ipynb_checkpoints", "__pycache__", ".pappusCast", "envs",
              "wheelhouse", "templates", "scratch", "projects"}

STATIC_HTML_DIR = STATE_DIR / "html_export"

BASE_MINUTES = 5
MAX_MINUTES = 30
HEAVY_INTERVAL_S = 6 * 3600

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(message)s",
    datefmt="%Y-%m-%dT%H:%M:%S",
)
log = logging.getLogger("pappusCast")


# ---------------------------------------------------------------------------
# Data structures (JSON-serializable, serde-ready)
# ---------------------------------------------------------------------------

@dataclass
class FileState:
    rel_path: str
    sha256: str
    mtime: float
    light_ok: bool = False
    medium_ok: bool = False
    medium_ts: float = 0.0
    heavy_ts: float = 0.0
    quarantined: bool = False
    quarantine_reason: str = ""


@dataclass
class PublishState:
    files: dict = field(default_factory=dict)
    last_light_ts: float = 0.0
    last_medium_ts: float = 0.0
    last_heavy_ts: float = 0.0
    publish_count: int = 0

    def save(self):
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        data = {
            "last_light_ts": self.last_light_ts,
            "last_medium_ts": self.last_medium_ts,
            "last_heavy_ts": self.last_heavy_ts,
            "publish_count": self.publish_count,
            "files": {k: asdict(v) for k, v in self.files.items()},
        }
        tmp = STATE_FILE.with_suffix(".tmp")
        tmp.write_text(json.dumps(data, indent=2))
        tmp.rename(STATE_FILE)

    @classmethod
    def load(cls):
        if not STATE_FILE.exists():
            return cls()
        try:
            data = json.loads(STATE_FILE.read_text())
            state = cls(
                last_light_ts=data.get("last_light_ts", 0),
                last_medium_ts=data.get("last_medium_ts", 0),
                last_heavy_ts=data.get("last_heavy_ts", 0),
                publish_count=data.get("publish_count", 0),
            )
            for k, v in data.get("files", {}).items():
                state.files[k] = FileState(**v)
            return state
        except (json.JSONDecodeError, TypeError, KeyError) as e:
            log.warning("Corrupt state file, starting fresh: %s", e)
            return cls()


# ---------------------------------------------------------------------------
# Watcher — detect changed files via mtime + sha256
# ---------------------------------------------------------------------------

def file_sha256(path: Path) -> str:
    h = hashlib.sha256()
    try:
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                h.update(chunk)
    except OSError:
        return ""
    return h.hexdigest()


def scan_workspace() -> dict[str, tuple[Path, float]]:
    """Walk watched dirs, return {rel_path: (abs_path, mtime)}."""
    found = {}
    for d in WATCHED_DIRS:
        src = ABG_SHARED / d
        if not src.exists():
            continue
        for root, dirs, files in os.walk(src, followlinks=True):
            dirs[:] = [x for x in dirs if x not in SKIP_NAMES]
            for fname in files:
                if fname.startswith("."):
                    continue
                fpath = Path(root) / fname
                rel = str(fpath.relative_to(ABG_SHARED))
                try:
                    found[rel] = (fpath, fpath.stat().st_mtime)
                except OSError:
                    continue
    return found


def detect_changes(state: PublishState) -> list[str]:
    """Return list of rel_paths that changed since last recorded state."""
    current = scan_workspace()
    changed = []
    for rel, (abs_path, mtime) in current.items():
        prev = state.files.get(rel)
        if prev is None or prev.mtime < mtime:
            new_hash = file_sha256(abs_path)
            if prev is None or prev.sha256 != new_hash:
                changed.append(rel)
    return changed


# ---------------------------------------------------------------------------
# Validators
# ---------------------------------------------------------------------------

def validate_light(abs_path: Path) -> tuple[bool, str]:
    """Fast structural checks for notebooks. Passthrough for non-notebooks."""
    if not abs_path.suffix == ".ipynb":
        return True, "non-notebook passthrough"
    try:
        with open(abs_path) as f:
            nb = json.load(f)
    except (json.JSONDecodeError, OSError) as e:
        return False, f"invalid JSON: {e}"

    kernel = nb.get("metadata", {}).get("kernelspec", {}).get("name", "")
    if not kernel:
        return False, "no kernelspec defined"

    avail = _voila_kernels()
    if kernel not in avail:
        return False, f"kernel '{kernel}' not available to voila (has: {', '.join(avail)})"

    title = nb.get("metadata", {}).get("title", "")
    if not title:
        return False, "missing metadata.title"

    return True, "ok"


_KERNEL_CACHE: Optional[set] = None
_KERNEL_CACHE_TS: float = 0


def _voila_kernels() -> set:
    global _KERNEL_CACHE, _KERNEL_CACHE_TS
    if _KERNEL_CACHE and time.time() - _KERNEL_CACHE_TS < 300:
        return _KERNEL_CACHE
    try:
        result = subprocess.run(
            ["sudo", "-u", VOILA_USER, f"{JUPYTER_BIN}/jupyter",
             "kernelspec", "list", "--json"],
            capture_output=True, text=True, timeout=15,
        )
        specs = json.loads(result.stdout).get("kernelspecs", {})
        _KERNEL_CACHE = set(specs.keys())
    except Exception:
        _KERNEL_CACHE = {"python3"}
    _KERNEL_CACHE_TS = time.time()
    return _KERNEL_CACHE


def validate_medium(abs_path: Path) -> tuple[bool, str]:
    """Execute notebook as voila user, check for cell errors."""
    if abs_path.suffix != ".ipynb":
        return True, "non-notebook passthrough"

    ok, reason = validate_light(abs_path)
    if not ok:
        return False, f"light failed: {reason}"

    try:
        result = subprocess.run(
            ["sudo", "-u", VOILA_USER, f"{JUPYTER_BIN}/jupyter", "nbconvert",
             "--execute", "--to", "notebook", "--stdout",
             "--ExecutePreprocessor.timeout=120", str(abs_path)],
            capture_output=True, text=True, timeout=150,
        )
        if result.returncode != 0:
            last_line = result.stderr.strip().split("\n")[-1][:200]
            return False, f"execution failed: {last_line}"

        nb_out = json.loads(result.stdout)
        for i, cell in enumerate(nb_out.get("cells", [])):
            if cell.get("cell_type") != "code":
                continue
            for output in cell.get("outputs", []):
                if output.get("output_type") == "error":
                    ename = output.get("ename", "Unknown")
                    evalue = output.get("evalue", "")[:120]
                    return False, f"cell[{i}]: {ename}: {evalue}"

        return True, f"executed cleanly ({len(nb_out.get('cells', []))} cells)"
    except subprocess.TimeoutExpired:
        return False, "execution timed out (150s)"
    except Exception as e:
        return False, str(e)[:200]


# ---------------------------------------------------------------------------
# Rate limiter — adaptive temporal lag
# ---------------------------------------------------------------------------

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
    except Exception:
        return 1


def compute_interval() -> float:
    """Compute publish interval in seconds based on active users."""
    active = get_active_users()
    minutes = min(BASE_MINUTES * max(1, active), MAX_MINUTES)
    log.debug("Active users: %d, publish interval: %d min", active, minutes)
    return minutes * 60


# ---------------------------------------------------------------------------
# Publisher
# ---------------------------------------------------------------------------

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


def export_static_html(rel_path: str) -> tuple[bool, str]:
    """Render a notebook to static HTML for always-on observer fallback."""
    src = PUBLIC_ROOT / rel_path
    if src.suffix != ".ipynb" or not src.exists():
        return False, "not a notebook"

    out_dir = STATIC_HTML_DIR / Path(rel_path).parent
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / Path(rel_path).with_suffix(".html").name

    try:
        result = subprocess.run(
            ["sudo", "-u", VOILA_USER, f"{JUPYTER_BIN}/jupyter", "nbconvert",
             "--to", "html", "--no-input",
             "--output", str(out_file), str(src)],
            capture_output=True, text=True, timeout=180,
        )
        if result.returncode != 0:
            return False, result.stderr.strip().split("\n")[-1][:200]
        return True, str(out_file)
    except subprocess.TimeoutExpired:
        return False, "html export timed out"
    except Exception as e:
        return False, str(e)[:200]


def append_changelog(entries: list[dict]):
    """Append heavy-tier changelog entries."""
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    with open(CHANGELOG_FILE, "a") as f:
        for e in entries:
            f.write(json.dumps(e) + "\n")


# ---------------------------------------------------------------------------
# Tier orchestrators
# ---------------------------------------------------------------------------

def run_light(state: PublishState, changed: list[str]) -> int:
    """Validate and publish changed files at light tier."""
    published = 0
    for rel in changed:
        src = ABG_SHARED / rel
        if not src.exists():
            publish_file(rel, state, "light")
            continue

        ok, reason = validate_light(src)
        if ok:
            if publish_file(rel, state, "light"):
                published += 1
        else:
            log.info("Light failed [%s]: %s", rel, reason)
            fs = state.files.get(rel)
            if fs is None:
                state.files[rel] = FileState(
                    rel_path=rel, sha256=file_sha256(src),
                    mtime=src.stat().st_mtime,
                )
            # Still publish non-notebooks that fail light (they have no kernel/title)
            if src.suffix != ".ipynb":
                if publish_file(rel, state, "light"):
                    published += 1

    state.last_light_ts = time.time()
    return published


def run_medium(state: PublishState) -> int:
    """Run medium validation on files that changed since last medium pass."""
    now = time.time()
    candidates = []
    for rel, fs in state.files.items():
        if fs.mtime > fs.medium_ts or not fs.medium_ok:
            src = ABG_SHARED / rel
            if src.exists() and src.suffix == ".ipynb":
                candidates.append(rel)

    if not candidates:
        log.info("Medium: no candidates")
        state.last_medium_ts = now
        return 0

    log.info("Medium: validating %d notebooks", len(candidates))
    published = 0
    for rel in candidates:
        src = ABG_SHARED / rel
        ok, reason = validate_medium(src)
        if ok:
            if publish_file(rel, state, "medium"):
                published += 1
        else:
            quarantine_file(rel, reason, state)

    state.last_medium_ts = now
    return published


def run_heavy(state: PublishState) -> dict:
    """Full regression + changelog generation."""
    now = time.time()
    log.info("Heavy: full regression of all public notebooks")

    current = scan_workspace()
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "total_files": len(current),
        "validated": 0,
        "failed": 0,
        "published": 0,
        "quarantined": [],
        "changes": [],
    }

    for rel, (abs_path, mtime) in sorted(current.items()):
        if abs_path.suffix != ".ipynb":
            publish_file(rel, state, "heavy")
            report["validated"] += 1
            continue

        ok, reason = validate_medium(abs_path)
        if ok:
            prev = state.files.get(rel)
            prev_hash = prev.sha256 if prev else ""
            new_hash = file_sha256(abs_path)

            if publish_file(rel, state, "heavy"):
                report["published"] += 1
                if prev_hash and prev_hash != new_hash:
                    report["changes"].append({
                        "file": rel,
                        "action": "updated",
                        "prev_hash": prev_hash[:12],
                        "new_hash": new_hash[:12],
                    })
                elif not prev_hash:
                    report["changes"].append({
                        "file": rel,
                        "action": "added",
                    })
            report["validated"] += 1
        else:
            quarantine_file(rel, reason, state)
            report["failed"] += 1
            report["quarantined"].append({"file": rel, "reason": reason})

    # Detect deletions
    published_files = set(state.files.keys())
    current_files = set(current.keys())
    for gone in published_files - current_files:
        dst = PUBLIC_ROOT / gone
        if dst.exists():
            dst.unlink()
            log.info("Removed stale: %s", gone)
            report["changes"].append({"file": gone, "action": "removed"})
        state.files.pop(gone, None)

    # Static HTML export for always-on observer fallback
    STATIC_HTML_DIR.mkdir(parents=True, exist_ok=True)
    html_count = 0
    for rel in sorted(current.keys()):
        if Path(rel).suffix == ".ipynb":
            ok, detail = export_static_html(rel)
            if ok:
                html_count += 1
    report["html_exported"] = html_count
    log.info("Heavy: exported %d static HTML renders", html_count)

    state.last_heavy_ts = now
    append_changelog([report])

    log.info("Heavy complete: %d validated, %d failed, %d published, %d changes",
             report["validated"], report["failed"], report["published"],
             len(report["changes"]))
    return report


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

def daemon_loop():
    """Main event loop: watch, validate, publish on adaptive schedule."""
    state = PublishState.load()
    log.info("pappusCast started (state: %d tracked files, %d publishes)",
             len(state.files), state.publish_count)

    last_publish = time.time()
    last_heavy = state.last_heavy_ts or 0

    while True:
        try:
            interval = compute_interval()
            changed = detect_changes(state)

            if changed:
                log.info("Detected %d changed files", len(changed))
                published = run_light(state, changed)
                if published:
                    state.publish_count += published
                    state.save()

            now = time.time()
            if now - last_publish >= interval:
                published = run_medium(state)
                if published:
                    state.publish_count += published
                state.save()
                last_publish = now

            if now - last_heavy >= HEAVY_INTERVAL_S:
                run_heavy(state)
                state.save()
                last_heavy = now

            time.sleep(30)

        except KeyboardInterrupt:
            log.info("Shutting down")
            state.save()
            break
        except Exception as e:
            log.error("Loop error: %s", e, exc_info=True)
            time.sleep(60)


def run_once(force: bool = False):
    """Single-shot: run all three tiers and exit."""
    state = PublishState.load()
    changed = detect_changes(state)

    if force:
        changed = list(scan_workspace().keys())
        log.info("Force mode: processing all %d files", len(changed))

    if changed:
        run_light(state, changed)

    run_medium(state)
    run_heavy(state)

    state.save()
    log.info("Single-shot complete: %d files tracked", len(state.files))


def show_status():
    """Print current pappusCast state as structured output."""
    state = PublishState.load()
    quarantined = [k for k, v in state.files.items() if v.quarantined]
    stale = [k for k, v in state.files.items()
             if v.mtime > v.medium_ts and not v.quarantined]

    def ts(t):
        return datetime.fromtimestamp(t, tz=timezone.utc).isoformat() if t else "never"

    status = {
        "tracked_files": len(state.files),
        "publish_count": state.publish_count,
        "quarantined": len(quarantined),
        "stale_medium": len(stale),
        "last_light": ts(state.last_light_ts),
        "last_medium": ts(state.last_medium_ts),
        "last_heavy": ts(state.last_heavy_ts),
        "active_users": get_active_users(),
        "current_interval_min": min(BASE_MINUTES * max(1, get_active_users()),
                                    MAX_MINUTES),
    }
    if quarantined:
        status["quarantined_files"] = [
            {"file": k, "reason": state.files[k].quarantine_reason}
            for k in quarantined
        ]
    print(json.dumps(status, indent=2))


def show_health():
    """Quick health check for systemd / monitoring integration."""
    checks = []
    ok = True

    if STATE_DIR.exists():
        checks.append({"check": "state_dir", "status": "ok"})
    else:
        checks.append({"check": "state_dir", "status": "fail", "detail": "missing"})
        ok = False

    if STATE_FILE.exists():
        try:
            state = PublishState.load()
            age_s = time.time() - state.last_light_ts if state.last_light_ts else -1
            if age_s > 600:
                checks.append({"check": "last_light", "status": "warn",
                                "detail": f"last light {age_s:.0f}s ago"})
            else:
                checks.append({"check": "last_light", "status": "ok",
                                "detail": f"{age_s:.0f}s ago"})
        except Exception as e:
            checks.append({"check": "state_file", "status": "fail", "detail": str(e)})
            ok = False
    else:
        checks.append({"check": "state_file", "status": "warn", "detail": "no state yet"})

    pub_count = sum(1 for _ in PUBLIC_ROOT.rglob("*.ipynb")
                    if ".pappusCast" not in str(_))
    checks.append({"check": "public_notebooks", "status": "ok",
                    "detail": f"{pub_count} notebooks"})

    result = {"healthy": ok, "checks": checks}
    print(json.dumps(result, indent=2))
    sys.exit(0 if ok else 1)


def main():
    parser = argparse.ArgumentParser(description="pappusCast — dandelion-seed propagation to observer surface")
    parser.add_argument("command", nargs="?", default="daemon",
                        choices=["daemon", "once", "status", "health"],
                        help="Command to run")
    parser.add_argument("--force", action="store_true",
                        help="Force-publish all files (with 'once')")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    STATE_DIR.mkdir(parents=True, exist_ok=True)
    QUARANTINE_DIR.mkdir(parents=True, exist_ok=True)

    if args.command == "status":
        show_status()
    elif args.command == "health":
        show_health()
    elif args.command == "once":
        run_once(force=args.force)
    else:
        daemon_loop()


if __name__ == "__main__":
    main()
