"""Static HTML export and index generation for pappusCast observer surface."""

import getpass
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from .config import (
    PUBLIC_ROOT, JUPYTER_BIN, VOILA_USER,
    STATIC_HTML_DIR, THEME_CSS, CHANGELOG_FILE, STATE_DIR, log,
)
from .state import PublishState, scan_workspace


def _nbconvert_cmd() -> list[str]:
    """Return the nbconvert command prefix, using sudo only when needed."""
    if getpass.getuser() == VOILA_USER:
        return [f"{JUPYTER_BIN}/jupyter"]
    return ["sudo", "-u", VOILA_USER, f"{JUPYTER_BIN}/jupyter"]


def export_static_html(rel_path: str) -> tuple[bool, str]:
    """Execute a notebook and render to static HTML for the observer surface."""
    src = PUBLIC_ROOT / rel_path
    if src.suffix != ".ipynb" or not src.exists():
        return False, "not a notebook"

    out_dir = STATIC_HTML_DIR / Path(rel_path).parent
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / Path(rel_path).with_suffix(".html").name

    try:
        result = subprocess.run(
            _nbconvert_cmd() + ["nbconvert",
             "--execute", "--to", "html", "--no-input",
             "--ExecutePreprocessor.timeout=120",
             "--output", str(out_file), str(src)],
            capture_output=True, text=True, timeout=180,
        )
        if result.returncode != 0:
            return False, result.stderr.strip().split("\n")[-1][:200]

        _inject_nav(out_file)
        return True, str(out_file)
    except subprocess.TimeoutExpired:
        return False, "html export timed out"
    except (subprocess.SubprocessError, OSError) as e:
        return False, str(e)[:200]


def _nb_title(nb_path: Path) -> str:
    """Extract metadata.title from a notebook, falling back to stem."""
    try:
        nb = json.loads(nb_path.read_text())
        return nb.get("metadata", {}).get("title", "") or nb_path.stem
    except (json.JSONDecodeError, OSError):
        return nb_path.stem


def _build_nav_html(current_rel: str = "") -> str:
    """Build a top-navigation bar linking to all rendered notebooks."""
    sections: dict[str, list[tuple[str, str]]] = {}
    for html_file in sorted(STATIC_HTML_DIR.rglob("*.html")):
        if html_file.name == "index.html":
            continue
        rel = html_file.relative_to(STATIC_HTML_DIR)
        section = rel.parts[0] if len(rel.parts) > 1 else "root"
        nb_src = PUBLIC_ROOT / rel.with_suffix(".ipynb")
        title = _nb_title(nb_src) if nb_src.exists() else rel.stem
        depth = len(Path(current_rel).parts) - 1 if current_rel else 0
        prefix = "../" * depth if depth > 0 else ""
        sections.setdefault(section, []).append((f"{prefix}{rel}", title))

    nav_items = ['<a href="{}index.html" style="font-weight:bold">Home</a>'.format(
        "../" * (len(Path(current_rel).parts) - 1) if current_rel and len(Path(current_rel).parts) > 1 else "")]
    for section, links in sorted(sections.items()):
        for href, title in links:
            nav_items.append(f'<a href="{href}">{title}</a>')

    return (
        '<nav style="background:#1a1a2e;padding:10px 20px;display:flex;'
        'flex-wrap:wrap;gap:12px;font-family:system-ui,sans-serif;font-size:14px;'
        'border-bottom:2px solid #16213e">'
        + "".join(
            f'<span style="color:#e2e2e2">{item}</span>' for item in nav_items
        )
        + "</nav>"
    )


def _inject_nav(html_path: Path):
    """Insert theme CSS, navigation bar, and rewrite Voila links."""
    try:
        content = html_path.read_text(encoding="utf-8")
    except OSError:
        return
    rel = str(html_path.relative_to(STATIC_HTML_DIR))
    nav = _build_nav_html(rel)

    depth = len(Path(rel).parts) - 1
    prefix = "../" * depth if depth > 0 else ""
    content = re.sub(
        r'href="/voila/render/([^"]+)\.ipynb"',
        lambda m: f'href="{prefix}{m.group(1)}.html"',
        content,
    )

    theme_css = ""
    if THEME_CSS.exists():
        theme_css = f"\n<style>\n{THEME_CSS.read_text(encoding='utf-8')}\n</style>\n"

    if "</head>" in content:
        content = content.replace("</head>", f"{theme_css}</head>", 1)

    if "<body" in content:
        body_end = content.find(">", content.find("<body")) + 1
        content = content[:body_end] + "\n" + nav + "\n" + content[body_end:]
    html_path.write_text(content, encoding="utf-8")


def generate_index_page():
    """Build an index.html landing page for the static observer surface."""
    STATIC_HTML_DIR.mkdir(parents=True, exist_ok=True)

    welcome_nb = PUBLIC_ROOT / "Welcome.ipynb"
    welcome_html = ""
    if welcome_nb.exists():
        try:
            result = subprocess.run(
                _nbconvert_cmd() + ["nbconvert",
                 "--execute", "--to", "html", "--no-input",
                 "--ExecutePreprocessor.timeout=120",
                 "--output", str(STATIC_HTML_DIR / "Welcome.html"),
                 str(welcome_nb)],
                capture_output=True, text=True, timeout=180,
            )
            if result.returncode == 0:
                welcome_path = STATIC_HTML_DIR / "Welcome.html"
                raw = welcome_path.read_text(encoding="utf-8")
                start = raw.find("<body")
                end = raw.find("</body>")
                if start != -1 and end != -1:
                    body_start = raw.find(">", start) + 1
                    welcome_html = raw[body_start:end]
                    welcome_html = re.sub(
                        r'href="/voila/render/([^"]+)\.ipynb"',
                        r'href="\1.html"',
                        welcome_html,
                    )
                    welcome_html = welcome_html.replace(
                        'href="/hub/login"', 'href="https://lab.primals.eco/hub/login"'
                    )
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, OSError):
            pass

    sections: dict[str, list[tuple[str, str]]] = {}
    for html_file in sorted(STATIC_HTML_DIR.rglob("*.html")):
        if html_file.name in ("index.html", "Welcome.html"):
            continue
        rel = html_file.relative_to(STATIC_HTML_DIR)
        section = rel.parts[0] if len(rel.parts) > 1 else "root"
        nb_src = PUBLIC_ROOT / rel.with_suffix(".ipynb")
        title = _nb_title(nb_src) if nb_src.exists() else rel.stem
        sections.setdefault(section, []).append((str(rel), title))

    section_html_parts = []
    section_order = ["showcase", "commons", "data", "pilot", "validation", "root"]
    for sec in section_order:
        links = sections.get(sec, [])
        if not links:
            continue
        label = sec.title() if sec != "root" else "Other"
        items = "".join(
            f'<li><a href="{href}" style="color:#a8dadc;text-decoration:none">{title}</a></li>'
            for href, title in links
        )
        section_html_parts.append(
            f'<div style="margin-bottom:24px">'
            f'<h3 style="color:#f1faee;border-bottom:1px solid #457b9d;'
            f'padding-bottom:6px">{label}</h3>'
            f'<ul style="list-style:none;padding-left:0;line-height:2">{items}</ul>'
            f'</div>'
        )

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    page = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>NUCLEUS Observer — lab.primals.eco</title>
<style>
  body {{ background:#0d1117; color:#e2e2e2; font-family:system-ui,sans-serif;
         margin:0; padding:0; }}
  .container {{ max-width:900px; margin:0 auto; padding:20px 24px; }}
  a {{ color:#a8dadc; }}
  a:hover {{ color:#f1faee; }}
  .welcome-embed {{ margin:20px 0; padding:20px; background:#161b22;
                    border-radius:8px; border:1px solid #30363d; }}
  .footer {{ margin-top:40px; padding-top:16px; border-top:1px solid #30363d;
             font-size:12px; color:#8b949e; }}
</style>
</head>
<body>
<div class="container">
<h1 style="color:#f1faee">NUCLEUS Observer</h1>
<p>Pre-rendered science notebooks from the active gate.
Read-only, no login required. Updated automatically by pappusCast.</p>
"""
    if welcome_html:
        page += f'<div class="welcome-embed">{welcome_html}</div>\n'

    page += '<h2 style="color:#f1faee">Notebooks</h2>\n'
    page += "".join(section_html_parts)

    page += f"""
<div class="footer">
<p>Last rendered: {ts} | Powered by pappusCast</p>
<p>Static pre-rendered HTML. No kernels launched. No compute cost per visit.</p>
</div>
</div>
</body>
</html>"""

    index_path = STATIC_HTML_DIR / "index.html"
    index_path.write_text(page, encoding="utf-8")
    log.info("Generated index.html (%d sections, %d notebooks)",
             len(section_html_parts),
             sum(len(v) for v in sections.values()))


def export_all_html(state: PublishState) -> int:
    """Export all public notebooks to static HTML and generate index page."""
    STATIC_HTML_DIR.mkdir(parents=True, exist_ok=True)
    html_count = 0
    current = scan_workspace()
    for rel in sorted(current.keys()):
        if Path(rel).suffix == ".ipynb":
            ok, detail = export_static_html(rel)
            if ok:
                html_count += 1
            else:
                log.warning("HTML export failed [%s]: %s", rel, detail)

    generate_index_page()
    welcome_html = STATIC_HTML_DIR / "Welcome.html"
    if welcome_html.exists():
        _inject_nav(welcome_html)
    log.info("Exported %d static HTML renders + index.html", html_count)
    return html_count


def append_changelog(entries: list[dict]):
    """Append heavy-tier changelog entries."""
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    with open(CHANGELOG_FILE, "a") as f:
        for e in entries:
            f.write(json.dumps(e) + "\n")
