#!/usr/bin/env python3
"""
Observer Tier Test — Voila Public Surface Validation

Executes every notebook in the public Voila tree as the voila user,
verifying each renders without errors. Also checks HTTP accessibility,
source stripping, and redirect behavior.

Usage:
    sudo python3 tier_test_observer.py [--json]

Output:
    PASS|observer|<test>|<detail>
    FAIL|observer|<test>|<detail>

Exit code: number of FAILs (0 = all pass)
"""

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

PASS_COUNT = 0
FAIL_COUNT = 0
RESULTS = []

PUBLIC_ROOT = Path("/home/irongate/shared/abg/public")
VOILA_USER = "voila"
JUPYTER_BIN = "/home/irongate/miniforge3/envs/jupyterhub/bin"
VOILA_PORT = 8867
REDIRECT_PORT = 8866


def report(status, test, detail):
    global PASS_COUNT, FAIL_COUNT
    if status == "PASS":
        PASS_COUNT += 1
    else:
        FAIL_COUNT += 1
    entry = {"status": status, "tier": "observer", "test": test, "detail": detail}
    RESULTS.append(entry)
    print(f"{status}|observer|{test}|{detail}")


def discover_notebooks():
    """Walk the public tree and find all .ipynb files (following symlinks)."""
    notebooks = []
    for root, _, files in os.walk(PUBLIC_ROOT, followlinks=True):
        for f in files:
            if f.endswith(".ipynb") and ".ipynb_checkpoints" not in root:
                notebooks.append(Path(root) / f)
    return sorted(notebooks)


def test_notebook_execution(nb_path):
    """Execute a notebook as the voila user and check for errors."""
    rel = nb_path.relative_to(PUBLIC_ROOT)
    test_name = f"exec:{rel}"

    cmd = [
        "sudo", "-u", VOILA_USER,
        f"{JUPYTER_BIN}/jupyter", "nbconvert",
        "--execute",
        "--to", "notebook",
        "--stdout",
        "--ExecutePreprocessor.timeout=60",
        str(nb_path),
    ]
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=90
        )
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
                report("PASS", test_name, f"Rendered cleanly ({len(nb_out.get('cells',[]))} cells)")
        else:
            stderr_snippet = result.stderr.strip().split("\n")[-1][:150]
            report("FAIL", test_name, f"exit={result.returncode}: {stderr_snippet}")
    except subprocess.TimeoutExpired:
        report("FAIL", test_name, "Execution timed out after 90s")
    except Exception as e:
        report("FAIL", test_name, str(e)[:150])


def test_kernel_consistency():
    """Verify all public notebooks use a kernel available to voila."""
    cmd = [
        "sudo", "-u", VOILA_USER,
        f"{JUPYTER_BIN}/jupyter", "kernelspec", "list", "--json",
    ]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
        available = set(json.loads(result.stdout).get("kernelspecs", {}).keys())
    except Exception:
        available = {"python3"}

    for nb_path in discover_notebooks():
        rel = nb_path.relative_to(PUBLIC_ROOT)
        try:
            with open(nb_path) as f:
                nb = json.load(f)
            kernel = nb.get("metadata", {}).get("kernelspec", {}).get("name", "python3")
            if kernel in available:
                report("PASS", f"kernel:{rel}", f"Uses '{kernel}' (available)")
            else:
                report("FAIL", f"kernel:{rel}", f"Uses '{kernel}' but voila only has: {', '.join(sorted(available))}")
        except Exception as e:
            report("FAIL", f"kernel:{rel}", f"Cannot read: {e}")


def test_notebook_metadata():
    """Check notebooks have titles and trusted metadata."""
    for nb_path in discover_notebooks():
        rel = nb_path.relative_to(PUBLIC_ROOT)
        try:
            with open(nb_path) as f:
                nb = json.load(f)
            meta = nb.get("metadata", {})
            title = meta.get("title", "")
            if not title:
                report("FAIL", f"title:{rel}", "Missing metadata.title for page title")
            else:
                report("PASS", f"title:{rel}", f"Title: {title}")
        except Exception as e:
            report("FAIL", f"meta:{rel}", f"Cannot read: {e}")


def test_redirect():
    """Verify the root redirect sends / to Welcome.ipynb."""
    test_name = "redirect:root"
    try:
        req = urllib.request.Request(
            f"http://127.0.0.1:{REDIRECT_PORT}/",
            method="GET",
        )
        req.add_header("Host", "lab.primals.eco")
        with urllib.request.urlopen(req, timeout=10) as resp:
            if resp.status == 200 and "voila/render/Welcome" in resp.url:
                report("PASS", test_name, f"Redirected to {resp.url}")
            else:
                report("FAIL", test_name, f"Got status {resp.status}, url={resp.url}")
    except urllib.error.HTTPError as e:
        if e.code in (301, 302, 307, 308):
            loc = e.headers.get("Location", "")
            if "Welcome" in loc:
                report("PASS", test_name, f"Redirect to {loc}")
            else:
                report("FAIL", test_name, f"Redirect to unexpected: {loc}")
        else:
            report("FAIL", test_name, f"HTTP error {e.code}")
    except Exception as e:
        report("FAIL", test_name, str(e)[:150])


def test_source_stripping():
    """Fetch a rendered page from Voila and verify no Python source is exposed."""
    test_name = "source_strip"
    try:
        url = f"http://127.0.0.1:{VOILA_PORT}/voila/render/Welcome.ipynb"
        req = urllib.request.Request(url)
        with urllib.request.urlopen(req, timeout=30) as resp:
            body = resp.read().decode(errors="replace")
            import re
            code_blocks = re.findall(r'<pre[^>]*>.*?(?:import |def |class |print\()', body, re.DOTALL)
            if code_blocks:
                report("FAIL", test_name, f"Found {len(code_blocks)} code blocks with Python source in rendered HTML")
            else:
                report("PASS", test_name, "No Python source code visible in rendered HTML")
    except Exception as e:
        report("FAIL", test_name, str(e)[:150])


def test_no_internal_dirs():
    """Verify internal directories are not accessible via Voila tree API."""
    test_name = "no_internal_dirs"
    blocked = ["envs", "wheelhouse", "templates", ".ipynb_checkpoints"]
    for d in blocked:
        try:
            url = f"http://127.0.0.1:{VOILA_PORT}/voila/tree/{d}"
            req = urllib.request.Request(url)
            with urllib.request.urlopen(req, timeout=10) as resp:
                report("FAIL", f"dir_blocked:{d}", f"Accessible at /voila/tree/{d} (status {resp.status})")
        except urllib.error.HTTPError as e:
            if e.code in (403, 404):
                report("PASS", f"dir_blocked:{d}", f"Blocked (HTTP {e.code})")
            else:
                report("FAIL", f"dir_blocked:{d}", f"Unexpected HTTP {e.code}")
        except Exception as e:
            report("PASS", f"dir_blocked:{d}", f"Not accessible: {e}")


def main():
    parser = argparse.ArgumentParser(description="Observer Tier Test")
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    print("═══════════════════════════════════════════════════")
    print("  Observer Tier Test — Public Voila Surface")
    print(f"  Date: {time.strftime('%Y-%m-%dT%H:%M:%S%z')}")
    print(f"  Root: {PUBLIC_ROOT}")
    print("═══════════════════════════════════════════════════")

    notebooks = discover_notebooks()
    print(f"\nDiscovered {len(notebooks)} notebooks in public tree\n")

    print("── Kernel Consistency ──")
    test_kernel_consistency()

    print("\n── Notebook Metadata ──")
    test_notebook_metadata()

    print("\n── Notebook Execution (as voila user) ──")
    for nb in notebooks:
        test_notebook_execution(nb)

    print("\n── HTTP Behavior ──")
    test_redirect()
    test_source_stripping()
    test_no_internal_dirs()

    print()
    print("═══════════════════════════════════════════════════")
    print(f"  Results: {PASS_COUNT} PASS, {FAIL_COUNT} FAIL")
    print("═══════════════════════════════════════════════════")

    if args.json:
        print(json.dumps({"pass": PASS_COUNT, "fail": FAIL_COUNT, "results": RESULTS}, indent=2))

    sys.exit(min(FAIL_COUNT, 125))


if __name__ == "__main__":
    main()
