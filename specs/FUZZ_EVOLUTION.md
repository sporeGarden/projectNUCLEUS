# Fuzz & Pen Testing Evolution — From Localhost to Membrane

**Date**: May 15, 2026
**Status**: Planning — priorities ordered by attack surface risk
**Relates to**: `specs/SECURITY_VALIDATION.md`, `validation/darkforest/`,
`infra/benchScale/pentest/`, per-primal `fuzz/` directories

---

## Current State

### What Exists

| Layer | Tool | Coverage | Location |
|-------|------|----------|----------|
| OS/Network | `security_validation.sh` Layer 1 | UFW, hidepid, iptables, DNS exfil | `deploy/security_validation.sh` |
| Primal API | `security_validation.sh` Layer 2 | MethodGate (13/13), 127.0.0.1 binding | Same |
| Application | `security_validation.sh` Layer 3 | JupyterHub headers, tier enforcement | Same |
| ABG Tier | `security_validation.sh` Layer 4 | Ionic tokens, scope validation | Same |
| Dark Forest | `darkforest` (Rust) | JSON-RPC fuzz, crypto, observer HTML | `validation/darkforest/` |
| Per-primal fuzz | cargo-fuzz (libfuzzer) | Parser/deserializer coverage | `primals/*/fuzz/` |
| BenchScale | Python/bash helpers | JSON-RPC fuzz, three-layer scan, tunnel probe | `infra/benchScale/pentest/` |

### What's Missing

| Gap ID | Description | Risk | Priority |
|--------|-------------|------|----------|
| **MEM-14** | BearDog TLS :8443 not in darkforest fuzz/crypto suites | HIGH — public TLS surface | P0 |
| **MEM-EXT** | darkforest is localhost-only, no VPS membrane probing | HIGH — outer membrane unvalidated | P0 |
| **FUZZ-CI** | cargo-fuzz runs are manual, no nightly CI | MEDIUM — regressions undetected | P1 |
| **BTSP-FUZZ** | BTSP handshake has no coverage-guided fuzzing | HIGH — authentication surface | P1 |
| **MULTI-GATE** | No cross-gate security probing | MEDIUM — covalent mesh assumed safe | P2 |
| **VPS-RECOVERY** | No automated destroy+reprovision test | MEDIUM — recovery time unknown | P2 |
| **CORPUS-MGMT** | Fuzz corpora not versioned or shared between gates | LOW — individual gate runs | P3 |

---

## Evolution Phases

### Phase 1: darkforest Membrane Mode (P0 — next sprint)

Extend the existing `validation/darkforest/` Rust binary with a `--membrane`
flag that probes the VPS outer membrane over the network instead of localhost.

**Targets:**

| Service | Port | Protocol | Probe Type |
|---------|------|----------|------------|
| BearDog TLS | :8443 | HTTPS/JSON-RPC | Malformed TLS handshakes, invalid certs, RPC fuzz |
| BearDog crypto | :9100 | TCP/JSON-RPC | BTSP handshake manipulation, replay attacks |
| SkunkBat audit | :9140 | TCP/JSON-RPC | Unauthorized method calls, event injection |
| Caddy TLS | :443 | HTTPS | Path traversal, header injection, oversized requests |
| Songbird TURN | :3478 | UDP | Malformed STUN/TURN, credential brute-force |
| RustDesk hbbs | :21116 | TCP | ID enumeration, key validation bypass |

**Implementation approach:**
```rust
// validation/darkforest/src/membrane.rs (new module)
pub struct MembraneTarget {
    pub host: String,       // VPS IP or membrane.primals.eco
    pub ssh_tunnel: bool,   // probe via SSH tunnel or direct
}

pub fn run_membrane_suite(target: &MembraneTarget) -> Vec<ProbeResult> {
    let mut results = vec![];
    results.extend(probe_beardog_tls(target));
    results.extend(probe_caddy_surface(target));
    results.extend(probe_turn_udp(target));
    results.extend(probe_rustdesk_hbbs(target));
    results.extend(probe_skunkbat_unauthorized(target));
    results
}
```

**Success criteria**: darkforest `--membrane --host 157.230.3.183` runs all
probes and generates report. Zero unexpected open ports. All unauthorized RPC
calls return `-32001`. TLS rejects invalid certs. TURN rejects unauthenticated
relay requests.

### Phase 2: BTSP Protocol Fuzzing (P1)

Add coverage-guided fuzzing to BearDog's BTSP authentication handshake.
This is the ionic surface's authentication boundary — the most critical
attack surface for external access.

**Fuzz targets** (add to `primals/bearDog/fuzz/`):

| Target | What It Fuzzes | Why |
|--------|---------------|-----|
| `fuzz_btsp_handshake` | Client hello → server challenge → response flow | Core auth path |
| `fuzz_btsp_token_parse` | Ed25519-signed ionic token deserialization | Token forgery surface |
| `fuzz_btsp_replay` | Replayed handshake messages with stale nonces | Replay attack resistance |
| `fuzz_btsp_scope_escalation` | Valid token with manipulated scope claims | Privilege escalation |
| `fuzz_tls_record` | TLS record layer parsing (songbird-tls crate) | Parsing bugs → crashes |

**Hardware allocation**: CPU-bound. Run on ironGate or southGate (128 GB RAM
allows parallel fuzz jobs without swapping). No GPU needed.

**Corpus seeding**: Extract real BTSP handshake captures from `journalctl -u
jupyterhub` → seed corpus for coverage-guided exploration.

### Phase 3: Continuous Fuzz CI (P1)

Nightly automated cargo-fuzz runs across all primals with existing fuzz targets.
Time-boxed to prevent infinite loops eating compute.

**Design:**

```bash
#!/bin/bash
# deploy/nightly_fuzz.sh — cron: 0 2 * * * (2 AM daily)

FUZZ_TARGETS=(
    "primals/toadStool/fuzz:jsonrpc_parse,btsp_framing,config_toml,gpu_buffer"
    "primals/nestGate/fuzz:zfs_commands,api_endpoints,config,manifests,network"
    "primals/sweetGrass/fuzz:query_filter,attribution,braid_deserialize"
    "primals/rhizoCrypt/crates/rhizo-crypt-core/fuzz:vertex_cbor,session,merkle"
    "primals/loamSpine/fuzz:certificate,spine_ops,entry_parsing"
    "primals/coralReef/fuzz:wgsl,spirv,jsonrpc_dispatch"
    "primals/bearDog/fuzz:btsp_handshake,btsp_token,tls_record"
)

TIME_PER_TARGET=600  # 10 minutes per fuzz target
RESULTS_DIR="validation/fuzz-corpus/$(date +%Y%m%d)"

for entry in "${FUZZ_TARGETS[@]}"; do
    IFS=: read -r dir targets <<< "$entry"
    IFS=, read -ra target_list <<< "$targets"
    for target in "${target_list[@]}"; do
        timeout ${TIME_PER_TARGET} cargo fuzz run "$target" \
            --fuzz-dir "$dir" \
            -- -max_total_time=${TIME_PER_TARGET} \
            2>&1 | tee "${RESULTS_DIR}/${target}.log"
    done
done

# Report crashes to skunkBat
if find "$RESULTS_DIR" -name "crash-*" | head -1 | grep -q .; then
    # Notify via skunkBat security.audit_log
    echo '{"jsonrpc":"2.0","method":"security.audit_log","params":{"event":"fuzz_crash","dir":"'"$RESULTS_DIR"'"},"id":1}' \
        | timeout 2 bash -c 'exec 3<>/dev/tcp/127.0.0.1/9140; cat >&3; read -t1 <&3'
fi
```

**Corpus management**: `validation/fuzz-corpus/` committed to git (only
interesting corpus entries, not raw libfuzzer output). Crashes immediately
trigger investigation.

### Phase 4: Multi-Gate Dark Forest (P2)

Probe from one covalent gate to another to validate:
- MethodGate enforcement over LAN (not just localhost)
- Port binding actually on 127.0.0.1 (not 0.0.0.0)
- BirdSong multicast doesn't leak to non-family
- Cross-gate ionic tokens properly scoped

**Implementation**: darkforest `--remote --target northgate` resolves the
gate's LAN IP via Songbird discovery, then runs the standard probe suite
against it. All probes should fail (ports bound to localhost) — success is
"everything is unreachable except Songbird discovery."

**Prerequisites**: 10G backbone active (or at least 1G LAN connectivity
between gates). A second gate running full NUCLEUS.

### Phase 5: VPS Snapshot Recovery Test (P2)

Automated validation that the outer membrane can be destroyed and restored
from deploy scripts alone.

**Process:**
1. Snapshot current VPS state (DO API)
2. Destroy VPS (`doctl compute droplet delete`)
3. Provision fresh Debian 12 droplet
4. Run `deploy/deploy_membrane.sh --fresh`
5. Validate all 6 services healthy (Tower composition)
6. Run `darkforest --membrane` against new instance
7. Measure total recovery time (target: <5 minutes)
8. Delete snapshot (the point is stateless recovery)

**Why this matters**: The VPS is a sacrificial outer membrane. If it can't
be destroyed and rebuilt in minutes, it's not truly sacrificial — it's become
a dependency. This test proves the organism survives membrane loss.

### Phase 6: Fuzz Corpus Sharing (P3)

Share fuzz corpora across gates so discoveries on one machine benefit all.

**Design**: NestGate content-addressed storage holds corpus artifacts.
When a gate discovers a new coverage-increasing input, it publishes to
NestGate. Other gates' nightly fuzz runs pull the shared corpus before
starting. Crashes propagate immediately via SkunkBat audit events.

---

## Hardware Allocation for Security Testing

| Activity | Gate | Why | Resource Use |
|----------|------|-----|--------------|
| darkforest (localhost) | ironGate | Full NUCLEUS running, all ports available | CPU: light |
| darkforest (membrane) | ironGate | SSH to VPS, direct TCP probes | CPU: light, network: SSH |
| Nightly cargo-fuzz | southGate | 128 GB RAM, spare capacity, no primary science | CPU: heavy (multi-target parallel) |
| BTSP protocol fuzzing | ironGate or southGate | CPU-bound, needs RAM for corpus | CPU: heavy |
| Multi-gate probing | ironGate → others | Needs LAN access to all gates | CPU: light, network: LAN |
| VPS recovery test | ironGate | SSH + DO API access | CPU: light, network: WAN |

southGate (128 GB DDR4, 5800X3D 8c/16t) is the natural fuzz farm — enough
RAM for parallel jobs, not needed for primary science workloads, 10G NIC
for corpus replication when backbone activates.

---

## Integration with Existing Security Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│  deploy/security_validation.sh (5-layer, runs on demand)     │
│    ├── Layer 1: OS/network                                   │
│    ├── Layer 2: Primal API (MethodGate)                      │
│    ├── Layer 3: Application (JupyterHub)                     │
│    ├── Layer 4: ABG tier enforcement                         │
│    └── Layer 5: Dark Forest                                  │
│         ├── darkforest --local (existing)                    │
│         ├── darkforest --membrane (Phase 1 — NEW)            │
│         └── darkforest --remote --target X (Phase 4 — NEW)   │
├─────────────────────────────────────────────────────────────┤
│  deploy/nightly_fuzz.sh (cron, 2 AM daily — Phase 3)         │
│    ├── Per-primal cargo-fuzz targets (time-boxed)            │
│    ├── BTSP protocol targets (Phase 2)                       │
│    └── Crash → SkunkBat security.audit_log                   │
├─────────────────────────────────────────────────────────────┤
│  VPS recovery test (weekly cron — Phase 5)                   │
│    └── Destroy → reprovision → validate → measure time       │
└─────────────────────────────────────────────────────────────┘
```

---

## Priority Sequence

1. **Now**: Add BearDog TLS :8443 to existing darkforest probes (closes MEM-14)
2. **Next sprint**: darkforest `--membrane` mode (Phase 1)
3. **Following**: BTSP fuzz targets in bearDog/fuzz/ (Phase 2) + nightly CI (Phase 3)
4. **After 10G**: Multi-gate probing (Phase 4)
5. **Monthly cadence**: VPS recovery test (Phase 5)
6. **Ongoing**: Corpus sharing via NestGate (Phase 6)

---

*267+ PASS today. The goal is not "more checks" — it's coverage of every
attack surface the organism exposes. The outer membrane is the newest surface.
It needs the newest tests.*
