#!/usr/bin/env python3
"""
Observer Tier Test — Static Pre-Rendered Surface Validation

Validates the static observer surface: notebook execution as voila user,
rendered HTML quality (theme, nav, links, tracebacks), source stripping,
security headers, and directory blocking.

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
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

PASS_COUNT = 0
FAIL_COUNT = 0
RESULTS = []

sys.path.insert(0, str(Path(__file__).resolve().parent))
from nucleus_paths import PUBLIC_ROOT, JUPYTER_BIN, VOILA_PORT

VOILA_USER = "voila"
REDIRECT_PORT = VOILA_PORT


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
    """Verify the root serves index.html with 200."""
    test_name = "root:index"
    try:
        req = urllib.request.Request(f"http://127.0.0.1:{REDIRECT_PORT}/")
        with urllib.request.urlopen(req, timeout=10) as resp:
            body = resp.read().decode(errors="replace")
            if resp.status == 200 and "NUCLEUS Observer" in body:
                report("PASS", test_name, "Root serves index.html with NUCLEUS Observer heading")
            else:
                report("FAIL", test_name, f"Got status {resp.status}, missing expected content")
    except Exception as e:
        report("FAIL", test_name, str(e)[:150])


def test_source_stripping():
    """Fetch a rendered page and verify no Python source is exposed."""
    test_name = "source_strip"
    html_dir = PUBLIC_ROOT / ".pappusCast" / "html_export"
    html_files = sorted(html_dir.rglob("*.html"))
    if not html_files:
        report("FAIL", test_name, "No HTML files in html_export/")
        return
    for html_file in html_files:
        if html_file.name == "index.html":
            continue
        rel = html_file.relative_to(html_dir)
        try:
            body = html_file.read_text(encoding="utf-8", errors="replace")
            code_inputs = re.findall(
                r'<div class="jp-InputArea[^"]*"[^>]*>\s*'
                r'<div class="jp-InputPrompt[^"]*"[^>]*>',
                body,
            )
            if code_inputs:
                report("FAIL", f"source:{rel}", f"Found {len(code_inputs)} code input cells visible (should use --no-input)")
            else:
                report("PASS", f"source:{rel}", "No Python source code visible")
        except Exception as e:
            report("FAIL", f"source:{rel}", str(e)[:150])


def test_static_html_quality():
    """Validate rendered HTML files: theme, nav, no broken Voila links, no error outputs."""
    html_dir = PUBLIC_ROOT / ".pappusCast" / "html_export"
    html_files = sorted(html_dir.rglob("*.html"))
    if not html_files:
        report("FAIL", "html_quality", "No HTML files found")
        return
    for html_file in html_files:
        if html_file.name == "index.html":
            continue
        rel = html_file.relative_to(html_dir)
        try:
            body = html_file.read_text(encoding="utf-8", errors="replace")
        except OSError as e:
            report("FAIL", f"read:{rel}", str(e)[:150])
            continue

        if "jp-layout-color0: #0d1117" in body:
            report("PASS", f"theme:{rel}", "Dark theme CSS injected")
        else:
            report("FAIL", f"theme:{rel}", "Missing dark theme CSS override")

        if '<nav ' in body and 'index.html' in body:
            report("PASS", f"nav:{rel}", "Navigation bar present with Home link")
        else:
            report("FAIL", f"nav:{rel}", "Missing navigation bar or Home link")

        voila_links = re.findall(r'href="/voila/render/', body)
        if voila_links:
            report("FAIL", f"links:{rel}", f"{len(voila_links)} broken Voila links remaining")
        else:
            report("PASS", f"links:{rel}", "No legacy Voila links")

        traceback_count = body.count("Traceback (most recent call last)")
        if traceback_count > 0:
            report("FAIL", f"errors:{rel}", f"{traceback_count} traceback(s) in rendered output")
        else:
            report("PASS", f"errors:{rel}", "No tracebacks in rendered output")


def test_no_internal_dirs():
    """Verify internal directories are not served by the static observer."""
    blocked = ["envs", "wheelhouse", "templates", ".ipynb_checkpoints", ".pappusCast"]
    for d in blocked:
        try:
            url = f"http://127.0.0.1:{REDIRECT_PORT}/{d}/"
            req = urllib.request.Request(url)
            with urllib.request.urlopen(req, timeout=5) as resp:
                report("FAIL", f"dir_blocked:{d}", f"Accessible at /{d}/ (status {resp.status})")
        except urllib.error.HTTPError as e:
            if e.code in (403, 404):
                report("PASS", f"dir_blocked:{d}", f"Blocked (HTTP {e.code})")
            else:
                report("FAIL", f"dir_blocked:{d}", f"Unexpected HTTP {e.code}")
        except Exception as e:
            report("PASS", f"dir_blocked:{d}", f"Not accessible: {e}")


def test_response_headers():
    """Verify security headers on the static observer."""
    test_name = "headers"
    try:
        req = urllib.request.Request(f"http://127.0.0.1:{REDIRECT_PORT}/")
        with urllib.request.urlopen(req, timeout=10) as resp:
            checks = {
                "X-Robots-Tag": "noai",
                "X-Content-Type-Options": "nosniff",
                "X-Frame-Options": "DENY",
                "Referrer-Policy": "no-referrer",
            }
            for header, expected in checks.items():
                actual = resp.headers.get(header, "")
                if expected in actual:
                    report("PASS", f"header:{header}", f"{header}: {actual}")
                else:
                    report("FAIL", f"header:{header}", f"Expected '{expected}', got '{actual}'")
    except Exception as e:
        report("FAIL", test_name, str(e)[:150])


def main():
    parser = argparse.ArgumentParser(description="Observer Tier Test")
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    print("═══════════════════════════════════════════════════")
    print("  Observer Tier Test — Static Pre-Rendered Surface")
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

    print("\n── Static HTML Quality ──")
    test_static_html_quality()

    print("\n── HTTP Behavior ──")
    test_redirect()
    test_source_stripping()
    test_response_headers()
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
