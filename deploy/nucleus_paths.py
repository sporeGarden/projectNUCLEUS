"""Gate-agnostic path and port resolution for NUCLEUS Python scripts.

All values respect environment variables (set by nucleus_config.sh or
exported manually). Falls back to sane defaults rooted at $HOME.
"""

import os
from pathlib import Path

GATE_HOME = Path(os.environ.get("GATE_HOME", os.environ.get("HOME", "/home/nobody")))

ABG_SHARED = Path(os.environ.get("ABG_SHARED", GATE_HOME / "shared" / "abg"))
PUBLIC_ROOT = ABG_SHARED / "public"

JUPYTERHUB_DIR = Path(os.environ.get("JUPYTERHUB_DIR", GATE_HOME / "jupyterhub"))
JUPYTERHUB_CONFIG = Path(os.environ.get("JUPYTERHUB_CONFIG", JUPYTERHUB_DIR / "jupyterhub_config.py"))
JUPYTERHUB_SQLITE = JUPYTERHUB_DIR / "jupyterhub.sqlite"

JUPYTER_BIN = os.environ.get("JUPYTER_BIN", str(GATE_HOME / "miniforge3" / "envs" / "jupyterhub" / "bin"))

JUPYTERHUB_PORT = int(os.environ.get("JUPYTERHUB_PORT", "8000"))
HUB_URL = os.environ.get("LAB_URL", "https://lab.primals.eco")
VOILA_PORT = int(os.environ.get("VOILA_PORT", "8866"))
