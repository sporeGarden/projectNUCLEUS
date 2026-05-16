# Artifact Validation Playbook

**Date**: May 15, 2026
**Status**: Operational — ready to exercise against live NUCLEUS
**Relates to**: `EVOLUTION_GAPS.md` (Horizon 4), `SOVEREIGN_TRANSACTION_MEMBRANE.md`,
`TIER2_CEREMONY_DESIGN.md`, `SCIENCE_DISPATCH_MAP.md`, `FUZZ_EVOLUTION.md`

---

## Purpose

This document is the practical "run this, validate that" companion to the
architectural specs. For each long-term artifact the ecosystem produces,
it defines:

1. The **smallest testable unit** (one RPC call that proves the primitive works)
2. The **smallest composition** (multi-primal flow proving the artifact is real)
3. **2-3 use cases** (things a family/gate/user would actually do)
4. **2-3 science cases** (what upstream springs validate through it)
5. **benchScale integration** (topology/scenario for lab and cross-gate exercise)

Two benchScale systems participate:
- **NUCLEUS `infra/benchScale/`** — bash scenarios against real endpoints (7 parity, 3 pentest)
- **Rust `sort-after/benchScale/`** — Docker/libvirt labs with topology YAML and IPC compliance

agentReagents provisions VMs from YAML manifests and wires Rust benchScale for
lifecycle monitoring. Together they model full gate-to-gate interactions in
reproducible isolation.

---

## Artifact 1: Provenance Trio Pipeline

**Primals**: rhizoCrypt + loamSpine + sweetGrass (+ NestGate for content)

### Smallest Testable Unit

```
RPC: dag.event.append (rhizoCrypt :9700)
Request:  { "session_id": "<active_session>", "event": { "type": "data", "payload": "<blake3_hash>" } }
Response: { "vertex_id": "v-...", "merkle_root": "0x..." }
```

One vertex appended to an active DAG. Proves rhizoCrypt is accepting events and
computing Merkle roots.

### Smallest Composition

The `nest_store` signal graph (primalSpring `graphs/signals/nest_store.toml`):

```
content.put (NestGate) → event.append (rhizoCrypt) → spine.seal (loamSpine) → braid.create (sweetGrass)
```

4 sequential RPC calls across 4 primals. Proves the full provenance pipeline
from content ingestion through attribution.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **Science workload provenance** — lithoSpore LTEE run results flow through DAG → cert → braid | Every computation step is auditable back to source data |
| 2 | **Family recipe collection** — grandmother's recipes stored with contributor attribution | Non-technical content benefits from the same integrity guarantees |
| 3 | **Forgejo commit provenance** — code push triggers DAG append + cert | Sovereign version control has cryptographic integrity beyond git SHA |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | wetSpring | WCM parameter sweep — each parameter set = DAG vertex, final model = sealed spine, attribution braid captures all contributors | strandGate (256 GB ECC) |
| 2 | hotSpring | Lattice QCD β sweep — each β value run = vertex, phase transition result = cert, braid links to Kokkos parity evidence | biomeGate (Titan V) |
| 3 | foundation | Thread 5 LTEE — each generation = vertex (12 populations × 75k gen), fitness trajectory sealed as spine | any node_atomic |

### benchScale Integration

- **Topology**: `topologies/nucleus/provenance_trio.yaml` — 4 primals in Docker, basement_lan preset
- **Scenario**: `scenarios/artifact_validation.sh` Section 1 — exercises nest_store signal end-to-end
- **Validation**: primalSpring `s_nestgate_content_pipeline` + `s_full_nucleus` (Tier 2)

---

## Artifact 2: Novel Ferment Transcript (NFT)

**Primals**: rhizoCrypt (fermentation vessel) + loamSpine (bottling) + sweetGrass (attribution)

### Smallest Testable Unit

```
RPC sequence (3 calls minimum):
1. dag.session.create     → session_id (the vessel)
2. dag.event.append × N   → vertices accumulate (fermentation)
3. dag.dehydration.trigger → dehydrated_hash (ready for bottling)
```

Then loamSpine seals:
```
4. certificate.mint { "spine_hash": "<dehydrated_hash>", "type": "ferment_transcript" }
```

### Smallest Composition

```
[create vessel] → [N events over time] → [dehydrate] → [mint cert] → [create braid]
     rhizoCrypt              rhizoCrypt        rhizoCrypt      loamSpine     sweetGrass
```

The key distinction from the trio pipeline: time passes between events. The
ferment transcript's value comes from accumulated history, not a single
transaction.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **VPS lifecycle tracker** — every membrane event (deploy, cert renewal, reboot, resize) appends to the VPS's ferment DAG. After 6 months the transcript teaches skunkBat what "normal" looks like. Future SkunkBats on new gates inherit this learned normal. | Operational memory persists across gate generations |
| 2 | **Game save history** — 40 hours of Elden Ring play: 12 boss fights, 3 trades, 2 deaths per boss. The ferment transcript is the provable history. Trade it and the buyer inherits your journey. | Digital objects carry irreversible memory |
| 3 | **Experiment notebook** — a full Jupyter session (30 cells executed, 4 parameter sweeps, 1 failed run, 1 successful model). Sealed as a ferment, not just the final output. | Scientific reproducibility requires process, not just results |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | foundation | Thread 5 LTEE as a ferment — each generation is a vertex, fitness trajectory is the fermentation, final Tier 2 result is the sealed transcript | any node_atomic |
| 2 | healthSpring | Patient data pipeline — temporal integrity of medical events. The DAG proves ordering and completeness without revealing content (only hashes cross the membrane). | ironGate (ABG composition) |
| 3 | ludoSpring | Game session transcript — validates that ferment value correlates with accumulated play time and decision complexity, not just scarcity | southGate (gaming) |

### benchScale Integration

- **Topology**: `topologies/nucleus/ferment_lifecycle.yaml` — rhizoCrypt + loamSpine + sweetGrass + time-delay injection
- **Scenario**: `scenarios/artifact_validation.sh` Section 2 — creates a 10-event ferment, dehydrates, mints
- **Key metric**: DAG vertex count, dehydration time, cert size vs event count

---

## Artifact 3: Loam Certificate

**Primals**: loamSpine (minting) + rhizoCrypt (source spine) + BearDog (signing authority)

### Smallest Testable Unit

```
RPC: certificate.mint (loamSpine :9710)
Request:  { "spine_id": "<sealed_spine>", "cert_type": "ownership", "subject": "game:elden-ring:save-001" }
Response: { "cert_id": "cert-...", "hash_chain": ["0x...", ...], "issued_at": "2026-05-15T..." }
```

One cert minted from a sealed spine. Proves loamSpine can create tamper-evident
ownership records.

### Smallest Composition

```
[seal spine] → [mint cert] → [verify cert] → [transfer cert]
  loamSpine      loamSpine     BearDog          loamSpine
```

Transfer requires BearDog genetic authority to validate the new owner's lineage
is compatible with the cert's trust requirements.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **Sovereign game key** — cert proves you own Elden Ring without Steam DRM. Your gate validates locally. No phone-home. If Steam disappears, you still own your games. | Ownership without third-party dependency |
| 2 | **Creator credential** — artist mints a cert for their album. sweetGrass braid attributes all contributors. The cert is the proof of authorship, not a platform profile. | Identity via creation, not registration |
| 3 | **Gate enrollment certificate** — proves a gate is family-bonded. Required for covalent access (full trust). Without it, access is ionic (metered) or metallic (delocalized). | Trust boundary enforcement via cryptographic proof |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | lithoSpore | Module completion certificate — Tier 1 → Tier 2 graduation proof. The cert's spine contains the full validation run (all assertions PASS). | any node_atomic |
| 2 | hotSpring | Cross-gate experiment attestation — lattice QCD results certified by biomeGate (ran it) AND northGate (verified it). Multi-sig cert. | biomeGate + northGate |

### benchScale Integration

- **Topology**: `topologies/nucleus/provenance_trio.yaml` (loamSpine is a trio member)
- **Scenario**: `scenarios/artifact_validation.sh` Section 3 — mint + verify + attempt invalid transfer
- **Key metric**: Cert chain length, verification time, invalid transfer rejection rate

---

## Artifact 4: Tier 2 Key Ceremony

**Primals**: BearDog (genetics + entropy) + loamSpine (ceremony certificate) + HSM (hardware)

### Smallest Testable Unit

```
RPC sequence (3 calls):
1. genetic.ceremony_init      → ceremony_id, challenge, collection_window
2. genetic.entropy_contribute → { "source": "HumanLivedExperience", "data": "<typed_phrase_hash>" }
3. genetic.ceremony_finalize  → { "derived_key_id": "k-...", "entropy_quality": 0.94, "ceremony_cert": "cert-..." }
```

Three calls producing a sovereign key. The ceremony_cert is a Loam Certificate
proving the key's provenance.

### Smallest Composition

```
[init] → [human entropy] → [HSM entropy] → [finalize] → [mint ceremony cert] → [derive first child key]
BearDog     BearDog          BearDog         BearDog        loamSpine              BearDog
```

The ceremony produces both a key AND a certificate proving how the key was born.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **Personal sovereignty ceremony** — individual + SoloKey HSM. Produces a key that is provably derived from family seed + personal entropy. Required for gate enrollment. | Self-sovereign identity without any external authority |
| 2 | **Family seed rotation** — N family members each contribute entropy. Old seed is superseded. All child keys remain valid (derivation path unchanged). | Key infrastructure can evolve without breaking downstream |
| 3 | **Event ceremony (concert)** — artist at a stadium contributes venue entropy (crowd noise hash). 10,000 attendees derive event-specific keys. Proof of presence without surveillance. | Shared experiences produce cryptographic proof without tracking |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | neuralSpring | Entropy quality validation — is the human entropy actually adding min-entropy bits above the HSM baseline? Statistical analysis of ceremony outputs across 100 runs. | ironGate (HSM connected) |
| 2 | groundSpring | Ceremony reproducibility — given the same transcript (without secrets), can an auditor verify the ceremony followed protocol? Formal verification of the ceremony state machine. | any node_atomic |

### benchScale Integration

- **Topology**: `topologies/nucleus/tower_membrane.yaml` (BearDog is tower core)
- **Scenario**: `scenarios/artifact_validation.sh` Section 4 — ceremony_init + synthetic entropy + finalize
- **Note**: Real ceremonies require human presence. Lab exercises use synthetic entropy for protocol validation only (key material is discarded).

---

## Artifact 5: Steam Data Federation

**Primals**: NestGate (storage + replication) + Songbird (relay) + BearDog (access control)

### Smallest Testable Unit

```
RPC: content.put (NestGate gate A :9500) + content.get (NestGate gate B :9500)
Request A: { "dataset": "steam-saves", "key": "elden-ring/save-001", "data": "<base64>" }
Request B: { "dataset": "steam-saves", "key": "elden-ring/save-001" }
Response B: { "data": "<base64>", "blake3": "0x...", "source_gate": "southgate" }
```

Content written on one gate, read from another. Proves cross-gate replication works.

### Smallest Composition

```
[put save on southGate] → [replicate via 10G] → [get on northGate] → [verify BLAKE3] → [backup to VPS]
     NestGate                  NestGate sync         NestGate           NestGate          NestGate (VPS)
```

Full federation: local write → backbone replication → remote read → integrity check → off-site backup.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **Save file sync** — play Elden Ring on southGate (gaming PC), continue on northGate (office). Saves sync automatically over 10G backbone within seconds. | Sovereign Steam Cloud equivalent without Valve dependency |
| 2 | **VPS off-site backup** — saves replicate to cellMembrane VPS (157.230.3.183) overnight. If house burns down, saves survive. | Disaster recovery without cloud vendor lock-in |
| 3 | **Multi-household sharing** — flockGate (remote family) accesses shared saves via Songbird relay. Ionic bonding (metered trust) controls what's shared. | Cross-WAN federation with trust boundaries |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | hotSpring | Large dataset replication — 2 GB lattice QCD checkpoint replicated across gates. Validates 10G backbone throughput and BLAKE3 integrity at scale. | biomeGate → northGate |
| 2 | wetSpring | FASTQ file federation — 50 GB genomics data shared between strandGate (sequencing) and biomeGate (analysis). Proves NestGate handles science-scale blobs. | strandGate → biomeGate |

### benchScale Integration

- **Topology**: `topologies/nucleus/provenance_trio.yaml` (NestGate is first node in nest_store)
- **Scenario**: `scenarios/artifact_validation.sh` Section 5 — put + replicate + get + verify
- **Blocker**: 10G cables ($50) for real cross-gate testing. Until then, loopback or Docker bridge.
- **Key metric**: Replication latency (target <100ms for 1MB over 10G), BLAKE3 match rate (must be 100%)

---

## Artifact 6: sunCloud Metabolic Routing

**Primals**: sweetGrass (braids) + sunCloud (value routing) + loamSpine (settlement certs)

### Smallest Testable Unit

```
RPC: braid.create (sweetGrass :9720)
Request:  {
  "contributors": [
    { "id": "gate:irongate", "weight": 0.60 },
    { "id": "gate:northgate", "weight": 0.33 },
    { "id": "infra:membrane", "weight": 0.07 }
  ],
  "context": "hotspring-lattice-qcd-run-042"
}
Response: { "braid_id": "b-...", "metabolic_split": { "irongate": 0.60, "northgate": 0.33, "membrane": 0.07 } }
```

A braid with explicit contribution weights. The 7% membrane split is the
metabolic mandate — infrastructure always gets funded.

### Smallest Composition

```
[create braid] → [compute split] → [route value] → [settle cert]
  sweetGrass       sunCloud          sunCloud         loamSpine
```

The split computation is deterministic: same braid always produces same routing.
Settlement cert proves the split was executed.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **Infrastructure funding** — every ferment minted, every cert issued, every workload dispatched contributes 3-7% to membrane maintenance. No separate billing. | Sustainability without subscription fees |
| 2 | **Artist royalty distribution** — album with 4 contributors. Each play generates a braid. sweetGrass ensures attribution flows to all contributors proportionally. | Fair compensation without intermediaries |
| 3 | **Compute-for-heat credit** — GPU heat recovery during winter earns infrastructure credit. The braid links thermal output to compute work to infrastructure value. | Physical world contributions have digital representation |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | groundSpring | Attribution correctness — 10 contributors, known weights. 1000 braids computed. Verify splits match weights within floating-point tolerance. | any node_atomic |
| 2 | hotSpring | Split determinism — same braid input on 3 different gates produces identical routing. Proves cross-gate consensus without coordination. | ironGate + northGate + biomeGate |

### benchScale Integration

- **Topology**: `topologies/nucleus/provenance_trio.yaml` (sweetGrass in the trio)
- **Scenario**: `scenarios/artifact_validation.sh` Section 6 — create multi-contributor braid, verify deterministic split
- **Key metric**: Split precision (must match weights to 6 decimal places), cross-gate determinism (identical outputs)

---

## Artifact 7: BearDog Genetic Authority

**Primals**: BearDog (genetics + entropy + auth) + HSM (hardware entropy)

### Smallest Testable Unit

```
RPC: genetic.derive_key (BearDog :9100)
Request:  { "purpose": "gate-enrollment", "path": "family/irongate/2026", "entropy_class": "SystemRng" }
Response: { "key_id": "k-...", "derivation_path": "m/44'/0'/0'/0", "trust_ceiling": "SystemRng" }
```

One key derived from family seed for a specific purpose. The trust_ceiling
reflects the weakest entropy in the derivation chain.

### Smallest Composition

```
[derive gate key] → [sign enrollment cert] → [skunkBat learns identity] → [issue ionic token]
     BearDog            BearDog + loamSpine       skunkBat                    BearDog
```

The key becomes an identity. skunkBat uses it to define "self" (thymic
selection). Ionic tokens are derived for external parties.

### Use Cases

| # | Use Case | What It Proves |
|---|----------|---------------|
| 1 | **Gate enrollment** — new gate derives its identity from family root. Without this, the gate is untrusted (weak bonding only). | Trust is derived from lineage, not registration |
| 2 | **skunkBat thymic training** — BearDog identity defines "self". skunkBat trains detectors against this definition. Anything not provably self is potential foreign. | Security from identity, not signatures |
| 3 | **Ionic token issuance** — ABG collaborator gets a metered access token. Token expires. Token cannot escalate to covalent access. Trust ceiling enforced by entropy hierarchy. | Graduated trust without all-or-nothing |

### Science Cases

| # | Spring | What's Validated | Gate |
|---|--------|-----------------|------|
| 1 | neuralSpring | Key derivation path uniqueness — derive 10,000 keys with different purposes. Verify zero collisions. Statistical analysis of output distribution. | any node_atomic |
| 2 | groundSpring | Entropy hierarchy enforcement — attempt to create a key with trust_ceiling "HardwareHSM" using only SystemRng. Verify rejection. | ironGate (HSM connected) |
| 3 | airSpring | Ionic token lifecycle — issue 100 tokens, expire 50, attempt use of expired. Verify 100% rejection of expired tokens. | any node_atomic |

### benchScale Integration

- **Topology**: `topologies/nucleus/tower_membrane.yaml` (BearDog is the tower authority)
- **Scenario**: `scenarios/artifact_validation.sh` Section 7 — derive + sign + verify + expire
- **Key metric**: Derivation time (<5ms), collision rate (must be 0), expired token rejection (must be 100%)

---

## Cross-Artifact Interaction Matrix

These artifacts compose. The matrix shows which artifacts require which others:

| Artifact | Requires | Enables |
|----------|----------|---------|
| Provenance Trio | BearDog (signing) | NFT, Loam Cert, sunCloud |
| NFT | Provenance Trio (DAG + cert) | skunkBat learning, game history |
| Loam Certificate | Provenance Trio (spine source) | Game key, gate enrollment, creator cred |
| Tier 2 Ceremony | BearDog (entropy), loamSpine (cert) | All key material, gate enrollment |
| Steam Federation | NestGate (storage), BearDog (access) | Cross-gate gaming, disaster recovery |
| sunCloud | sweetGrass (braids), loamSpine (settle) | Infrastructure sustainability |
| BearDog Genetics | HSM (hardware) | Everything else (root of trust) |

**Dependency order for validation**: BearDog → Provenance Trio → Loam Cert → NFT → Federation → sunCloud → Ceremony

---

## Testing Across Basement Metal vs VMs

### Basement Metal (real gates)

Use NUCLEUS `infra/benchScale/scenarios/artifact_validation.sh` against live primals:
- Fastest feedback (no VM boot)
- Real network conditions (10G backbone when cabled)
- Real HSM interaction (SoloKey on ironGate)
- Thermal/power effects visible

### Docker Labs (Rust benchScale)

Use `sort-after/benchScale/topologies/nucleus/*.yaml`:
- Reproducible isolation (no state leakage between runs)
- Network condition simulation (latency injection, packet loss)
- Parallel labs (run 4 topology variants simultaneously)
- CI-friendly (no hardware dependency)

### VM Substrates (agentReagents)

Use `sort-after/agentReagents/templates/nucleus_gate.yaml`:
- Full OS-level isolation (kernel, filesystem, networking)
- Cloud-init provisioning (matches real gate bootstrap)
- Multi-gate simulation (provision 3 VMs = 3 gates)
- Closest to production (VM ≈ real gate minus hardware)

### When to Use Which

| Scenario | Tool | Why |
|----------|------|-----|
| Quick artifact smoke test | basement metal | Fastest, all primals already running |
| Multi-gate interaction | Docker topology | Reproducible, parallel, CI |
| Full gate lifecycle (boot → deploy → validate) | agentReagents VM | OS-level fidelity |
| Network degradation testing | Docker + presets | Fine-grained latency/loss control |
| HSM/ceremony testing | basement metal only | Requires physical hardware |
| Cross-architecture (ARM mobile simulation) | agentReagents | Can provision ARM QEMU |

---

## What Upstream Springs Validate Through NUCLEUS

Springs don't test NUCLEUS directly. They test **through** it:

| Spring | Uses NUCLEUS For | Key Validation |
|--------|-----------------|----------------|
| primalSpring | Composition health, method discovery, signal graphs | 35 scenarios in `ecoPrimal/src/validation/scenarios/` |
| hotSpring | toadStool dispatch of GPU workloads, provenance of results | Lattice QCD run → DAG → cert chain |
| wetSpring | Large blob federation (FASTQ), WCM parameter provenance | NestGate + rhizoCrypt on strandGate |
| lithoSpring | Module completion certs, Tier 1→2 graduation | loamSpine cert minting after validation PASS |
| foundation | LTEE generation tracking, thread provenance | Ferment transcripts for 75k-generation runs |
| groundSpring | Attribution correctness, formal verification targets | sweetGrass braid determinism proofs |
| neuralSpring | Entropy quality, key derivation statistics | BearDog genetic authority + ceremony outputs |
| airSpring | Token lifecycle, cross-gate security posture | Ionic token issuance + expiry + rejection |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-05-15 | Initial playbook. 7 artifacts mapped with units, compositions, use cases, science cases. |
