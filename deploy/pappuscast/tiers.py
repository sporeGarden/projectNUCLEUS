"""Light and medium validation tiers for pappusCast."""

import json
import subprocess
import time
from pathlib import Path
from typing import Optional

from .config import VOILA_USER, JUPYTER_BIN, log


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
