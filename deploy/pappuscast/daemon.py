"""Tier orchestrators and daemon loop for pappusCast."""

import time
from datetime import datetime, timezone
from pathlib import Path

from .config import ABG_SHARED, PUBLIC_ROOT, HEAVY_INTERVAL_S, log
from .state import FileState, PublishState, file_sha256, scan_workspace, detect_changes
from .tiers import validate_light, validate_medium
from .publisher import publish_file, quarantine_file, compute_interval
from .export import export_static_html, export_all_html, generate_index_page, append_changelog


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
