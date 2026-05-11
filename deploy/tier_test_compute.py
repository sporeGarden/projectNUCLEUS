#!/usr/bin/env python3
"""
Compute Tier Test — User Environment and Workspace Validation

Validates that a compute user (e.g. tamison) has the expected environment:
  - bioinfo venv exists with expected packages
  - Can write to personal workspace
  - Shared ABG workspace is read-only
  - Correct kernelspecs installed
  - Can execute all commons notebooks in their kernel

Usage:
    sudo python3 tier_test_compute.py [--user tamison] [--json]

Exit code: number of FAILs (0 = all pass)
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

PASS_COUNT = 0
FAIL_COUNT = 0
RESULTS = []

sys.path.insert(0, str(Path(__file__).resolve().parent))
from nucleus_paths import ABG_SHARED, JUPYTER_BIN, JUPYTERHUB_DIR


def report(status, test, detail):
    global PASS_COUNT, FAIL_COUNT
    if status == "PASS":
        PASS_COUNT += 1
    else:
        FAIL_COUNT += 1
    entry = {"status": status, "tier": "compute", "test": test, "detail": detail}
    RESULTS.append(entry)
    print(f"{status}|compute|{test}|{detail}")


def sudo_run(user, cmd, timeout=30):
    """Run a command as a given user."""
    return subprocess.run(
        ["sudo", "-u", user] + cmd,
        capture_output=True, text=True, timeout=timeout
    )


def test_venv(user):
    """Check that the bioinfo venv exists and key packages import."""
    home = Path(f"/home/{user}")
    venv = home / ".venv" / "bioinfo"

    if venv.exists():
        report("PASS", "venv:exists", f"{venv} exists")
    else:
        report("FAIL", "venv:exists", f"{venv} missing")
        return

    python = venv / "bin" / "python3"
    packages = ["numpy", "pandas", "scipy", "matplotlib", "Bio"]
    for pkg in packages:
        result = sudo_run(user, [str(python), "-c", f"import {pkg}; print({pkg}.__version__)"])
        if result.returncode == 0:
            report("PASS", f"pkg:{pkg}", f"v{result.stdout.strip()}")
        else:
            report("FAIL", f"pkg:{pkg}", f"Import failed: {result.stderr.strip()[:100]}")


def test_kernels(user):
    """Check that the user has expected kernelspecs."""
    home = Path(f"/home/{user}")
    kernel_dir = home / ".local" / "share" / "jupyter" / "kernels"

    expected = ["bioinfo"]
    for k in expected:
        kdir = kernel_dir / k
        if kdir.exists():
            try:
                with open(kdir / "kernel.json") as f:
                    spec = json.load(f)
                report("PASS", f"kernel:{k}", f"display_name={spec.get('display_name','?')}")
            except (json.JSONDecodeError, OSError) as e:
                report("FAIL", f"kernel:{k}", f"kernel.json unreadable: {e}")
        else:
            report("FAIL", f"kernel:{k}", f"{kdir} missing")


def test_workspace(user):
    """Verify personal workspace writability and shared ABG read-only."""
    home = Path(f"/home/{user}")

    personal_dirs = ["notebooks", "results"]
    for d in personal_dirs:
        target = home / d
        if target.exists():
            test_file = target / f".tier_test_{os.getpid()}"
            result = sudo_run(user, ["touch", str(test_file)])
            if result.returncode == 0:
                report("PASS", f"write:{d}", f"Writable at {target}")
                sudo_run(user, ["rm", "-f", str(test_file)])
            else:
                report("FAIL", f"write:{d}", f"Cannot write: {result.stderr.strip()[:100]}")
        else:
            report("FAIL", f"write:{d}", f"{target} does not exist")

    shared_test = ABG_SHARED / f".tier_test_{os.getpid()}"
    result = sudo_run(user, ["touch", str(shared_test)])
    if result.returncode != 0:
        report("PASS", "shared:readonly", "Cannot write to shared ABG (correct)")
    else:
        report("FAIL", "shared:readonly", "Wrote to shared ABG — should be read-only")
        sudo_run(user, ["rm", "-f", str(shared_test)])


def test_notebook_execution(user):
    """Execute commons notebooks as the compute user using system jupyter."""
    commons = ABG_SHARED / "commons"
    for nb_path in sorted(commons.glob("*.ipynb")):
        if ".ipynb_checkpoints" in str(nb_path):
            continue
        rel = nb_path.name
        test_name = f"exec:{rel}"

        timeout_s = 120 if "security" in rel.lower() or "primal" in rel.lower() else 60
        cmd = [
            "sudo", "-u", user,
            f"{JUPYTER_BIN}/jupyter", "nbconvert",
            "--execute",
            "--to", "notebook",
            "--stdout",
            f"--ExecutePreprocessor.timeout={timeout_s}",
            "--ExecutePreprocessor.kernel_name=python3",
            str(nb_path),
        ]
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout_s + 30)
            if result.returncode == 0:
                nb_out = json.loads(result.stdout)
                errors = []
                for i, cell in enumerate(nb_out.get("cells", [])):
                    if cell.get("cell_type") != "code":
                        continue
                    for output in cell.get("outputs", []):
                        if output.get("output_type") == "error":
                            ename = output.get("ename", "Unknown")
                            evalue = output.get("evalue", "")[:120]
                            errors.append(f"cell[{i}]: {ename}: {evalue}")
                if errors:
                    report("FAIL", test_name, "; ".join(errors[:3]))
                else:
                    report("PASS", test_name, f"Executed cleanly ({len(nb_out.get('cells',[]))} cells)")
            else:
                snippet = result.stderr.strip().split("\n")[-1][:150]
                report("FAIL", test_name, f"exit={result.returncode}: {snippet}")
        except subprocess.TimeoutExpired:
            report("FAIL", test_name, "Timed out after 90s")
        except (subprocess.SubprocessError, json.JSONDecodeError, OSError) as e:
            report("FAIL", test_name, str(e)[:150])


def test_env_vars(user):
    """Check NUCLEUS_TIER — set by JupyterHub spawner, not login shell.
    We verify the JupyterHub config declares it rather than testing the shell."""
    jh_config = JUPYTERHUB_DIR / "jupyterhub_config.py"
    if jh_config.exists():
        import re
        text = jh_config.read_text()
        if "NUCLEUS_TIER" in text:
            report("PASS", "env:NUCLEUS_TIER", "Set in JupyterHub spawner config")
        else:
            report("FAIL", "env:NUCLEUS_TIER", "Not declared in JupyterHub config")
    else:
        report("FAIL", "env:NUCLEUS_TIER", "JupyterHub config not found")


def main():
    parser = argparse.ArgumentParser(description="Compute Tier Test")
    parser.add_argument("--user", default="tamison", help="Compute user to test")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    user = args.user
    print("═══════════════════════════════════════════════════")
    print(f"  Compute Tier Test — User: {user}")
    print(f"  Date: {time.strftime('%Y-%m-%dT%H:%M:%S%z')}")
    print("═══════════════════════════════════════════════════")

    print("\n── Virtual Environment ──")
    test_venv(user)

    print("\n── Kernelspecs ──")
    test_kernels(user)

    print("\n── Workspace Permissions ──")
    test_workspace(user)

    print("\n── Environment Variables ──")
    test_env_vars(user)

    print("\n── Notebook Execution (as compute user) ──")
    test_notebook_execution(user)

    print()
    print("═══════════════════════════════════════════════════")
    print(f"  Results: {PASS_COUNT} PASS, {FAIL_COUNT} FAIL")
    print("═══════════════════════════════════════════════════")

    if args.json:
        print(json.dumps({"pass": PASS_COUNT, "fail": FAIL_COUNT, "results": RESULTS}, indent=2))

    sys.exit(min(FAIL_COUNT, 125))


if __name__ == "__main__":
    main()
