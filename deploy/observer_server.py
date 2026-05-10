"""Static observer server — serves pre-rendered notebook HTML from pappusCast.

Replaces the Voila dynamic rendering + redirect proxy with a zero-compute
static file server. All notebooks are pre-rendered by pappusCast (Medium
and Heavy tiers) and served as plain HTML.

Binds 127.0.0.1:8866 — same port the Cloudflare tunnel expects.

Usage:
    python3 observer_server.py
"""

import os
import sys
from functools import partial
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from nucleus_paths import PUBLIC_ROOT, VOILA_PORT

STATIC_DIR = PUBLIC_ROOT / ".pappusCast" / "html_export"
BIND = "127.0.0.1"
PORT = VOILA_PORT


class ObserverHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path in ("/", ""):
            self.path = "/index.html"
        super().do_GET()

    def end_headers(self):
        self.send_header("X-Robots-Tag", "noai")
        self.send_header("Cache-Control", "public, max-age=300")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("X-Frame-Options", "DENY")
        self.send_header("Referrer-Policy", "no-referrer")
        self.send_header("Server", "")
        super().end_headers()

    def log_message(self, fmt, *args):
        pass


def main():
    if not STATIC_DIR.exists():
        print(f"Static HTML dir not found: {STATIC_DIR}", file=sys.stderr)
        print("Run 'python3 pappusCast.py export' first to generate HTML.", file=sys.stderr)
        sys.exit(1)

    handler = partial(ObserverHandler, directory=str(STATIC_DIR))
    server = HTTPServer((BIND, PORT), handler)
    print(f"Observer serving {STATIC_DIR} on {BIND}:{PORT}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down")
        server.shutdown()


if __name__ == "__main__":
    main()
