# Validation Results — Phase 1-2a Pipeline Validation (2026-05-06)

Concrete evidence from the first full local validation run on the active gate.
This document records what worked, what didn't, and what it means for
rigor, reproducibility, and security.

---

## Composition

13 primals deployed via `deploy.sh --composition full --gate <active-gate>`:

| Primal | Port | Health | Transport |
|--------|------|--------|-----------|
| BearDog | 9100 | alive (nc) | Newline-delimited JSON-RPC |
| Songbird | 9200 | alive (GET /health) | HTTP |
| Squirrel | 9300 | alive (nc) | Newline-delimited JSON-RPC |
| ToadStool | 9400 | alive (HTTP POST) | HTTP JSON-RPC |
| NestGate | 9500 | alive (nc) | Newline-delimited JSON-RPC |
| rhizoCrypt | 9601 (tarpc) / 9602 (JSON-RPC) | alive (nc 9602) | Dual-port: tarpc base, JSON-RPC base+1 |
| loamSpine | 9700 | alive (HTTP POST) | HTTP JSON-RPC |
| coralReef | 9730 | alive (nc) | Newline-delimited JSON-RPC |
| barraCuda | 9740 | alive (nc) | Newline-delimited JSON-RPC |
| sweetGrass | 9850 | alive (nc) | Newline-delimited JSON-RPC |
| skunkBat | 9140 | alive (nc) | Newline-delimited JSON-RPC |
| biomeOS | 9800 | alive (nc) | Newline-delimited JSON-RPC |
| petalTongue | 9900 | alive (nc) | HTTP |

Family ID: `9b32f3a8` (pop-os-sovereign)

---

## Workload Results

11 wetSpring workloads dispatched through ToadStool CLI (`toadstool execute <toml>`):

| Workload | Checks | Duration | Status | Output BLAKE3 |
|----------|--------|----------|--------|---------------|
| wetspring-16s-rust-validation | 37/37 | <1s | PASS | `5cee126110521afb` |
| wetspring-algae-16s-rust | 34/34 | 23s | PASS | `cb214cbfb0b74227` |
| wetspring-diversity-rust-validation | 27/27 | <1s | PASS | `27b6937d809c4f18` |
| wetspring-gonzales-cpu-parity | 43/43 | <1s | PASS | `d069f714b803ee0a` |
| wetspring-r-industry-parity | 53/53 | <1s | PASS | `a0b2c33d799ff7dd` |
| wetspring-real-ncbi-pipeline | 25/25 | <1s | PASS | `cd9690f0d79dfc81` |
| wetspring-fajgenbaum-pathway | 8/8 | <1s | PASS | `3d4d6cc1acfcf11f` |
| wetspring-cold-seep-pipeline | 8/8 | <1s | PASS | `6fc90d282786f388` |
| wetspring-16s-python-baseline | 0/0 | 2s | RUN | `9bf0e25610a50687` |
| wetspring-benchmark-python-baseline | 0/0 | 4s | RUN | `d531007b106c177f` |
| wetspring-exp001-python-baseline | 0/1 | <1s | FAIL | `0fdee90d580d6420` |

**Totals**: 8 PASS, 2 RUN (baselines without assertion counts), 1 FAIL.
235 checks passed, 1 check failed.

### Failure Analysis

**wetspring-exp001-python-baseline**: Failed with `Data dir missing: /tmp/MiSeq_SOP`.
This is a data dependency — the MiSeq Standard Operating Procedure dataset
must be downloaded and staged before running. Not a code or composition bug.
The workload TOML is correct; the prerequisite data wasn't provisioned.

### RUN Status

The two Python baselines report `RUN` (not `PASS`) because they emit timing
and output data without structured `[OK]`/`[FAIL]` assertion lines. The
provenance pipeline recorded them with exit code 0 and valid BLAKE3 output
hashes. These are observation workloads, not validation workloads.

---

## Provenance Chain

The full 9-phase provenance pipeline completed end-to-end:

### Phase Execution

| Phase | Operation | Result |
|-------|-----------|--------|
| 1 | Health checks (13 primals) | 13/13 OK |
| 2 | rhizoCrypt DAG session create | Session `019dfe5d-c17f-7a93-889e-01bf813ee7f8` |
| 3 | loamSpine spine create | Spine `019dfe59-52ff-7da1-ad89-a7c8751d8c3e` |
| 4 | Register NCBI data artifacts (4 FASTQs, BLAKE3) | 4 artifacts, 5.4 GB total |
| 5 | Execute workloads with DAG+ledger tracking | 11 workloads, 26 events |
| 6 | Dehydrate DAG → Merkle root | `b106aa1d1bb45430d00d605626e10488119f9e4f9f315a738939049a6da9ceec` |
| 7 | loamSpine permanent commit | Index 32 |
| 8 | sweetGrass attribution braid | `urn:braid:b106aa1d...` with ed25519 witness |
| 9 | Write manifest + braid JSON | `/tmp/provenance_results/` |

### Data Artifacts

| Key | BLAKE3 Hash | Size |
|-----|-------------|------|
| ncbi:SRR7760408:R1 | `6250f200f9ff45e0f3aa52ede78dbe4ad4a68dd1a55b355d7502b02afeaa672a` | 2.2 GB |
| ncbi:SRR7760408:R2 | `cd89f43d74d09c64b4c832040f0cc04837c30bf7bb897f083dcd89ee6ece1d7c` | 2.4 GB |
| ncbi:SRR5534045:R1 | `096878541679cd066ffa873ac024c7ca3089f4e5df0e6c81dbe05ed64acaeb30` | 444 MB |
| ncbi:SRR5534045:R2 | `bee510af71ac914a5442492574f57b02b6a490eabeecce9d06242c333d9e1d7d` | 451 MB |

Source: Nannochloropsis outdoor 16S (Wageningen), PRJNA488170 / PRJNA382322.
DOI: 10.1007/s00253-022-11815-3.

### Braid

```json
{
  "@id": "urn:braid:b106aa1d1bb45430d00d605626e10488119f9e4f9f315a738939049a6da9ceec",
  "@type": "Entity",
  "data_hash": "b106aa1d...",
  "mime_type": "application/x-provenance-pipeline",
  "was_attributed_to": "did:primal:b741dfd4-63b5-4051-a460-7b830cc6a6a5",
  "witness": {
    "agent": "did:key:z6MkL98JJsBioodl4bTwDjoorgsfIw2Xb4QdKo0ey-CswJA",
    "algorithm": "ed25519",
    "evidence": "X/qEhmorI7zFFa89+C39NCluc6ZG+v/i...",
    "kind": "signature",
    "tier": "tower"
  }
}
```

PROV-O compliant (`prov:`, `schema:`, `xsd:` context). DID attribution.
Content-addressed URN from Merkle root. Tower-tier ed25519 witness.

---

## Rigor Assessment

### What Is Rigorous

- **Content-addressed hashing**: Every input and output has a BLAKE3 hash.
  Tamper with one byte and the hash changes, the Merkle root changes,
  the braid witness invalidates.
- **235 structured checks**: Not "it ran without crashing" — each workload
  has named assertions (`[OK] DADA2 distinct → 2 ASVs`, etc.) with
  expected values and tolerances.
- **Merkle root over DAG**: The 26-event DAG dehydrates to a single root
  hash. This covers all data registrations and workload results in one
  integrity proof.
- **ed25519 witness**: The braid carries a cryptographic signature from
  BearDog's key hierarchy. The signature is verifiable by anyone with
  the public key (`did:key:z6MkL98JJs...`).

### What Is Not Yet Rigorous

| Gap | Impact | Fix |
|-----|--------|-----|
| `ecop.loam_anchor` is null | Braid references ledger by ID but doesn't embed the commit hash. Cross-primal verification requires two queries. | sweetGrass should embed the loamSpine entry hash in the braid. |
| `ecop.witnesses` array empty | Only one witness (sweetGrass's own ed25519). No multi-witness (e.g., BearDog co-sign). | Wire BearDog as secondary witness on braid creation. |
| `ecop.certificate` is null | No X.509 or BTSP certificate chain embedded. | Phase 3: BTSP certificate chain in braid metadata. |
| `was_generated_by` is null | Pipeline that generated the braid isn't self-referentially recorded. | Set `was_generated_by` to the pipeline session DID. |
| No automated re-verification | Manifest says "re-run to verify" but no script automates the comparison. | Add `verify_provenance.sh` that re-hashes inputs and compares to manifest. |

---

## Reproducibility Assessment

### What Is Reproducible

- **NCBI accessions are permanent**: SRR7760408, SRR5534045 are public
  NCBI SRA records. Anyone can download the same reads.
- **BLAKE3 hashes pin exact bytes**: If the downloaded FASTQs match the
  hashes above, you have the exact same input data.
- **Workload TOMLs are declarative**: Each workload specifies the binary,
  working directory, and resource limits. Running the same TOML through
  ToadStool on any machine with the same binary should produce the same
  output.
- **Output hashes verify results**: After re-running, compare output
  BLAKE3 hashes. Identical means bit-for-bit reproducible.

### What Is Not Reproducible (By Design)

- **Merkle root changes per run**: DAG session IDs include timestamps
  and UUIDs. Two runs of the same workloads produce different Merkle
  roots. This is correct — you want to distinguish run A from run B.
- **Braid URN changes per run**: Content-addressed from the Merkle root,
  so it changes too. The braid is per-run provenance, not per-experiment.

### Reproducibility Protocol

To reproduce this validation:
1. Download NCBI FASTQs, verify BLAKE3 hashes match the table above
2. Build wetSpring Rust binaries (or use plasmidBin)
3. Deploy full composition: `deploy.sh --composition full --gate <active-gate>`
4. Run pipeline: `provenance_pipeline.sh --workloads-dir workloads/wetspring`
5. Compare per-workload output BLAKE3 hashes against the table above
6. If all match, science is reproduced. Merkle root and braid will differ (expected).

---

## Security Assessment

### What Is Secure

| Layer | Evidence |
|-------|----------|
| Family seed cryptography | BearDog ed25519 witness is real, verifiable |
| BTSP enforcement | rhizoCrypt rejects non-BTSP TCP connections (confirmed by log) |
| Content integrity | BLAKE3 hashing of all artifacts — tamper-evident |
| DID attribution | Braid attributed to `did:primal:b741dfd4-...` |

### What Is Not Yet Secure

| Gap | Risk | Mitigation Timeline | Status |
|-----|------|---------------------|--------|
| ToadStool security incomplete | Warns "incomplete cryptographic verification" | ToadStool team evolution | Open (upstream) |
| Workload isolation = None | Workloads execute as host user | Phase 2: cgroups + namespaces | Open |
| Single witness | No multi-witness or certificate chain | Phase 2b: BearDog co-sign | Open |
| NestGate TCP fallback ungated | TCP fallback (Tier 5, localhost) dispatches all methods ungated | MethodGate on 13/13 primals mitigates | Mitigated |
| ~~JupyterHub plain HTTP~~ | ~~No BTSP on localhost~~ | ~~Phase 2a: Cloudflare tunnel (external TLS)~~ | **RESOLVED** — cell membrane (tunnel TLS) |
| ~~No ionic token scoping~~ | ~~Tokens advisory, not enforcing~~ | ~~Phase 60: MethodGate enforcement~~ | **RESOLVED** — Phase 60 (2026-05-08) |
| ~~Cross-primal token federation~~ | ~~No cross-gate token verification~~ | ~~JH-11~~ | **RESOLVED** — BearDog `auth.public_key` + biomeOS `BearDogVerifier` |
| ~~toadStool/squirrel MethodGate~~ | ~~No pre-dispatch auth gate on 2 primals~~ | ~~Upstream handback~~ | **RESOLVED** — 13/13 MethodGate |

**Resolved (primalSpring Phase 59)**: All 13/13 primals default `127.0.0.1` bind. Fully closed.
**Resolved (Phase 60+)**: MethodGate enforced on 13/13 primals. toadStool + squirrel resolved upstream after deep debt handback.
**Resolved (2026-05-10)**: Cell membrane architecture — external TLS via Cloudflare tunnel for lab/git subdomains.
**Resolved (2026-05-11)**: NestGate Session 60 — `content.*` transport parity on all 4 surfaces. All per-primal composition debt closed. L1 CLEAN.

### Security Evolution Path

See `specs/TUNNEL_EVOLUTION.md` for the systematic replacement plan.
Each external security dependency (Cloudflare TLS, PAM auth) maps to a
primal replacement with measurable validation targets.

---

## Bugs Fixed During Validation

| Bug | File | Root Cause | Fix |
|-----|------|-----------|-----|
| rhizoCrypt/loamSpine/sweetGrass/squirrel reject `--family-id` | deploy.sh | CLI doesn't accept this arg; uses env vars | Removed `--family-id` from launch commands |
| rhizoCrypt rejects all TCP/UDS without FAMILY_SEED | deploy.sh | BTSP requires seed env var to accept connections | Added `export FAMILY_SEED="$BEACON_SEED"` |
| rhizoCrypt JSON-RPC on port+1, not base port | provenance_pipeline.sh | tarpc on 9601, JSON-RPC on 9602 (undocumented) | Target `$((RHIZOCRYPT_PORT + 1))` for JSON-RPC |
| Songbird health check fails via JSON-RPC POST | provenance_pipeline.sh | Songbird uses HTTP GET `/health`, not JSON-RPC | Switch to `curl GET /health` |
| sweetGrass RPC fails via HTTP POST | provenance_pipeline.sh | sweetGrass uses newline-delimited TCP, not HTTP | Switch to `nc` |

### Upstream Issues Identified

| Issue | Owner | Severity |
|-------|-------|----------|
| rhizoCrypt dual-port undocumented | rhizoCrypt / primalSpring | High — deploy graphs list 9601 but JSON-RPC is 9602 |
| ToadStool JSON-RPC `submit_workload` schema undocumented | ToadStool / wateringHole | High — CLI vs JSON-RPC impedance mismatch |
| ToadStool security warnings in production mode | ToadStool team | Medium |
| exp001 depends on unprovisioned data (`/tmp/MiSeq_SOP`) | wetSpring team | Low |
| Transport matrix inconsistencies across primals | primalSpring / wateringHole | Medium — specs say "TCP" but actual transport varies |
