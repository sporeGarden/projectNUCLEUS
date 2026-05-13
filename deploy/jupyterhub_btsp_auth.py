# SPDX-License-Identifier: AGPL-3.0-or-later
"""
JupyterHub BTSP Authenticator — BearDog ionic token verification.

Dual-auth plugin: accepts both BTSP ionic tokens and PAM passwords
during the shadow run period. Users submit their ionic token in the
password field. The plugin verifies via BearDog JSON-RPC, falling back
to PAM when BTSP verification fails and dual-auth is enabled.

Shadow run protocol (H2-01 through H2-04):
  1. BTSP + PAM both accepted (dual-auth mode)
  2. Each login logs which auth method succeeded
  3. After 7-day shadow run with >=99.9% BTSP success and <50ms overhead,
     PAM can be disabled

Wire: auth.verify_ionic on BearDog (port from BEARDOG_PORT env, default 9100)
"""

import json
import logging
import os
import socket
import time

from jupyterhub.auth import Authenticator, PAMAuthenticator
from traitlets import Bool, Int, Unicode

logger = logging.getLogger(__name__)

BEARDOG_HOST = os.environ.get("BEARDOG_HOST", "127.0.0.1")
BEARDOG_PORT = int(os.environ.get("BEARDOG_PORT", "9100"))
ALLOWED_TIERS = {"compute", "admin", "pi"}


def beardog_rpc(method: str, params: dict, timeout: float = 2.0) -> dict:
    """Send a JSON-RPC 2.0 request to BearDog over TCP."""
    request = json.dumps({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    }).encode()

    try:
        with socket.create_connection(
            (BEARDOG_HOST, BEARDOG_PORT), timeout=timeout
        ) as sock:
            sock.sendall(request + b"\n")
            chunks = []
            while True:
                chunk = sock.recv(4096)
                if not chunk:
                    break
                chunks.append(chunk)
                try:
                    return json.loads(b"".join(chunks))
                except json.JSONDecodeError:
                    continue
            return json.loads(b"".join(chunks))
    except (OSError, json.JSONDecodeError) as exc:
        logger.warning("BearDog RPC failed: %s", exc)
        return {"error": {"code": -32000, "message": str(exc)}}


class BTSPAuthenticator(Authenticator):
    """
    Authenticator that verifies BearDog ionic tokens.

    In dual_auth mode (default during shadow run), falls back to PAM
    when BTSP verification fails. Logs which method succeeded for
    shadow run analysis.
    """

    dual_auth = Bool(
        True,
        config=True,
        help="Accept PAM passwords as fallback alongside BTSP tokens.",
    )

    beardog_host = Unicode(
        BEARDOG_HOST, config=True, help="BearDog RPC host."
    )

    beardog_port = Int(
        BEARDOG_PORT, config=True, help="BearDog RPC port."
    )

    btsp_timeout_ms = Int(
        2000, config=True, help="Timeout for BearDog RPC calls in milliseconds."
    )

    async def authenticate(self, handler, data):
        username = data.get("username", "")
        token = data.get("password", "")

        start_ns = time.monotonic_ns()
        btsp_result = self._verify_btsp(token)
        elapsed_ms = (time.monotonic_ns() - start_ns) / 1_000_000

        if btsp_result is not None:
            logger.info(
                "AUTH_BTSP user=%s tier=%s latency_ms=%.1f",
                btsp_result["name"], btsp_result["auth_state"]["tier"],
                elapsed_ms,
            )
            return btsp_result

        if self.dual_auth:
            pam = PAMAuthenticator()
            pam_result = await pam.authenticate(handler, data)
            if pam_result is not None:
                pam_name = (
                    pam_result["name"]
                    if isinstance(pam_result, dict) else pam_result
                )
                logger.info(
                    "AUTH_PAM user=%s btsp_latency_ms=%.1f", pam_name,
                    elapsed_ms,
                )
                return pam_result

        logger.warning(
            "AUTH_FAIL user=%s dual_auth=%s btsp_latency_ms=%.1f",
            username, self.dual_auth, elapsed_ms,
        )
        return None

    def _verify_btsp(self, token: str) -> dict | None:
        """Verify an ionic token via BearDog RPC."""
        if not token or len(token) < 16:
            return None

        resp = beardog_rpc(
            "auth.verify_ionic",
            {"token": token},
            timeout=self.btsp_timeout_ms / 1000,
        )

        if "error" in resp:
            return None

        result = resp.get("result", {})
        if not result.get("valid"):
            return None

        tier = result.get("tier", "")
        identity = result.get("identity", "")

        if tier not in ALLOWED_TIERS:
            logger.warning(
                "BTSP tier rejected: identity=%s tier=%s", identity, tier
            )
            return None

        return {
            "name": identity,
            "auth_state": {
                "tier": tier,
                "token_expiry": result.get("expiry"),
                "auth_method": "btsp",
            },
        }

    async def pre_spawn_hook(self, spawner):
        """Inject BTSP auth state into the notebook server environment."""
        auth_state = await spawner.user.get_auth_state()
        if auth_state and auth_state.get("auth_method") == "btsp":
            spawner.environment["BTSP_TIER"] = auth_state.get("tier", "")
            spawner.environment["BTSP_AUTH"] = "1"
