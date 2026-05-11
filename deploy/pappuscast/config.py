"""Shared configuration for pappusCast modules."""

import logging
import os
from pathlib import Path

from nucleus_paths import ABG_SHARED, PUBLIC_ROOT, JUPYTER_BIN, JUPYTERHUB_PORT

STATE_DIR = PUBLIC_ROOT / ".pappusCast"
QUARANTINE_DIR = STATE_DIR / "quarantine"
STATE_FILE = STATE_DIR / "last_publish.json"
CHANGELOG_FILE = STATE_DIR / "changelog.jsonl"
LOG_DIR = Path(__file__).resolve().parent.parent / "tier_test_results"

VOILA_USER = "voila"
HUB_API = f"http://127.0.0.1:{JUPYTERHUB_PORT}/hub/api"

WATCHED_DIRS = ["commons", "showcase", "data", "pilot", "validation"]
SKIP_NAMES = {".ipynb_checkpoints", "__pycache__", ".pappusCast", "envs",
              "wheelhouse", "templates", "scratch", "projects"}

STATIC_HTML_DIR = STATE_DIR / "html_export"
THEME_CSS = Path(__file__).resolve().parent.parent / "observer_theme.css"

BASE_MINUTES = 5
MAX_MINUTES = 30
HEAVY_INTERVAL_S = 6 * 3600

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(message)s",
    datefmt="%Y-%m-%dT%H:%M:%S",
)
log = logging.getLogger("pappusCast")
