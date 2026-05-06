# Transport Matrix — projectNUCLEUS

**Generation**: gen4 — composition and deployment
**Upstream**: primalSpring `Phase 59 convergence`, wateringHole `PRIMAL_IPC_PROTOCOL.md`
**Scope**: Per-primal wire protocol, transports, BTSP, hash encoding, port table

---

## Principle

Every NUCLEUS primal communicates via JSON-RPC 2.0. The underlying transport
(UDS, TCP, HTTP) varies by primal and environment. The substrate must know
the transport characteristics of each primal to correctly health-check,
route, and validate communications.

## Wire Standard L3

Every primal's `capabilities.list` response includes `protocol` and
`transport` fields. This eliminates hardcoded assumptions about how to
reach a primal:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "name": "beardog",
    "version": "0.8.x",
    "protocol": "jsonrpc",
    "transport": ["uds", "tcp"],
    "capabilities": ["security.encrypt", "security.decrypt", "btsp.negotiate"]
  }
}
```

## Per-Primal Transport Table

| Primal | TCP Port | Primary Transport | JSON-RPC Framing | BTSP Required | Notes |
|--------|----------|-------------------|-------------------|---------------|-------|
| BearDog | 9100 | UDS, TCP | Newline-delimited | Self (origin) | Must start first; provides BTSP to all others |
| Songbird | 9200 | UDS, TCP, HTTP | HTTP JSON-RPC | Yes | `ipc.resolve` / `ipc.register`; HTTP on UDS |
| Squirrel | 9300 | UDS, TCP | Newline-delimited | Yes | AI coordination; filesystem-scoped sockets |
| ToadStool | 9400 | UDS, TCP | Newline-delimited | Yes | Workload dispatch; `sandbox.*` methods |
| NestGate | 9500 | TCP | Newline-delimited TCP | Yes (JWT also) | `storage.*`; newline-delimited — not HTTP |
| rhizoCrypt | 9601 (tarpc) / 9602 (JSON-RPC) | UDS, TCP | Newline-delimited | Yes | DAG anchoring; `dag.*` — **dual-port**: tarpc on base, JSON-RPC on base+1 |
| loamSpine | 9700 | UDS, TCP, HTTP | HTTP JSON-RPC | Yes | Ledger; hex hash encoding at boundary |
| coralReef | 9730 | UDS, TCP | Newline-delimited | Yes | GPU shader; `--rpc-bind` env config |
| barraCuda | 9740 | UDS, TCP | Newline-delimited | Yes | Tensor/math; `--rpc-bind` env config |
| skunkBat | 9140 | UDS, TCP | Newline-delimited | Yes | Defensive/monitoring |
| biomeOS | 9800 | UDS, TCP | Newline-delimited | Yes | Neural API orchestration |
| sweetGrass | 9850 | TCP (BTSP) | BTSP-encrypted TCP | Yes (strict) | Braid verification; 39085 is deprecated legacy |
| petalTongue | 9900 | UDS, TCP | Newline-delimited | Yes | Visualization/UI; `server` mode JSON-RPC UDS + TCP |
| sourDough | — | — | — | — | Scaffolding/starter culture for new primals. No binary yet (manifest only). |

## BTSP Phase 3 — Encrypted by Default

All 13 live primals implement `btsp.negotiate` with ChaCha20-Poly1305 AEAD.
(sourDough is in plasmidBin manifest but has no binary — scaffolding primal.)
After negotiation, all traffic on that connection is encrypted.

**Critical transport rule**: After `btsp.negotiate` completes, the transport
**must switch to encrypted frames**. The BufReader used during negotiation
must persist — do not create a new reader (data loss from buffered bytes).

### BTSP Interaction Patterns

| Pattern | Example |
|---------|---------|
| Direct BTSP | `beardog → rhizocrypt` (both have BTSP) |
| BTSP via Tower | Product → BearDog BTSP → capability call forwarded |
| No BTSP | Development only (`NUCLEUS_INSECURE=1`) |

## JSON-RPC Framing Differences

Two framing styles exist. The substrate must use the correct one:

### Newline-delimited (most primals)

```
{"jsonrpc":"2.0","method":"health.liveness","id":1}\n
```

Response is a single JSON line terminated by newline. Use `nc` or raw TCP.

### HTTP JSON-RPC (Songbird, loamSpine, petalTongue)

```http
POST / HTTP/1.1
Content-Type: application/json

{"jsonrpc":"2.0","method":"health.liveness","id":1}
```

Response is an HTTP response with JSON body. Use `curl`.

## Hash Encoding Contract

At JSON-RPC boundaries, hashes may appear as hex strings or byte arrays:

| Primal | Hash Encoding | Example |
|--------|---------------|---------|
| rhizoCrypt | Hex string | `"abc123..."` (64 chars) |
| loamSpine | Hex string | Normalized at integration boundary |
| sweetGrass | Hex string | BLAKE3 merkle roots |
| NestGate | Opaque key string | `{domain}:{id}:{qualifier}` |

**Rule**: All BLAKE3 hashes at the projectNUCLEUS boundary are lowercase
hex strings. Internal `[u8; 32]` representations are primal-internal.

## Health Probing by Transport

`deploy.sh` probes health per transport type:

```bash
# Newline-delimited primals
echo '{"jsonrpc":"2.0","method":"health.liveness","id":1}' | nc -w2 localhost $PORT

# HTTP primals
curl -sf -X POST http://localhost:$PORT/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"health.liveness","id":1}'
```

## Port Override Hierarchy

1. Gate TOML (`gates/<name>.toml` `[ports]` section)
2. Environment variable (`BEARDOG_PORT=9100`)
3. Deploy graph `tcp_fallback_port` field
4. Default port table (this document)

## References

- wateringHole: `PRIMAL_IPC_PROTOCOL.md` (JSON-RPC contract)
- wateringHole: `UNIVERSAL_IPC_STANDARD_V3.md` (transport selection)
- wateringHole: `BTSP_PROTOCOL_STANDARD.md` (Phase 3 AEAD)
- primalSpring: `Phase 59 convergence` (port canonical table)
- wateringHole handoff: `PROVENANCE_TRIO_OPERATIONAL_HANDOFF_MAY2026.md`
- wateringHole handoff: `NESTGATE_STORAGE_PATTERNS_HANDOFF_MAY2026.md`
