# NUCLEUS — A Primer

A short introduction to what NUCLEUS is, why it exists, and what it means
for collaborators. Written for bioinformaticians, computational scientists,
and anyone who might use or contribute compute to the system.

---

## The One-Sentence Version

NUCLEUS is a way to turn any Linux machine into a sovereign compute node
that can run bioinformatics pipelines, host websites, and share resources
with trusted peers — using only open-source Rust binaries and no cloud
dependencies.

## The Problem

Scientific computing today depends on a stack of external services:
cloud providers for compute, GitHub for distribution, Cloudflare for
hosting, Tailscale for secure access, SLURM for job scheduling. Each
dependency is a point of failure, a cost center, and a trust assumption.

Most researchers have capable hardware sitting idle. A gaming PC with
an RTX 3090 and 128 GB RAM is a serious compute node. A room with
five such machines connected by ethernet is a small HPC cluster. But
making that hardware accessible, secure, and useful for science requires
solving networking, identity, workload dispatch, and storage — problems
that usually push people toward cloud services.

## What NUCLEUS Does

NUCLEUS composes small, purpose-built Rust programs (called **primals**)
into a system that handles all of those problems locally:

| What You Need | Primal | What It Does |
|---------------|--------|--------------|
| Identity and encryption | **BearDog** | Cryptographic identity, signing, BTSP authentication |
| Networking | **Songbird** | Discovery, mesh networking, NAT traversal |
| Job dispatch | **ToadStool** | Accepts workload specs (TOML), dispatches to hardware |
| GPU compute | **barraCuda** | Executes compute shaders across GPU vendors |
| Shader compilation | **coralReef** | Compiles WGSL programs for the target GPU/CPU |
| Storage | **NestGate** | Content-addressed storage with encryption at rest |
| Provenance | **loamSpine** | Audit trails — who ran what, when, on which machine |

These primals are statically-linked Rust binaries. No Python runtime, no
Docker, no JVM. Each is under 5 MB. They communicate over Unix domain
sockets (fast, local) and TCP (cross-machine).

## Atomics: Composable Building Blocks

You don't deploy all primals everywhere. NUCLEUS defines three **atomics**
— minimal compositions matched to what a machine can do:

**Tower** (the trust layer): BearDog + Songbird. Every machine runs Tower.
It handles identity and networking. A Raspberry Pi or NUC can run Tower
as a tunnel endpoint.

**Node** (the compute layer): Tower + ToadStool + barraCuda + coralReef.
Any machine with a GPU or substantial CPU runs Node. This is where
science workloads execute.

**Nest** (the storage layer): Tower + NestGate + provenance tools. Machines
with large storage run Nest. Results are content-addressed and auditable.

A full NUCLEUS is all three atomics plus AI coordination (Squirrel) and
orchestration (biomeOS).

## How Workloads Run

A workload is a TOML file that describes what to execute:

```toml
[metadata]
name = "16s-pipeline-validation"
description = "Validate Rust 16S pipeline against Python baseline"

[execution]
type = "native"
command = "/path/to/validate_16s_pipeline"
working_dir = "/path/to/wetSpring"
```

You submit it to ToadStool: `toadstool execute workload.toml`

ToadStool selects the right runtime (native binary, Python script, GPU
shader, WASM module), executes it, and returns results with an execution
ID for audit.

This has been validated end-to-end with real bioinformatics: 133 checks
across 16S pipeline, diversity metrics, immunological modeling, and algae
community analysis — all passing through ToadStool dispatch on a single
Node Atomic.

## Bonding: How Machines Trust Each Other

When multiple machines participate, NUCLEUS uses a chemistry-inspired
trust model:

**Covalent**: Machines share a family seed. Full trust. This is how your
own local cluster works — all machines are yours, connected by ethernet.

**Ionic**: Scoped, metered access. A friend lends you GPU time. They get
a capability token, not your family seed. You control what they can run.

**Metallic**: Institutional scale. A university HPC cluster joins as a
pool of interchangeable compute nodes.

## What's Running Today

On a validated Phase 1 gate (i9 + 96 GB DDR5 + RTX 5070):

- Node Atomic: 5 primals running (BearDog, Songbird, ToadStool,
  barraCuda, coralReef)
- wetSpring science: 133/133 Rust validation checks passing through
  composition dispatch
- Python baselines: benchmark suite completing in 4.15s through ToadStool
- JupyterHub: notebook access with bioinformatics kernels (Python + R)

Phase 2 (multi-gate, friend compute sharing) is next.

## For ABG Collaborators

If you're in the Accelerated Bioinformatics Group, here's what this
means concretely:

1. **Submit workloads**: Write a TOML spec for your pipeline, submit it
   to the Node Atomic. ToadStool dispatches it to available hardware.

2. **Use existing tools**: Python scripts run through ToadStool as
   native subprocesses. Your existing QIIME2, Scanpy, or custom
   pipelines work as-is.

3. **Validate in public**: Every result gets an execution ID. The Rust
   implementations are validated against published results and Python
   baselines. The validation pattern is:
   Published data → Python/standard tools → Rust → NUCLEUS composition.

4. **Your work helps evolve the system**: Every workload you run
   exercises the dispatch pipeline. Gaps we find become improvement
   targets. The gap reports are public (AGPL).

## The Validation Pattern

This is the core methodology:

```
Published results (papers, databases)
        ↓
Python / established tools (QIIME2, SciPy, R)
        ↓
Rust implementation (wetSpring, barracuda)
        ↓
NUCLEUS composition dispatch (toadStool execute)
        ↓
Parity check + gap report
```

Each arrow is independently verifiable. The Rust matches the Python.
The composition matches standalone Rust. Gaps are documented in
wateringHole for other teams to address.

## Licensing

Everything is published under the scyBorg triple license:

- **AGPL-3.0-or-later**: All code (primals, springs, tools)
- **ORC** (Open RPG Creative): System mechanics and interaction patterns
- **CC-BY-SA 4.0**: Documentation, scientific content, creative works

These three licenses apply orthogonally across all ecoPrimals work.
The AGPL ensures the infrastructure stays open. The ORC covers the
compositional mechanics. CC-BY-SA covers everything you're reading.

## Learn More

- [primals.eco](https://primals.eco) — project website
- [primals.eco/science](https://primals.eco/science) — validated experiments
- [primals.eco/methodology](https://primals.eco/methodology) — the constrained evolution thesis
