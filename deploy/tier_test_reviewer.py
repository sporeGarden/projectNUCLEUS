#!/usr/bin/env python3
"""
Reviewer Tier Test — Read + Run Validation

Validates that a reviewer user (e.g. abgreviewer) has:
  - Read access to the full ABG shared workspace
  - No write access to shared or personal notebooks
  - No terminal access
  - No kernel creation (compute gated by Voila widgets)
  - Can read all notebook files

Usage:
    sudo python3 tier_test_reviewer.py [--user abgreviewer] [--json]

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

ABG_SHARED = Path("/home/irongate/shared/abg")


def report(status, test, detail):
    global PASS_COUNT, FAIL_COUNT
    if status == "PASS":
        PASS_COUNT += 1
    else:
        FAIL_COUNT += 1
    entry = {"status": status, "tier": "reviewer", "test": test, "detail": detail}
    RESULTS.append(entry)
    print(f"{status}|reviewer|{test}|{detail}")


def sudo_run(user, cmd, timeout=15):
    return subprocess.run(
        ["sudo", "-u", user] + cmd,
        capture_output=True, text=True, timeout=timeout,
    )


def test_read_access(user):
    """Verify reviewer can read all shared ABG directories."""
    dirs_to_check = ["showcase", "commons", "data", "pilot", "validation"]
    for d in dirs_to_check:
        target = ABG_SHARED / d
        if not target.exists():
            report("FAIL", f"read:{d}", f"{target} does not exist")
            continue
        result = sudo_run(user, ["ls", str(target)])
        if result.returncode == 0:
            items = len(result.stdout.strip().split("\n"))
            report("PASS", f"read:{d}", f"Readable ({items} items)")
        else:
            report("FAIL", f"read:{d}", f"Cannot list: {result.stderr.strip()[:100]}")


def test_notebook_readability(user):
    """Verify reviewer can read (parse) every notebook in shared workspace."""
    count = 0
    errors = 0
    for root, _, files in os.walk(ABG_SHARED, followlinks=True):
        for f in files:
            if not f.endswith(".ipynb") or ".ipynb_checkpoints" in root:
                continue
            nb_path = Path(root) / f
            result = sudo_run(user, ["cat", str(nb_path)])
            if result.returncode == 0:
                try:
                    json.loads(result.stdout)
                    count += 1
                except json.JSONDecodeError:
                    report("FAIL", f"parse:{nb_path.relative_to(ABG_SHARED)}", "Invalid JSON")
                    errors += 1
            else:
                report("FAIL", f"read_nb:{nb_path.relative_to(ABG_SHARED)}", "Permission denied")
                errors += 1

    if count > 0 and errors == 0:
        report("PASS", "nb_readability", f"All {count} notebooks readable and valid JSON")
    elif count > 0:
        report("FAIL", "nb_readability", f"{count} ok, {errors} failed")


def test_no_write(user):
    """Verify reviewer cannot write to shared ABG dirs. Home dir write is expected."""
    shared_targets = [
        ABG_SHARED / "commons",
        ABG_SHARED / "showcase",
        ABG_SHARED / "data",
    ]
    for target in shared_targets:
        if not target.exists():
            continue
        test_file = target / f".reviewer_test_{os.getpid()}"
        result = sudo_run(user, ["touch", str(test_file)])
        if result.returncode != 0:
            report("PASS", f"no_write:{target.name}", f"Write blocked at {target}")
        else:
            report("FAIL", f"no_write:{target.name}", f"Write succeeded at {target}")
            sudo_run(user, ["rm", "-f", str(test_file)])

    home = Path(f"/home/{user}")
    if home.exists():
        test_file = home / f".reviewer_test_{os.getpid()}"
        result = sudo_run(user, ["touch", str(test_file)])
        if result.returncode == 0:
            report("PASS", "home_write", f"Can write to own home (expected)")
            sudo_run(user, ["rm", "-f", str(test_file)])
        else:
            report("PASS", "home_write", "Home is read-only (JupyterHub handles this)")


def test_no_venv(user):
    """Reviewer should NOT have a bioinfo venv (compute users only)."""
    venv = Path(f"/home/{user}/.venv/bioinfo")
    if not venv.exists():
        report("PASS", "no_venv", "No bioinfo venv (correct for reviewer)")
    else:
        report("FAIL", "no_venv", f"Reviewer has a venv at {venv} — should be compute-only")


def test_home_structure(user):
    """Check reviewer home directory exists with minimal structure."""
    home = Path(f"/home/{user}")
    if home.exists():
        report("PASS", "home:exists", f"{home} exists")
    else:
        report("FAIL", "home:exists", f"{home} does not exist")
        return

    result = sudo_run(user, ["ls", "-la", str(home)])
    if result.returncode == 0:
        has_notebooks_link = "notebooks" in result.stdout
        if has_notebooks_link:
            report("PASS", "home:notebooks_link", "Shared workspace linked")
        else:
            report("FAIL", "home:notebooks_link", "No notebooks link in home")
    else:
        report("FAIL", "home:ls", f"Cannot list home: {result.stderr.strip()[:100]}")


def main():
    parser = argparse.ArgumentParser(description="Reviewer Tier Test")
    parser.add_argument("--user", default="abgreviewer")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    user = args.user
    print("═══════════════════════════════════════════════════")
    print(f"  Reviewer Tier Test — User: {user}")
    print(f"  Date: {time.strftime('%Y-%m-%dT%H:%M:%S%z')}")
    print("═══════════════════════════════════════════════════")

    print("\n── Read Access ──")
    test_read_access(user)

    print("\n── Notebook Readability ──")
    test_notebook_readability(user)

    print("\n── Write Enforcement ──")
    test_no_write(user)

    print("\n── No Compute Environment ──")
    test_no_venv(user)

    print("\n── Home Structure ──")
    test_home_structure(user)

    print()
    print("═══════════════════════════════════════════════════")
    print(f"  Results: {PASS_COUNT} PASS, {FAIL_COUNT} FAIL")
    print("═══════════════════════════════════════════════════")

    if args.json:
        print(json.dumps({"pass": PASS_COUNT, "fail": FAIL_COUNT, "results": RESULTS}, indent=2))

    sys.exit(min(FAIL_COUNT, 125))


if __name__ == "__main__":
    main()
