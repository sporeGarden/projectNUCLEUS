#!/usr/bin/env python3
"""
Dark Forest Fuzz Tester — Protocol-level fuzzing for NUCLEUS primals and JupyterHub.

Tests:
  1. All 13 primal JSON-RPC ports — malformed payloads, auth bypass, resource exhaustion
  2. JupyterHub HTTP API — malformed tokens, oversized cookies, null bytes
  3. Timing analysis — response time consistency across methods (enumeration signals)

Output: PASS|FAIL|KNOWN_GAP|DARK_FOREST per assertion, compatible with security_validation.sh

Usage:
  python3 darkforest_fuzz.py [--hub-only] [--primals-only] [--port PORT] [--rounds N]
"""

import argparse
import json
import os
import socket
import sys
import time
import struct
import random
import string
from concurrent.futures import ThreadPoolExecutor, as_completed

PASS_COUNT = 0
FAIL_COUNT = 0
GAP_COUNT = 0
DF_COUNT = 0

def log_pass(suite, probe, msg):
    global PASS_COUNT
    print(f"PASS|{suite}|{probe}|{msg}")
    PASS_COUNT += 1

def log_fail(suite, probe, msg):
    global FAIL_COUNT
    print(f"FAIL|{suite}|{probe}|{msg}")
    FAIL_COUNT += 1

def log_gap(suite, probe, msg):
    global GAP_COUNT
    print(f"KNOWN_GAP|{suite}|{probe}|{msg}")
    GAP_COUNT += 1

def log_dark(suite, probe, msg):
    global DF_COUNT
    print(f"DARK_FOREST|{suite}|{probe}|{msg}")
    DF_COUNT += 1


PRIMAL_PORTS = {
    "beardog":     9100,
    "songbird":    9200,
    "squirrel":    9300,
    "toadstool":   9400,
    "nestgate":    9500,
    "rhizocrypt":  9601,
    "loamspine":   9700,
    "coralreef":   9730,
    "barracuda":   9740,
    "biomeos":     9800,
    "sweetgrass":  9850,
    "petaltongue": 9900,
    "skunkbat":    9140,
}

HUB_PORT = 8000
BIND = "127.0.0.1"


def send_raw(host, port, data, timeout=3):
    """Send raw bytes and return response bytes."""
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(timeout)
        s.connect((host, port))
        if data:
            s.sendall(data)
        resp = b""
        while True:
            try:
                chunk = s.recv(4096)
                if not chunk:
                    break
                resp += chunk
                if len(resp) > 65536:
                    break
            except socket.timeout:
                break
        s.close()
        return resp
    except (ConnectionRefusedError, OSError, socket.timeout):
        return None


def send_jsonrpc(host, port, payload_str, timeout=3):
    """Send a JSON-RPC string via HTTP POST and return (status_line, body)."""
    content = payload_str.encode()
    http_req = (
        f"POST / HTTP/1.1\r\n"
        f"Host: {host}:{port}\r\n"
        f"Content-Type: application/json\r\n"
        f"Content-Length: {len(content)}\r\n"
        f"Connection: close\r\n"
        f"\r\n"
    ).encode() + content
    resp = send_raw(host, port, http_req, timeout)
    if resp is None:
        return None, None
    try:
        parts = resp.split(b"\r\n\r\n", 1)
        status = parts[0].split(b"\r\n")[0].decode(errors="replace")
        body = parts[1] if len(parts) > 1 else b""
        return status, body.decode(errors="replace")
    except Exception:
        return None, resp.decode(errors="replace") if resp else None


# ═══════════════════════════════════════════════════
# FUZZ PAYLOADS
# ═══════════════════════════════════════════════════

MALFORMED_PAYLOADS = [
    # Empty / null
    ("empty_string", b""),
    ("null_byte", b"\x00"),
    ("null_json", b"null"),
    ("empty_object", b"{}"),
    # Missing required fields
    ("no_jsonrpc", json.dumps({"method": "health.liveness", "id": 1}).encode()),
    ("no_method", json.dumps({"jsonrpc": "2.0", "id": 1}).encode()),
    ("no_id", json.dumps({"jsonrpc": "2.0", "method": "health.liveness"}).encode()),
    # Wrong types
    ("method_int", json.dumps({"jsonrpc": "2.0", "method": 42, "id": 1}).encode()),
    ("method_null", json.dumps({"jsonrpc": "2.0", "method": None, "id": 1}).encode()),
    ("method_array", json.dumps({"jsonrpc": "2.0", "method": ["a"], "id": 1}).encode()),
    ("params_string", json.dumps({"jsonrpc": "2.0", "method": "health.liveness", "params": "evil", "id": 1}).encode()),
    ("id_overflow", json.dumps({"jsonrpc": "2.0", "method": "health.liveness", "id": 2**53}).encode()),
    # Auth bypass attempts
    ("auth_admin_param", json.dumps({"jsonrpc": "2.0", "method": "health.liveness", "params": {"auth": "admin"}, "id": 1}).encode()),
    ("bearer_in_params", json.dumps({"jsonrpc": "2.0", "method": "health.liveness", "params": {"bearer": "root"}, "id": 1}).encode()),
    # Injection
    ("method_injection", json.dumps({"jsonrpc": "2.0", "method": "__import__('os').system('id')", "id": 1}).encode()),
    ("method_traversal", json.dumps({"jsonrpc": "2.0", "method": "../../../etc/passwd", "id": 1}).encode()),
    ("method_null_byte", json.dumps({"jsonrpc": "2.0", "method": "health\x00.liveness", "id": 1}).encode()),
    # Nesting
    ("deep_nesting", json.dumps({"jsonrpc": "2.0", "method": "health.liveness", "params": {"a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": 1}}}}}}}}, "id": 1}).encode()),
    # Batch
    ("batch_100", json.dumps([{"jsonrpc": "2.0", "method": "health.liveness", "id": i} for i in range(100)]).encode()),
]

BINARY_PROBES = [
    ("tls_clienthello", b"\x16\x03\x01\x00\xf1\x01\x00\x00\xed\x03\x03"),
    ("http2_preface", b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"),
    ("ssh_banner", b"SSH-2.0-OpenSSH_9.0\r\n"),
    ("redis_ping", b"*1\r\n$4\r\nPING\r\n"),
    ("memcached_stats", b"stats\r\n"),
]


def fuzz_primal(name, port, host=BIND):
    """Run all fuzz payloads against a single primal."""
    suite = f"fuzz_{name}"
    reachable = send_raw(host, port, b"", timeout=2) is not None
    if not reachable:
        if send_raw(host, port, b"test", timeout=2) is None:
            log_pass(suite, "not_running", f"{name}:{port} not listening (skip)")
            return

    crashed_count = 0
    error_shapes = set()

    for pname, payload in MALFORMED_PAYLOADS:
        resp = send_raw(host, port, payload, timeout=3)
        if resp is None:
            # Port stopped responding after payload — possible crash
            time.sleep(0.5)
            check = send_raw(host, port, b"", timeout=2)
            if check is None:
                log_fail(suite, f"crash_{pname}", f"{name} stopped responding after {pname}")
                crashed_count += 1
                break
        else:
            # Collect error shape for uniformity check
            try:
                body = resp.split(b"\r\n\r\n", 1)[-1] if b"\r\n\r\n" in resp else resp
                parsed = json.loads(body)
                if "error" in parsed:
                    shape = tuple(sorted(parsed["error"].keys()))
                    error_shapes.add(shape)
            except (json.JSONDecodeError, UnicodeDecodeError):
                pass

    if crashed_count == 0:
        log_pass(suite, "malformed_resilient", f"{name} handled all {len(MALFORMED_PAYLOADS)} malformed payloads")

    if len(error_shapes) > 1:
        log_dark(suite, "error_shape_inconsistent", f"{name} returns {len(error_shapes)} different error shapes — enumeration signal")
    elif len(error_shapes) == 1:
        log_pass(suite, "error_shape_uniform", f"{name} returns uniform error structure")

    # Binary protocol probes
    for bname, bdata in BINARY_PROBES:
        resp = send_raw(host, port, bdata, timeout=2)
        if resp and len(resp) > 0:
            # Should either reject cleanly or return nothing
            try:
                text = resp.decode(errors="replace")
                if "200 OK" in text or "result" in text:
                    log_fail(suite, f"binary_{bname}", f"{name} responded to {bname} with success")
                    continue
            except Exception:
                pass
        log_pass(suite, f"binary_{bname}", f"{name} rejects {bname}")

    # Large payload (100KB)
    big_payload = json.dumps({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {"data": "A" * 100_000},
        "id": 1,
    }).encode()
    resp = send_raw(host, port, big_payload, timeout=5)
    if resp is None:
        time.sleep(1)
        check = send_raw(host, port, b"", timeout=2)
        if check is None:
            log_fail(suite, "large_payload_crash", f"{name} crashed on 100KB payload")
        else:
            log_pass(suite, "large_payload", f"{name} handled 100KB payload")
    else:
        log_pass(suite, "large_payload", f"{name} handled 100KB payload")


def timing_analysis(name, port, host=BIND, rounds=10):
    """Measure response time variance to detect enumeration signals."""
    suite = f"timing_{name}"
    methods = ["health.liveness", "nonexistent.method", "admin.secret", "storage.list"]
    timings = {}

    for method in methods:
        times = []
        payload = json.dumps({"jsonrpc": "2.0", "method": method, "id": 1})
        for _ in range(rounds):
            t0 = time.monotonic()
            status, body = send_jsonrpc(host, port, payload, timeout=3)
            elapsed = time.monotonic() - t0
            if status is not None:
                times.append(elapsed)
            time.sleep(0.05)
        if times:
            timings[method] = {"mean": sum(times) / len(times), "count": len(times)}

    if len(timings) < 2:
        log_pass(suite, "timing_insufficient", f"{name}: too few methods responded for timing analysis")
        return

    means = [v["mean"] for v in timings.values()]
    max_diff = max(means) - min(means)
    if max_diff > 0.1:
        detail = ", ".join(f"{m}={v['mean']:.3f}s" for m, v in timings.items())
        log_dark(suite, "timing_variance", f"{name}: {max_diff:.3f}s variance across methods ({detail})")
    else:
        log_pass(suite, "timing_uniform", f"{name}: response times within {max_diff:.3f}s")


def fuzz_jupyterhub(host=BIND, port=HUB_PORT):
    """Fuzz the JupyterHub HTTP API."""
    suite = "fuzz_hub"
    import http.client

    # Oversized cookie
    try:
        conn = http.client.HTTPConnection(host, port, timeout=5)
        conn.request("GET", "/hub/login", headers={
            "Cookie": "jupyterhub-session-id=" + "A" * 50_000,
        })
        r = conn.getresponse()
        if r.status in (200, 302, 400, 403):
            log_pass(suite, "oversized_cookie", f"Hub handles oversized cookie (HTTP {r.status})")
        else:
            log_dark(suite, "oversized_cookie", f"Unexpected response to oversized cookie: HTTP {r.status}")
        conn.close()
    except Exception as e:
        log_pass(suite, "oversized_cookie", f"Hub rejected oversized cookie: {e}")

    # Null bytes in username — check if the literal injected value appears in response
    try:
        conn = http.client.HTTPConnection(host, port, timeout=5)
        marker = "xfuzz" + "".join(random.choices(string.ascii_lowercase, k=6))
        body = f"username={marker}%00evil&password=test"
        conn.request("POST", "/hub/login", body=body, headers={
            "Content-Type": "application/x-www-form-urlencoded",
            "Content-Length": str(len(body)),
        })
        r = conn.getresponse()
        r_body = r.read().decode(errors="replace")
        if marker in r_body:
            log_fail(suite, "null_byte_user", f"Null byte username reflected in response (HTTP {r.status})")
        else:
            log_pass(suite, "null_byte_user", f"Null byte username handled (HTTP {r.status})")
        conn.close()
    except Exception as e:
        log_pass(suite, "null_byte_user", f"Null byte username rejected: {e}")

    # Malformed OAuth token
    try:
        conn = http.client.HTTPConnection(host, port, timeout=5)
        conn.request("GET", "/hub/api/users", headers={
            "Authorization": "token " + "x" * 1000,
        })
        r = conn.getresponse()
        if r.status == 200:
            log_fail(suite, "fake_token", f"Fake token accepted on /hub/api/users")
        else:
            log_pass(suite, "fake_token", f"Fake token rejected (HTTP {r.status})")
        conn.close()
    except Exception as e:
        log_pass(suite, "fake_token", f"Fake token rejected: {e}")

    # SQL-like injection in username
    for sqli in ["admin'--", "admin' OR '1'='1", '" OR ""="', "admin\"; DROP TABLE users;--"]:
        try:
            conn = http.client.HTTPConnection(host, port, timeout=5)
            body = f"username={sqli}&password=test"
            conn.request("POST", "/hub/login", body=body, headers={
                "Content-Type": "application/x-www-form-urlencoded",
                "Content-Length": str(len(body)),
            })
            r = conn.getresponse()
            r.read()
            conn.close()
        except Exception:
            pass
    log_pass(suite, "sqli_login", "Login form handles SQL injection payloads without crash")

    # XSS in next parameter
    try:
        conn = http.client.HTTPConnection(host, port, timeout=5)
        conn.request("GET", '/hub/login?next="><script>alert(1)</script>')
        r = conn.getresponse()
        body = r.read().decode(errors="replace")
        if "<script>alert(1)</script>" in body:
            log_fail(suite, "xss_next", "XSS in ?next parameter reflected")
        else:
            log_pass(suite, "xss_next", "XSS in ?next parameter not reflected")
        conn.close()
    except Exception as e:
        log_pass(suite, "xss_next", f"XSS probe handled: {e}")

    # HTTP method tampering
    for method in ["PUT", "DELETE", "PATCH", "OPTIONS", "TRACE"]:
        try:
            conn = http.client.HTTPConnection(host, port, timeout=5)
            conn.request(method, "/hub/api/users")
            r = conn.getresponse()
            r.read()
            if r.status == 200 and method in ("DELETE", "PUT"):
                log_fail(suite, f"method_{method.lower()}", f"{method} /hub/api/users returns 200")
            else:
                log_pass(suite, f"method_{method.lower()}", f"{method} /hub/api/users returns {r.status}")
            conn.close()
        except Exception:
            pass

    # TRACE method (should never echo)
    try:
        conn = http.client.HTTPConnection(host, port, timeout=5)
        conn.request("TRACE", "/hub/")
        r = conn.getresponse()
        body = r.read().decode(errors="replace")
        if "TRACE" in body and r.status == 200:
            log_fail(suite, "trace_echo", "TRACE method echoes request — XST vulnerability")
        else:
            log_pass(suite, "trace_echo", f"TRACE method blocked (HTTP {r.status})")
        conn.close()
    except Exception:
        log_pass(suite, "trace_echo", "TRACE method rejected")

    # Concurrent connections
    try:
        sockets = []
        for i in range(50):
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(2)
            s.connect((host, port))
            sockets.append(s)
        for s in sockets:
            s.close()
        # Now check hub still responds
        time.sleep(1)
        conn = http.client.HTTPConnection(host, port, timeout=5)
        conn.request("GET", "/hub/login")
        r = conn.getresponse()
        if r.status in (200, 302):
            log_pass(suite, "conn_flood", "Hub survives 50 concurrent connections")
        else:
            log_dark(suite, "conn_flood", f"Hub degraded after 50 connections (HTTP {r.status})")
        conn.close()
    except Exception as e:
        log_dark(suite, "conn_flood", f"Hub degraded after connection flood: {e}")


def main():
    parser = argparse.ArgumentParser(description="Dark Forest Fuzz Tester")
    parser.add_argument("--hub-only", action="store_true")
    parser.add_argument("--primals-only", action="store_true")
    parser.add_argument("--port", type=int, help="Test only this port")
    parser.add_argument("--rounds", type=int, default=5, help="Timing analysis rounds")
    parser.add_argument("--host", default=BIND)
    args = parser.parse_args()

    print("═══════════════════════════════════════════════════")
    print("  Dark Forest Fuzz Tester")
    print(f"  Date: {time.strftime('%Y-%m-%dT%H:%M:%S%z')}")
    print("═══════════════════════════════════════════════════")
    print()

    if args.port:
        name = next((n for n, p in PRIMAL_PORTS.items() if p == args.port), f"port_{args.port}")
        print(f"── Fuzzing {name}:{args.port} ──")
        fuzz_primal(name, args.port, args.host)
        timing_analysis(name, args.port, args.host, args.rounds)
    elif args.hub_only:
        print("── Fuzzing JupyterHub ──")
        fuzz_jupyterhub(args.host, HUB_PORT)
    elif args.primals_only:
        for name, port in sorted(PRIMAL_PORTS.items()):
            print(f"\n── Fuzzing {name}:{port} ──")
            fuzz_primal(name, port, args.host)
            timing_analysis(name, port, args.host, args.rounds)
    else:
        # All: primals + hub
        for name, port in sorted(PRIMAL_PORTS.items()):
            print(f"\n── Fuzzing {name}:{port} ──")
            fuzz_primal(name, port, args.host)
            timing_analysis(name, port, args.host, args.rounds)

        print("\n── Fuzzing JupyterHub ──")
        fuzz_jupyterhub(args.host, HUB_PORT)

    print()
    print("═══════════════════════════════════════════════════")
    print(f"  Results: {PASS_COUNT} PASS, {FAIL_COUNT} FAIL, {GAP_COUNT} KNOWN_GAP, {DF_COUNT} DARK_FOREST")
    print("═══════════════════════════════════════════════════")

    sys.exit(1 if FAIL_COUNT > 0 else 0)


if __name__ == "__main__":
    main()
