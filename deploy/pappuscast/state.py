"""Data structures and workspace scanning for pappusCast."""

import hashlib
import json
import os
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

from .config import (
    ABG_SHARED, STATE_DIR, STATE_FILE, WATCHED_DIRS, SKIP_NAMES, log,
)


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
