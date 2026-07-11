<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Tiered Remote Access Architecture — projectNUCLEUS

**Date**: June 15, 2026
**Status**: Architectural Vision (Phase 1 operational, Phases 2-4 evolution)
**Generation**: gen4 — composition and deployment
**Upstream**: `SOVEREIGN_COMPUTE_SHARING.md`, `TIER2_CEREMONY_DESIGN.md`, `TRANSPORT_MATRIX.md`
**Authority**: wateringHole consensus + projectNUCLEUS deployment practice

---

## Purpose

Define how external users access NUCLEUS compute through a tiered sovereignty
model. Each tier grants a different level of observation and action, enforced
first by OS permissions (Phase 1), then by protocol parsing (Phase 2), and
ultimately by bearDog cryptographic key derivation (Phase 3+).

The fundamental insight: because remote connections are parseable (RustDesk
protocol separates view from input; JupyterHub API separates notebook render
from kernel execution; IDE protocols separate read from write), the system
can enforce **view/action separation** at the transport layer — not just at
the application layer.

---

## Four Tiers

### Tier 1: Observer (Static)

**Access pattern**: Pre-rendered HTML. No compute. No interaction. No live state.

**What they see**: Notebook outputs rendered as static HTML by pappusCast.
Published results, figures, tables. Equivalent to reading a paper.

**What they cannot do**: Execute code, modify notebooks, see live state,
interact with running processes, access the gate.

**Sovereignty tool (now)**: pappusCast (pre-renders notebooks as HTML,
served by `observer_server.py` on :8866)

**Primal replacement (future)**: petalTongue static render mode. Notebooks
are rendered to HTML by petalTongue's grammar-of-graphics engine and served
as content-addressed artifacts via nestGate.

**Enforcement**: No authentication required for static content. Content is
public by design (published results). No session, no state, no compute path.

---

### Tier 2: Reviewer (Live Image, Read-Only)

**Access pattern**: Live screen image of running state. Can observe in
real-time. Can approve or reject (project lead function). Cannot execute,
type, or modify.

**What they see**: Live rendered output from a running session — the same
pixels the compute user sees, streamed as video/frames. Dashboard state,
job progress, live notebook output.

**What they cannot do**: Send keystrokes. Execute cells. Modify files.
Start/stop processes. Access raw data. Only receives rendered pixels.

**Sovereignty tool (now)**: RustDesk view-only mode. The RustDesk protocol
separates the video stream (server→client) from the input channel
(client→server). View-only disables the input channel entirely.

**Primal replacement (future)**: songBird-relayed screen stream. songBird
mesh carries the rendered frame stream over BTSP-encrypted channels. The
receiver gets pixels; the sender's raw state is never transmitted.

**Enforcement (now)**: RustDesk server configured with view-only permission
for reviewer keys. OS user `abgreviewer` has no write access, no terminal,
no kernel creation.

**Enforcement (future)**: bearDog reviewer key derivation. The key itself
encodes read-only capability. Attempting to send input with a reviewer key
produces a cryptographic rejection — the transport layer refuses to encrypt
input frames under a reviewer-derived session key.

**Project Lead role**: The project lead holds a reviewer key but also has
an approval channel. They can observe the live state and issue approve/reject
signals (a narrow action path — not arbitrary execution, only structured
decisions). This maps to bearDog's event ceremony model (Type 3): the project
lead is the "authority" who signs off on work.

---

### Tier 3: User / Compute

**Access pattern**: Full bidirectional compute access. Push work to the mesh.
Run notebooks. Execute science. See own results. Equivalent to sitting at
the machine — but scoped to compute, not OS administration.

**What they see**: Their own workspace, notebooks, results. JupyterHub
kernels (Python, R, domain-specific). IDE environment (Cursor-like). File
system scoped to their home + shared commons.

**What they cannot do**: Modify NUCLEUS deployment. Access other users'
private workspaces. Escalate to hardware control. See primal internals or
deployment state. Access bearDog keys or family material.

**Sovereignty tools (now)**:
- JupyterHub (multi-user notebooks) — live at lab.primals.eco
- Cursor/IDE (code editing, agent execution, mesh push)
- SSH tunnel (headless compute, script submission)

**Primal replacement (future)**:
- toadStool dispatches workloads (replaces JupyterHub kernel management)
- squirrel AI orchestrates agentic work (replaces IDE agent layer)
- biomeOS routes capability.call to appropriate hardware
- petalTongue serves the interactive interface (replaces JupyterHub frontend)

**Enforcement (now)**: Unix user per ABG member. cgroups for resource
isolation. Per-user home directories. JupyterHub spawner limits.

**Enforcement (future)**: bearDog user ionic key (derived from project
lead's key via HKDF). User's compute state is encrypted with their ionic
key — neither the hardware operator nor other users can decrypt it. The
NUCLEUS primals operate on encrypted blobs; only the user's key can
decrypt results.

**IDE convergence**: The user accesses a Cursor-like IDE that pushes work
to the mesh. From the user's perspective, they're editing code and running
experiments. From the mesh's perspective, it receives toadStool workload
specifications and dispatches to available hardware. The IDE is a thin
client over the primal composition — squirrel decomposes intent into
atomic signals, biomeOS dispatches them.

---

### Tier 4: Operator (Hardware)

**Access pattern**: Physical hardware administration. Power cycle, disk
replacement, network configuration, OS installation. The "ops" tier —
exclusively physical-space actions that cannot be agentified.

**What they see**: Hardware state (BIOS, disks, network interfaces, power).
OS-level metrics. Physical indicators. Cannot see compute workloads
(encrypted by user keys).

**What they cannot do**: Read user data. Decrypt compute results. Access
notebooks or code. See primal internal state beyond health metrics. In full
deployment, the operator is cryptographically blind to user compute — they
see encrypted blobs they cannot interpret.

**Sovereignty tool (now)**: Direct SSH/console. Physical presence. IPMI/BMC
for remote power. The operator is the human building hardware.

**Primal replacement (future)**: bearDog sovereign key ceremony (Type 1:
Personal Sovereignty). The hardware key is ceremony-derived, held on HSM,
and cannot decrypt software-layer material. The gate responds to hardware
commands only when authenticated by the operator's sovereign key — but
the commands it accepts are limited to hardware operations.

**Enforcement (now)**: Root access restricted to operator. SSH keys managed
per-gate. Physical access required for initial setup.

**Enforcement (future)**: bearDog hardware/software split. Two independent
key hierarchies:
- Hardware key: can power cycle, configure network, replace disks
- Software key: can deploy primals, manage NUCLEUS, update binaries
- Neither can derive the other. Neither can decrypt user compute state.

---

## View/Action Separation Principle

The parseable nature of remote protocols enables enforcement at the transport
layer:

| Protocol | View Channel | Action Channel | Separation Method |
|----------|-------------|----------------|-------------------|
| RustDesk | Video frames (server→client) | Input events (client→server) | Disable input channel |
| JupyterHub | Notebook render (HTML output) | Kernel execution (execute_request) | Block execute messages |
| IDE/LSP | Document state (textDocument/didOpen) | Edit operations (textDocument/didChange) | Read-only mode |
| BTSP (future) | Encrypted read stream | Encrypted write stream | Key derivation scope |

Because we can parse the remote connection, we can separate what flows in
each direction. A reviewer key encrypts only the view channel — the action
channel is cryptographically unreachable. This is not ACL-based filtering
(which can be bypassed); it's structural — the key literally cannot produce
valid ciphertext for the action channel.

---

## bearDog Blindness Model

In full BTSP Phase 4 deployment, four independent key hierarchies exist on
each gate:

```
FAMILY_SEED (root)
  ├── Hardware Sovereign Key (Tier 4)
  │     └── Can: power, disk, network, physical
  │     └── Cannot: decrypt compute, read user state, deploy software
  │
  ├── Software Sovereign Key (Tier 4 elevated / deploy admin)
  │     └── Can: deploy primals, update NUCLEUS, manage services
  │     └── Cannot: access hardware controls, read user state
  │
  ├── Project Lead Key (Tier 3 elevated)
  │     └── Can: approve/reject, view all project state, manage users
  │     └── Derives: per-user ionic keys via HKDF
  │     │
  │     ├── User Ionic Key A (Tier 3)
  │     │     └── Can: compute, run notebooks, see own results
  │     │     └── Cannot: see other users, modify NUCLEUS, hardware
  │     │
  │     └── User Ionic Key B (Tier 3)
  │           └── Same scope, different user, independent encryption
  │
  └── Reviewer Key (Tier 2)
        └── Derived read-only from Project Lead Key
        └── Can: observe rendered output (pixels/frames)
        └── Cannot: execute, modify, access raw state
```

The critical property: **no key can derive a key from a different branch**.
The hardware operator cannot derive a user key. The user cannot derive the
hardware key. The reviewer cannot derive an action key. This is structural
blindness — not policy, not ACLs, not trust-me-bro. The math prevents it.

This converges with `TIER2_CEREMONY_DESIGN.md`:
- Type 1 (Personal Sovereignty) → Hardware/Software operator keys
- Type 2 (Family Seed) → Root derivation material
- Type 3 (Event Ceremony) → Project-scoped user keys
- Type 4 (Collaborative Creation) → Multi-user shared compute contexts

---

## Convergence with Existing Infrastructure

### pappusCast → Tier 1

pappusCast already implements Tier 1: notebooks are pre-rendered as static
HTML by the `pappuscast/` daemon and served by `observer_server.py`. The
observer test suite (`darkforest --suite observer`) validates that observer users
have no compute path.

Evolution: petalTongue's static render mode replaces pappusCast. Rendered
artifacts are content-addressed via nestGate. The observer surface becomes
a primal-native static site — no Python, no Jupyter dependency.

### RustDesk → Tier 2

RustDesk view-only mode implements Tier 2: the reviewer sees live screen
state but cannot send input. The relay on golgiBody-ext (:21115-21117)
carries the stream.

Evolution: songBird mesh carries rendered frames. The stream is BTSP-
encrypted with a reviewer-derived session key that can only decrypt the
view channel. songBird's existing relay architecture (mesh peer forwarding)
carries the frames without modification — it's just bytes to songBird.

### JupyterHub + Cursor/IDE → Tier 3

JupyterHub (lab.primals.eco) implements Tier 3 for notebook users. A
Cursor-like IDE implements Tier 3 for code/agent users. Both push work
to the mesh — one via kernel execution, one via agent signals.

Evolution: toadStool replaces JupyterHub's kernel management. squirrel
replaces the IDE's agent layer. The user interface (petalTongue web or
native client) sends structured workload specs; toadStool dispatches to
hardware; squirrel handles agentic decomposition. The user never directly
touches the OS — they interact with an abstraction that happens to run
on physical hardware they can't see.

### SSH/Console → Tier 4

Direct SSH and physical console access implement Tier 4. The operator is
the human building hardware — "ops" in the ecoPrimals taxonomy.

Evolution: bearDog sovereign key ceremonies produce hardware keys stored
on HSM (SoloKey/FIDO2). The operator authenticates with their hardware
key to access physical controls. The software admin authenticates with
their software key to deploy NUCLEUS. Neither can read user compute state.

---

## Convergence Phases

### Phase 1: Current (Wave 114)

| Tier | Implementation | Status |
|------|---------------|--------|
| 1 Observer | pappusCast static HTML | LIVE |
| 2 Reviewer | RustDesk view-only via golgiBody relay | VALIDATING |
| 3 User | JupyterHub at lab.primals.eco | LIVE |
| 4 Operator | SSH + physical | LIVE |

Enforcement: OS-level (Unix permissions, cgroups, tier_enforcement_test.sh).

### Phase 2: Protocol Parsing (Wave 115-116)

| Tier | Evolution | Depends On |
|------|-----------|------------|
| 2 Reviewer | RustDesk view-only enforced per-user key | RustDesk key config |
| 3 User | IDE access via relay (Cursor-like, push to mesh) | fieldGate intake NUC |
| 3 User | BTSP Phase 2 auth required for all tiers | bearDog ionic verification |
| All | fieldGate routes users to workload gates by tier | cellMembrane routing |

Enforcement: Protocol-level (RustDesk input channel disabled, JupyterHub
message filtering, BTSP identity verification).

### Phase 3: Primal-Native (Wave 117+)

| Tier | Evolution | Replaces |
|------|-----------|----------|
| 1 Observer | petalTongue static render + nestGate content | pappusCast |
| 2 Reviewer | songBird frame stream + BTSP view-key | RustDesk |
| 3 User | toadStool dispatch + squirrel AI | JupyterHub |
| 4 Operator | bearDog hardware sovereign key | SSH trust |

Enforcement: Cryptographic (bearDog key derivation scopes, BTSP channel
encryption, structural blindness between tiers).

### Phase 4: Full Sovereignty (Endgame)

- No JupyterHub, no RustDesk, no SSH — all primal-native
- Each tier is a bearDog key ceremony output
- View/action separation is cryptographic, not network/OS
- Remote IDE = squirrel + petalTongue over BTSP
- Hardware and software operators are mutually blind
- Users are blind to each other and to operators

---

## Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| `TIER2_CEREMONY_DESIGN.md` | Key ceremonies produce tier-specific keys |
| `TRANSPORT_MATRIX.md` | Per-primal transport characteristics inform routing per tier |
| `COMPOSITION_CONTRACT.md` | Composition determines which primals serve which tier |
| `SOVEREIGNTY_VALIDATION_PROTOCOL.md` | Validates that tier boundaries hold |
| `TUNNEL_EVOLUTION.md` | Tunnel patterns evolve into BTSP-native channels |
| `EXECUTION_MODEL.md` | Primals are invisible substrate — users never see them |
| `SCIENCE_DISPATCH_MAP.md` | Maps science workloads to primal compositions |

---

## References

- `deploy/tier_enforcement_test.sh` — OS-level tier validation (4 tiers, 4 test users)
- `darkforest --suite observer` — Observer tier: no compute, static only (Rust, 86 PASS)
- `deploy/tier_test_all.sh` — All tiers: observer + reviewer + compute + hub + pappusCast
- Legacy Python tier tests archived to `validation/archive/legacy/`
- `deploy/jupyterhub_btsp_auth.py` — BTSP auth integration (Phase 2 bridge)
- `deploy/pappuscast/` — Static observer rendering daemon
- `deploy/observer_server.py` — Static HTML server for observer tier
- `infra/wateringHole/compute-sharing/SOVEREIGN_COMPUTE_SHARING.md` — Full evolution phases
- `infra/wateringHole/MEMBRANE_CHANNEL_ARCHITECTURE.md` — Channel 2b (RustDesk relay)
- `infra/wateringHole/handoffs/NETWORK_SEGMENTATION_POLICY_WAVE114_JUN15_2026.md` — Routing zones
