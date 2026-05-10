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
    python3 pappusCast.py export    # one-shot static HTML generation
    python3 pappusCast.py status
    python3 pappusCast.py health

The static HTML export is the primary observer surface (served by
observer_server.py on port 8866). Medium and Heavy tiers both re-export
changed notebooks to keep the observer current.

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

from nucleus_paths import ABG_SHARED, PUBLIC_ROOT, JUPYTER_BIN, JUPYTERHUB_PORT

STATE_DIR = PUBLIC_ROOT / ".pappusCast"
QUARANTINE_DIR = STATE_DIR / "quarantine"
STATE_FILE = STATE_DIR / "last_publish.json"
CHANGELOG_FILE = STATE_DIR / "changelog.jsonl"
LOG_DIR = Path(__file__).resolve().parent / "tier_test_results"

VOILA_USER = "voila"
HUB_API = f"http://127.0.0.1:{JUPYTERHUB_PORT}/hub/api"

WATCHED_DIRS = ["commons", "showcase", "data", "pilot", "validation"]
SKIP_NAMES = {".ipynb_checkpoints", "__pycache__", ".pappusCast", "envs",
              "wheelhouse", "templates", "scratch", "projects"}

STATIC_HTML_DIR = STATE_DIR / "html_export"
THEME_CSS = Path(__file__).resolve().parent / "observer_theme.css"

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
    except (subprocess.SubprocessError, json.JSONDecodeError, OSError):
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
    except (subprocess.SubprocessError, json.JSONDecodeError, OSError) as e:
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
    except (urllib.error.URLError, json.JSONDecodeError, OSError, ValueError):
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


def _nbconvert_cmd() -> list[str]:
    """Return the nbconvert command prefix, using sudo only when needed."""
    import getpass
    if getpass.getuser() == VOILA_USER:
        return [f"{JUPYTER_BIN}/jupyter"]
    return ["sudo", "-u", VOILA_USER, f"{JUPYTER_BIN}/jupyter"]


def export_static_html(rel_path: str) -> tuple[bool, str]:
    """Execute a notebook and render to static HTML for the observer surface."""
    src = PUBLIC_ROOT / rel_path
    if src.suffix != ".ipynb" or not src.exists():
        return False, "not a notebook"

    out_dir = STATIC_HTML_DIR / Path(rel_path).parent
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / Path(rel_path).with_suffix(".html").name

    try:
        result = subprocess.run(
            _nbconvert_cmd() + ["nbconvert",
             "--execute", "--to", "html", "--no-input",
             "--ExecutePreprocessor.timeout=120",
             "--output", str(out_file), str(src)],
            capture_output=True, text=True, timeout=180,
        )
        if result.returncode != 0:
            return False, result.stderr.strip().split("\n")[-1][:200]

        _inject_nav(out_file)
        return True, str(out_file)
    except subprocess.TimeoutExpired:
        return False, "html export timed out"
    except (subprocess.SubprocessError, OSError) as e:
        return False, str(e)[:200]


def _nb_title(nb_path: Path) -> str:
    """Extract metadata.title from a notebook, falling back to stem."""
    try:
        nb = json.loads(nb_path.read_text())
        return nb.get("metadata", {}).get("title", "") or nb_path.stem
    except (json.JSONDecodeError, OSError):
        return nb_path.stem


def _build_nav_html(current_rel: str = "") -> str:
    """Build a top-navigation bar linking to all rendered notebooks."""
    sections: dict[str, list[tuple[str, str]]] = {}
    for html_file in sorted(STATIC_HTML_DIR.rglob("*.html")):
        if html_file.name == "index.html":
            continue
        rel = html_file.relative_to(STATIC_HTML_DIR)
        section = rel.parts[0] if len(rel.parts) > 1 else "root"
        nb_src = PUBLIC_ROOT / rel.with_suffix(".ipynb")
        title = _nb_title(nb_src) if nb_src.exists() else rel.stem
        depth = len(Path(current_rel).parts) - 1 if current_rel else 0
        prefix = "../" * depth if depth > 0 else ""
        sections.setdefault(section, []).append((f"{prefix}{rel}", title))

    nav_items = ['<a href="{}index.html" style="font-weight:bold">Home</a>'.format(
        "../" * (len(Path(current_rel).parts) - 1) if current_rel and len(Path(current_rel).parts) > 1 else "")]
    for section, links in sorted(sections.items()):
        for href, title in links:
            nav_items.append(f'<a href="{href}">{title}</a>')

    return (
        '<nav style="background:#1a1a2e;padding:10px 20px;display:flex;'
        'flex-wrap:wrap;gap:12px;font-family:system-ui,sans-serif;font-size:14px;'
        'border-bottom:2px solid #16213e">'
        + "".join(
            f'<span style="color:#e2e2e2">{item}</span>' for item in nav_items
        )
        + "</nav>"
    )


def _inject_nav(html_path: Path):
    """Insert theme CSS, navigation bar, and rewrite Voila links."""
    try:
        content = html_path.read_text(encoding="utf-8")
    except OSError:
        return
    rel = str(html_path.relative_to(STATIC_HTML_DIR))
    nav = _build_nav_html(rel)

    import re
    depth = len(Path(rel).parts) - 1
    prefix = "../" * depth if depth > 0 else ""
    content = re.sub(
        r'href="/voila/render/([^"]+)\.ipynb"',
        lambda m: f'href="{prefix}{m.group(1)}.html"',
        content,
    )

    theme_css = ""
    if THEME_CSS.exists():
        theme_css = f"\n<style>\n{THEME_CSS.read_text(encoding='utf-8')}\n</style>\n"

    if "</head>" in content:
        content = content.replace("</head>", f"{theme_css}</head>", 1)

    if "<body" in content:
        body_end = content.find(">", content.find("<body")) + 1
        content = content[:body_end] + "\n" + nav + "\n" + content[body_end:]
    html_path.write_text(content, encoding="utf-8")


def generate_index_page():
    """Build an index.html landing page for the static observer surface."""
    STATIC_HTML_DIR.mkdir(parents=True, exist_ok=True)

    welcome_nb = PUBLIC_ROOT / "Welcome.ipynb"
    welcome_html = ""
    if welcome_nb.exists():
        try:
            result = subprocess.run(
                _nbconvert_cmd() + ["nbconvert",
                 "--execute", "--to", "html", "--no-input",
                 "--ExecutePreprocessor.timeout=120",
                 "--output", str(STATIC_HTML_DIR / "Welcome.html"),
                 str(welcome_nb)],
                capture_output=True, text=True, timeout=180,
            )
            if result.returncode == 0:
                welcome_path = STATIC_HTML_DIR / "Welcome.html"
                raw = welcome_path.read_text(encoding="utf-8")
                start = raw.find("<body")
                end = raw.find("</body>")
                if start != -1 and end != -1:
                    body_start = raw.find(">", start) + 1
                    welcome_html = raw[body_start:end]
                    import re
                    welcome_html = re.sub(
                        r'href="/voila/render/([^"]+)\.ipynb"',
                        r'href="\1.html"',
                        welcome_html,
                    )
                    welcome_html = welcome_html.replace(
                        'href="/hub/login"', 'href="https://lab.primals.eco/hub/login"'
                    )
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, OSError):
            pass

    sections: dict[str, list[tuple[str, str]]] = {}
    for html_file in sorted(STATIC_HTML_DIR.rglob("*.html")):
        if html_file.name in ("index.html", "Welcome.html"):
            continue
        rel = html_file.relative_to(STATIC_HTML_DIR)
        section = rel.parts[0] if len(rel.parts) > 1 else "root"
        nb_src = PUBLIC_ROOT / rel.with_suffix(".ipynb")
        title = _nb_title(nb_src) if nb_src.exists() else rel.stem
        sections.setdefault(section, []).append((str(rel), title))

    section_html_parts = []
    section_order = ["showcase", "commons", "data", "pilot", "validation", "root"]
    for sec in section_order:
        links = sections.get(sec, [])
        if not links:
            continue
        label = sec.title() if sec != "root" else "Other"
        items = "".join(
            f'<li><a href="{href}" style="color:#a8dadc;text-decoration:none">{title}</a></li>'
            for href, title in links
        )
        section_html_parts.append(
            f'<div style="margin-bottom:24px">'
            f'<h3 style="color:#f1faee;border-bottom:1px solid #457b9d;'
            f'padding-bottom:6px">{label}</h3>'
            f'<ul style="list-style:none;padding-left:0;line-height:2">{items}</ul>'
            f'</div>'
        )

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    page = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>NUCLEUS Observer — lab.primals.eco</title>
<style>
  body {{ background:#0d1117; color:#e2e2e2; font-family:system-ui,sans-serif;
         margin:0; padding:0; }}
  .container {{ max-width:900px; margin:0 auto; padding:20px 24px; }}
  a {{ color:#a8dadc; }}
  a:hover {{ color:#f1faee; }}
  .welcome-embed {{ margin:20px 0; padding:20px; background:#161b22;
                    border-radius:8px; border:1px solid #30363d; }}
  .footer {{ margin-top:40px; padding-top:16px; border-top:1px solid #30363d;
             font-size:12px; color:#8b949e; }}
</style>
</head>
<body>
<div class="container">
<h1 style="color:#f1faee">NUCLEUS Observer</h1>
<p>Pre-rendered science notebooks from the active gate.
Read-only, no login required. Updated automatically by pappusCast.</p>
"""
    if welcome_html:
        page += f'<div class="welcome-embed">{welcome_html}</div>\n'

    page += '<h2 style="color:#f1faee">Notebooks</h2>\n'
    page += "".join(section_html_parts)

    page += f"""
<div class="footer">
<p>Last rendered: {ts} | Powered by pappusCast</p>
<p>Static pre-rendered HTML. No kernels launched. No compute cost per visit.</p>
</div>
</div>
</body>
</html>"""

    index_path = STATIC_HTML_DIR / "index.html"
    index_path.write_text(page, encoding="utf-8")
    log.info("Generated index.html (%d sections, %d notebooks)",
             len(section_html_parts),
             sum(len(v) for v in sections.values()))


def export_all_html(state: PublishState) -> int:
    """Export all public notebooks to static HTML and generate index page."""
    STATIC_HTML_DIR.mkdir(parents=True, exist_ok=True)
    html_count = 0
    current = scan_workspace()
    for rel in sorted(current.keys()):
        if Path(rel).suffix == ".ipynb":
            ok, detail = export_static_html(rel)
            if ok:
                html_count += 1
            else:
                log.warning("HTML export failed [%s]: %s", rel, detail)

    generate_index_page()
    welcome_html = STATIC_HTML_DIR / "Welcome.html"
    if welcome_html.exists():
        _inject_nav(welcome_html)
    log.info("Exported %d static HTML renders + index.html", html_count)
    return html_count


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
                ok_html, _ = export_static_html(rel)
                if ok_html:
                    log.debug("Medium: re-exported HTML for %s", rel)
        else:
            quarantine_file(rel, reason, state)

    if published:
        generate_index_page()

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

    report["html_exported"] = export_all_html(state)
    log.info("Heavy: exported %d static HTML renders", report["html_exported"])

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
        except (json.JSONDecodeError, OSError, KeyError, ValueError) as e:
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
                        choices=["daemon", "once", "status", "health", "export"],
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
    elif args.command == "export":
        state = PublishState.load()
        count = export_all_html(state)
        log.info("Export complete: %d notebooks rendered to %s", count, STATIC_HTML_DIR)
    elif args.command == "once":
        run_once(force=args.force)
    else:
        daemon_loop()


if __name__ == "__main__":
    main()
