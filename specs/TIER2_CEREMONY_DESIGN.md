# Tier 2 Ceremony Design — Human Entropy Protocol

**Date**: May 15, 2026
**Status**: Specification — BearDog genetics infrastructure exists, ceremony protocol pending
**Relates to**: `NUCLEUS_TWO_TIER_CRYPTO_MODEL.md` (Tier 2 placeholder),
`ENTROPY_HIERARCHY_PRINCIPLE.md`, `beardog-genetics/src/genetics/entropy_hierarchy/`,
`gen4/architecture/SOVEREIGN_TRANSACTION_MEMBRANE.md` (Section 2)

---

## Purpose

This spec defines the **operational protocol** for Tier 2 key ceremonies
in projectNUCLEUS. The whitePaper architecture doc describes the WHY and
the abstract model. This doc describes the WHAT — specific BearDog RPC
calls, gate hardware involvement, entropy requirements, and output formats.

---

## Ceremony Types

### Type 1: Personal Sovereignty Ceremony

**Goal**: Individual derives a sovereign key from family material + personal entropy.
**Participants**: 1 human + 1 HSM
**Output**: Tier 2 sovereign key (owned by individual, provably family-derived)

### Type 2: Family Seed Ceremony (rotation/creation)

**Goal**: Create or rotate the family root key.
**Participants**: N family members + HSMs
**Output**: New family root key (replaces prior on rotation)

### Type 3: Event Ceremony (artist/audience)

**Goal**: Create event-specific key material tied to physical presence.
**Participants**: 1 authority (artist) + N attendees
**Output**: Event key + per-attendee derived certificates

### Type 4: Collaborative Creation Ceremony

**Goal**: Shared key for a multi-creator project.
**Participants**: N creators (all equal authority)
**Output**: Project key (all creators can derive from it)

---

## Protocol: Personal Sovereignty Ceremony (Type 1)

### Prerequisites

- Gate running full NUCLEUS (ironGate or equivalent)
- BearDog `genetic.*` RPC methods available
- SoloKey FIDO2 (or equivalent HSM) connected via USB
- Family seed present (`~/.config/biomeos/family/.beacon.seed`)
- Human operator at keyboard (physical presence required)

### Step 1: Ceremony Initialization

```
RPC: genetic.ceremony_init
Request:
  {
    "ceremony_type": "personal_sovereignty",
    "participant_count": 1,
    "required_entropy_classes": ["HumanLivedExperience", "HardwareHSM"],
    "hsm_device": "/dev/hidraw0"
  }
Response:
  {
    "ceremony_id": "urn:ceremony:a7b3c...",
    "challenge": "<random_bytes_hex>",
    "collection_window_seconds": 120,
    "status": "collecting"
  }
```

### Step 2: Entropy Collection

BearDog opens a collection window. During this window:

**HSM contribution** (Tier 1 — HardwareHSM):
```
RPC: crypto.hsm_sign_challenge
Request:
  {
    "challenge": "<challenge_from_step_1>",
    "device": "/dev/hidraw0"
  }
Response:
  {
    "signature": "<ed25519_signature>",
    "device_attestation": "<fido2_attestation>",
    "entropy_class": "HardwareHSM"
  }
```

**Human behavioral entropy** (Tier 3 — HumanLivedExperience):
```
RPC: genetic.collect_human_entropy
Request:
  {
    "ceremony_id": "urn:ceremony:a7b3c...",
    "source": "keyboard_timing",
    "min_events": 64,
    "timeout_seconds": 60
  }
Response:
  {
    "entropy_bytes": 32,
    "events_collected": 87,
    "timing_variance_ms": 142.7,
    "entropy_class": "HumanLivedExperience",
    "quality_score": 0.94
  }
```

BearDog validates:
- HSM attestation matches known SoloKey
- Human entropy quality_score > 0.7 (sufficient timing variance)
- No simulated entropy detected (monotonic timing, zero variance = rejected)
- Collection window respected (entropy arrived within 120s)

### Step 3: Entropy Mixing

```
RPC: genetic.mix_entropy
Request:
  {
    "ceremony_id": "urn:ceremony:a7b3c...",
    "sources": [
      {"class": "HardwareHSM", "ref": "hsm_sig_hash"},
      {"class": "HumanLivedExperience", "ref": "keyboard_entropy_hash"}
    ],
    "mixing_strategy": "hkdf_sha256",
    "purpose": "sovereign-tier2-v1"
  }
Response:
  {
    "seed_id": "urn:seed:f9e2d...",
    "entropy_class": "HumanLivedExperience",
    "source_count": 2,
    "mixing_hash": "<blake3_of_mixed_material>",
    "provenance": {
      "ceremony_id": "urn:ceremony:a7b3c...",
      "timestamp": "2026-05-15T21:30:00Z",
      "gate": "irongate",
      "sources_summary": "1xHSM + 1xHuman (keyboard, 87 events)"
    }
  }
```

### Step 4: Sovereign Key Derivation

```
RPC: genetic.derive_sovereign_key
Request:
  {
    "seed_id": "urn:seed:f9e2d...",
    "family_key_purpose": "sovereignty",
    "human_identifier": "operator_a",
    "bind_to_gate": "irongate"
  }
Response:
  {
    "sovereign_key_id": "urn:key:sovereign:3a8b1...",
    "derivation_proof": {
      "family_lineage": true,
      "entropy_class": "HumanLivedExperience",
      "ceremony_id": "urn:ceremony:a7b3c...",
      "timestamp": "2026-05-15T21:30:05Z"
    },
    "capabilities": [
      "sign_certificates",
      "derive_purpose_keys",
      "create_ionic_tokens",
      "initiate_ceremonies"
    ],
    "cannot_be_derived_by": "family_seed_alone",
    "revocation": "owner_only (via genetic.revoke_sovereign_key)"
  }
```

### Step 5: Ceremony Record (Provenance)

```
RPC: loamspine.entry.append
Request:
  {
    "spine": "ceremonies",
    "entry_type": "ceremony_completion",
    "content": {
      "ceremony_id": "urn:ceremony:a7b3c...",
      "type": "personal_sovereignty",
      "output_key_id": "urn:key:sovereign:3a8b1...",
      "entropy_sources": 2,
      "entropy_class": "HumanLivedExperience",
      "gate": "irongate",
      "timestamp": "2026-05-15T21:30:05Z"
    },
    "sign_with": "purpose_key:ledger"
  }
```

---

## Protocol: Event Ceremony (Type 3)

### Prerequisites

- Artist's gate running full NUCLEUS
- Artist's SoloKey connected
- Songbird relay active (VPS outer membrane for NAT traversal)
- BearDog ceremony mode enabled
- Attendee devices: any device running BearDog (phone, laptop, tablet)

### Differences from Type 1

| Aspect | Personal (Type 1) | Event (Type 3) |
|--------|-------------------|----------------|
| Authority | Individual | Artist (elevated) |
| Participants | 1 | 1 + N attendees |
| HSM required from | The individual | The artist only |
| Attendee entropy class | N/A | StoreBoughtMachine (device RNG) |
| Output keys | 1 sovereign key | 1 event key + N attendee derivations |
| Attenuation | None (permanent) | Time-bound decay |
| Transport | Local (gate) | Songbird relay (NAT traversal) |

### Event Ceremony Flow

```
1. Artist: genetic.ceremony_init (type: "event", max_attendees: 500)
   → ceremony_id, join_code (short alphanumeric for audience)

2. Attendees: connect via Songbird relay using join_code
   → BearDog validates: device entropy contributed
   → Each device provides: /dev/urandom sample (StoreBoughtMachine)

3. Artist: contributes HSM + performance timing entropy
   → HumanLivedExperience (performance gestures, button presses)

4. Mixing: genetic.mix_entropy (artist_sources + all attendee_sources)
   → event_entropy_seed
   → Validation: at least 1 HumanLivedExperience source (artist)

5. Derivation: genetic.derive_event_key
   → event_key = HMAC-SHA256(artist_family_key + event_seed, "event-v1:...")
   → For each attendee i:
       attendee_cert_i = HMAC-SHA256(event_key, "attendee:..." + device_hash_i)

6. Minting: loamSpine certificates for each attendee
   → Cert proves: device was present at ceremony mixing moment
   → Capabilities: access event content, prove attendance
   → Restrictions: non-transferable (bound to device hash)

7. Attenuation schedule (configurable by artist):
   → Day 0-30: full content access (ceremony bond)
   → Day 30-365: scoped access (ionic, view only)
   → Day 365+: public content, cert remains as proof-of-presence
```

---

## Hardware Requirements

| Ceremony Type | Minimum Hardware | Recommended |
|--------------|-----------------|-------------|
| Personal sovereignty | Gate + SoloKey USB | ironGate + SoloKey + quiet room |
| Family seed rotation | Any gate + all family SoloKeys | ironGate + all HSMs present |
| Event ceremony | Gate + SoloKey + Songbird relay | ironGate + VPS + artist HSM |
| Collaborative | Any gate with all creators connected | strandGate (many connections) |

### Gate Suitability

| Gate | Ceremony Role | Why |
|------|--------------|-----|
| ironGate | Primary ceremony host | Full NUCLEUS, ABG-facing, SoloKey accessible |
| northGate | Backup / family events | i9-14900K for many concurrent connections |
| VPS | Relay only (NO key material) | Songbird relay for NAT traversal |
| NUC M6 | Witness node (Type 2 family) | Isolated covalent pair for key witnessing |

**Critical**: The VPS outer membrane NEVER holds key material. It relays
encrypted ceremony traffic. All entropy mixing and key derivation happens
intracellularly (on gate hardware the family controls).

---

## Security Properties

| Property | Mechanism |
|----------|-----------|
| **Non-reproducibility** | HumanLivedExperience entropy is non-fungible (unique timing) |
| **Family membership proof** | sovereign_key derivable from family_key path (HKDF chain) |
| **Individual ownership** | Human entropy cannot be reproduced by family alone |
| **Ceremony provenance** | loamSpine entry records ceremony metadata (not key material) |
| **Revocation** | Owner-only via `genetic.revoke_sovereign_key` |
| **Simulation rejection** | BearDog entropy hierarchy rejects StoreBoughtMachine for Tier 2 |
| **Presence proof (events)** | Device entropy + Songbird connection + timing = co-location |
| **Attenuation** | Event keys time-bound; capabilities shrink per schedule |

---

## Implementation Status

| Component | Status | Location |
|-----------|--------|----------|
| Entropy hierarchy types | Implemented | `beardog-genetics/src/genetics/entropy_hierarchy/types.rs` |
| Entropy mixing engine | Implemented | `beardog-genetics/src/genetics/entropy_hierarchy/sources.rs` |
| Entropy validation | Implemented | `beardog-genetics/src/genetics/entropy_hierarchy/validation.rs` |
| `genetic.mix_entropy` RPC | Defined (IPC map) | `primalSpring/docs/NUCLEUS_IPC_METHOD_MAP.md` |
| `genetic.derive_lineage_key` RPC | Defined | Same |
| HKDF key derivation | Implemented | `beardog-genetics/src/birdsong/key_derivation.rs` |
| `seed_workflow.sh` (family ceremony) | Implemented | `infra/plasmidBin/seed_workflow.sh` |
| Ceremony protocol (full) | **NOT IMPLEMENTED** | This spec |
| Event ceremony (multi-device) | **NOT IMPLEMENTED** | This spec |
| Attenuation scheduler | **NOT IMPLEMENTED** | Needs loamSpine expiry logic |
| Songbird ceremony relay | **NOT IMPLEMENTED** | Needs Songbird ceremony mode |

### Implementation Priority

1. **Personal sovereignty ceremony** (Type 1) — all primitives exist, needs orchestration
2. **Family seed rotation** (Type 2) — `seed_workflow.sh` already does most of this
3. **Event ceremony** (Type 3) — needs Songbird ceremony relay + multi-device BearDog
4. **Collaborative** (Type 4) — lower priority, shares primitives with Type 3

---

*Human entropy is non-fungible. That's the point. Your typing rhythm,
your HSM's attestation, your physical presence at a moment in time —
these cannot be reproduced. Tier 2 sovereign keys inherit this property.
The family seed gives you membership. The ceremony gives you identity.*
