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
import json
import sys
import time
from datetime import datetime, timezone

from pappuscast.config import (
    STATE_DIR, QUARANTINE_DIR, PUBLIC_ROOT, STATIC_HTML_DIR,
    BASE_MINUTES, MAX_MINUTES, log,
)
from pappuscast.state import PublishState
from pappuscast.publisher import get_active_users
from pappuscast.export import export_all_html
from pappuscast.daemon import daemon_loop, run_once


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

    state_file = STATE_DIR / "last_publish.json"
    if state_file.exists():
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
