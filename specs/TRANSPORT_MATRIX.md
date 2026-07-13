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

## Spore Validation Columns (U/V/W)

Columns U, V, W extend the primalSpring NUCLEUS Validation Matrix
(`specs/NUCLEUS_VALIDATION_MATRIX.md`) with spore lifecycle operations.
These define which primals participate when pseudoSpores flow through
the NUCLEUS substrate.

### Column Definitions

| Column | Operation | What to Validate | Signal Graph |
|--------|-----------|-----------------|--------------|
| **U: Spore Ingest** | `biomeos nucleus ingest <dir>` | 6-step sequential: validate envelope → store → DAG session → ledger entry → braid → sign receipt | `nest_ingest_spore.toml` |
| **V: Spore Emit** | `biomeos nucleus emit` | Retrieve content → assemble envelope → sign | — |
| **W: Domain Profile** | `litho emit-pseudospore --spring X --domain-profile Y` | Spring's `domain_profile.toml` produces valid pseudoSpore with BLAKE3 manifest | — |

### Per-Primal Spore Participation

| Primal | U: Ingest | V: Emit | W: Profile |
|--------|-----------|---------|------------|
| BearDog | `crypto.sign` (receipt) | `crypto.sign` (envelope) | — |
| Songbird | — | — | — |
| skunkBat | — | — | — |
| Squirrel | — | — | — |
| ToadStool | — | — | — |
| NestGate | `storage.store` (content-addressed) | `storage.retrieve` (content) | — |
| rhizoCrypt | `dag.session.create` + `dag.event.append` | — | — |
| loamSpine | `entry.append` (permanent ledger) | — | — |
| coralReef | — | — | — |
| barraCuda | — | — | — |
| sweetGrass | `braid.create` (attribution) | — | — |
| biomeOS | orchestrate (6-step signal) | orchestrate (emit flow) | select profile |
| petalTongue | — | — | — |

### Provenance Model

Three-era spore provenance:

1. **Era 1 (ad-hoc)**: Hand-authored `scope.toml`, blind copy (v1.0–v1.6.0)
2. **Era 2 (pipeline-derived)**: Metadata extracted + cross-checked (v1.6.1)
3. **Era 3 (NUCLEUS nest deploy)**: Provenance trio signs via `nest_ingest_spore`
   signal (v2.0+) — content-addressed storage, DAG anchoring, permanent ledger,
   semantic attribution braid, Ed25519 receipt

Gate criterion: "Any spring can emit a pseudoSpore; any NUCLEUS can ingest it."

### projectNUCLEUS Integration

`nucleus-deploy spore` exercises the emit path (column V). The `spore/trio.rs`
module captures provenance via rhizoCrypt → loamSpine → sweetGrass. Column U
(ingest) is driven by `biomeos nucleus ingest` which consumes the signal graph.

Column W (domain profile) is spring-specific — not directly exercised by
projectNUCLEUS tools, but validated structurally by primalSpring scenarios
(`s_nest_atomic` Phase 4, `exp115_nest_ingest_pseudospore`).

## References

- wateringHole: `PRIMAL_IPC_PROTOCOL.md` (JSON-RPC contract)
- wateringHole: `UNIVERSAL_IPC_STANDARD_V3.md` (transport selection)
- wateringHole: `BTSP_PROTOCOL_STANDARD.md` (Phase 3 AEAD)
- wateringHole: `SPORE_OWNERSHIP_MATRIX.md` (domain/envelope/gateway split)
- primalSpring: `Phase 59 convergence` (port canonical table)
- primalSpring: `specs/NUCLEUS_VALIDATION_MATRIX.md` (columns A-X)
- wateringHole handoff: `PROVENANCE_TRIO_OPERATIONAL_HANDOFF_MAY2026.md`
- wateringHole handoff: `NESTGATE_STORAGE_PATTERNS_HANDOFF_MAY2026.md`
