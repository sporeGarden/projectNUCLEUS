# Science Dispatch Map — Spring Projects to Gate Hardware

**Date**: May 15, 2026
**Status**: Planning — 10G backbone (cables pending) is the primary unlocker
**Relates to**: `gates/*.toml`, `graphs/node_atomic.toml`,
`gen4/architecture/SOVEREIGN_HPC_EVOLUTION.md`

---

## The Dispatch Model

The sovereign compute trio operates identically on every gate:
- **barraCuda** (WHAT): defines the mathematics — 826+ WGSL shaders
- **coralReef** (HOW): compiles WGSL to target GPU binary (sm_70, gfx906, SM120...)
- **toadStool** (WHERE): dispatches workload to the best available gate

This document maps **which science** lands on **which hardware** and why.
toadStool uses this mapping (via `[science]` sections in gate TOMLs) to
route workloads to gates where they'll execute most efficiently.

---

## Gate Capabilities Summary

| Gate | CPU | RAM | GPU (work) | Special | 10G |
|------|-----|-----|------------|---------|-----|
| **biomeGate** | TR 3970X (32c/64t) | 256 GB DDR4 | Titan V + K80 (HBM2 fleet floats) | HBM2 bench, coralReef dev | No |
| **strandGate** | Dual EPYC 7452 (64c/128t) | 256 GB ECC | RTX 3090 + RX 6950 XT | Akida NPU, dual-vendor | No |
| **northGate** | 9950X3D (16c/32t) | 96 GB DDR5 | RTX 5090 (32 GB) | Strongest single GPU | Yes |
| **ironGate** | i9-14900K (24c/32t) | 96 GB DDR5 | RTX 5070 (12 GB) | ABG gate, composition validation | No |
| **southGate** | 5800X3D (8c/16t) | 128 GB DDR4 | RTX 4060 + float 3090s | Gaming + heavy compute | Yes |
| **eastGate** | i9-12900 (16c/24t) | 32 GB DDR5 | RTX 4070 + Akida NPU | Neuromorphic primary | Yes |
| **westGate** | i7-4771 (4c/8t) | 32 GB DDR3 | RTX 2070S | 76 TB ZFS cold storage | Yes |
| **flockGate** | i9-13900K (24c/32t) | 64 GB DDR5 | RTX 3070 Ti | Remote covalent (WAN) | No |

---

## Spring-to-Gate Dispatch Matrix

### hotSpring — Plasma Physics, Lattice QCD, WDM

| Workload | Primary Gate | Fallback | Requirements | Blocker |
|----------|-------------|----------|--------------|---------|
| Lattice QCD production (32^4 β sweeps) | biomeGate (Titan V) | northGate (SM120 DF64) | HBM2, FP64 >7 TFLOPS, >12 GB | 10G for multi-gate parameter sweeps |
| WDM Yukawa MD (N=10k, Green-Kubo) | biomeGate | northGate | HBM2 bandwidth, FP64 | coralReef DF64 safe-path on SM120 |
| Kokkos parity benchmarks (Exp 053) | biomeGate (Titan V) | — | Same card as reference run | None — active now |
| Deconfinement (β_c search, 32^4) | northGate (RTX 5090) | biomeGate | Large VRAM (32 GB), long runtime | 13.6h per run, thermal management |
| Nuclear EOS (195 checks) | any node_atomic | — | barraCuda shaders, minimal GPU | None |
| Sarkas Yukawa MD validation | any node_atomic | — | 8 GB RAM, 90% CPU | None |

**Why biomeGate is primary**: HBM2 bandwidth (653-1024 GB/s) is critical for
lattice QCD where global memory access dominates. Consumer GDDR7 on northGate
(1792 GB/s on 5090) actually exceeds this, but the FP64 ratio is 1:64 on
consumer vs full-rate on Titan V.

### wetSpring — Bioinformatics, Genomics, Whole-Cell Modeling

| Workload | Primary Gate | Fallback | Requirements | Blocker |
|----------|-------------|----------|--------------|---------|
| Whole-cell modeling (ABG) | strandGate | — | 256 GB ECC, 64 cores | ABG ionic access path complete |
| scRNA-seq (alignment, quantification) | strandGate | southGate (128 GB) | >128 GB RAM, many cores | None |
| Metagenomics (Kraken2, MetaPhlAn) | strandGate | — | Large RAM (DB in memory), CPU | Kraken2 DB on NVMe |
| NCBI FASTQ storage/retrieval | westGate (ZFS) | — | 76 TB cold, 10G replication | 10G cables for bulk transfer |
| Genome graph construction | strandGate | — | 256 GB ECC, CPU-bound | None |

**Why strandGate is primary**: 256 GB ECC RAM holds entire genome databases in
memory. 64 cores parallelize alignment/assembly. ECC ensures bit-exact results
for publishable science. Dual-vendor GPU validates barraCuda parity.

### groundSpring — LTEE, GEMM, Uncertainty Propagation

| Workload | Primary Gate | Fallback | Requirements | Blocker |
|----------|-------------|----------|--------------|---------|
| LTEE reproduction (7 lithoSpore modules) | strandGate + ironGate | — | CPU (breseq), provenance | lithoSpore data pipeline |
| GEMM validation (GemmF64 Tikhonov) | biomeGate (Titan V) | northGate | FP64 GPU, ECC preferred | None |
| GPU bench (gs-bench-gpu) | any node_atomic | — | GPU + barraCuda feature | None |
| Anderson spectral (Thread 7) | strandGate | biomeGate | CPU + GPU, 64-core ideal | None |
| Uncertainty propagation | strandGate | ironGate | Large RAM, many cores | None |

**Why split across gates**: LTEE is CPU-bound bioinformatics (strandGate) but
needs provenance pipeline (ironGate has full NUCLEUS). GEMM is GPU-bound FP64
(biomeGate HBM2).

### neuralSpring — ML Inference, LSTM, Surrogate Learning

| Workload | Primary Gate | Fallback | Requirements | Blocker |
|----------|-------------|----------|--------------|---------|
| LSTM inference (surrogate models) | northGate (RTX 5090) | biomeGate | Large VRAM, tensor cores | None |
| Attention mechanisms | northGate | flockGate | VRAM > 8 GB | None |
| Neuromorphic inference (Akida) | eastGate | biomeGate (Akida) | AKD1000 NPU | toadStool NPU dispatch |
| Training (small models) | northGate or strandGate | — | VRAM + RAM | Model size dependent |

**Why northGate**: RTX 5090's 32 GB GDDR7 and tensor cores are optimized for
ML workloads. 3D V-Cache on 9950X3D helps data preparation.

### airSpring — Hydrology, Agriculture, Environmental

| Workload | Primary Gate | Fallback | Requirements | Blocker |
|----------|-------------|----------|--------------|---------|
| ET0 reference evapotranspiration | ironGate | any | Lightweight CPU | None |
| Soil water balance | ironGate | any | Lightweight CPU | None |
| Environmental atlas | ironGate | strandGate | Moderate RAM + CPU | None |
| QS validation (Thread 4) | any node_atomic | — | barraCuda shaders | None |

**Why ironGate**: airSpring workloads are lightweight — they don't need HBM2 or
256 GB RAM. ironGate is the ABG-facing gate with full composition validation,
making it the natural home for airSpring's applied-science outputs.

### ludoSpring — Game Math, Creative Compute

| Workload | Primary Gate | Fallback | Requirements | Blocker |
|----------|-------------|----------|--------------|---------|
| Perlin noise generation | northGate | flockGate | GPU (any) | None |
| Wave Function Collapse | northGate | flockGate | GPU + VRAM | None |
| Game shader profiling | northGate (RTX 5090) | — | Largest VRAM | None |

**Why northGate/flockGate**: Gaming-adjacent workloads naturally sit on gaming
machines. Owner can run ludoSpring shaders as "productive idle" between sessions.

---

## Cross-Gate Dispatch Patterns (10G Backbone)

Once 10G cables connect northGate, southGate, eastGate, westGate:

```
┌──────────────────────────────────────────────────────────────────┐
│  PIPELINE: Whole-cell modeling (wetSpring + ABG)                   │
│                                                                    │
│  1. FASTQ download → westGate (76 TB ZFS, 10G write)             │
│  2. Alignment/assembly → strandGate (64c EPYC, 256 GB ECC)       │
│  3. GPU acceleration → northGate (RTX 5090, 32 GB)               │
│  4. Results cache → westGate (ZFS snapshots)                      │
│  5. Provenance braid → ironGate (sweetGrass + loamSpine)          │
│  6. Public rendering → VPS outer membrane (sporePrint)             │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  PIPELINE: Lattice QCD parameter sweep (hotSpring)                │
│                                                                    │
│  1. Configuration generation → ironGate (composition validation)  │
│  2. β=5.5 → biomeGate Titan V (HBM2 FP64)                       │
│  3. β=5.7 → northGate RTX 5090 (SM120, DF64 path)               │
│  4. β=5.9 → southGate float 3090 (when slotted)                  │
│  5. Observables merge → strandGate (CPU, 256 GB for analysis)     │
│  6. Provenance → ironGate (braid)                                 │
│  7. Publication → VPS (sporePrint)                                 │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  PIPELINE: LTEE reproduction (groundSpring + lithoSpore)          │
│                                                                    │
│  1. Raw data retrieval → westGate (FASTQ from ZFS)                │
│  2. breseq mutation calling → strandGate (CPU-parallel)           │
│  3. Allele frequency analysis → strandGate                        │
│  4. Statistical validation → ironGate (provenance-wrapped)        │
│  5. lithoSpore module publish → ironGate (guideStone artifact)    │
│  6. Foundation thread update → VPS (sporePrint /lab/)              │
└──────────────────────────────────────────────────────────────────┘
```

---

## Blockers and Evolution Path

| Blocker | Impact | Resolution | Timeline |
|---------|--------|------------|----------|
| 10G cables (~$50) | No wire-speed cross-gate pipelines | Purchase Cat6a/DAC | Immediate |
| ABG ionic access path | ABG can't submit workloads | BTSP ionic tokens + tunnel complete | Phase 2b |
| coralReef DF64 on SM120 | northGate limited to native FP64 (1:64) | DF64 safe-path fix (Exp 053 gap) | hotSpring v0.6.32+ |
| toadStool NPU dispatch | Akida workloads manual | toadStool `compute.dispatch` NPU capability | groundSpring V114+ |
| lithoSpore data pipeline | LTEE modules can't auto-run | lithoSpore → foundation integration | groundSpring B4+ |
| Kraken2 DB staging | Metagenomics needs DB in RAM | Pre-stage on strandGate NVMe | Manual (one-time) |
| Multi-gate dark forest | Can't validate cross-gate security | darkforest remote mode (FUZZ_EVOLUTION.md) | Next sprint |

---

## toadStool Dispatch Logic (Future)

When toadStool receives a `compute.dispatch.submit` request:

1. Parse workload TOML: `domain`, `requirements` (RAM, GPU, FP64, HBM2, CPU cores)
2. Query all covalent gates: `compute.capabilities` → available resources
3. Match requirements to gate `[science].capabilities`
4. Check `owner_priority` / `available_compute` — respect owner load
5. Select optimal gate (lowest latency + best hardware match)
6. Dispatch via Songbird (LAN BirdSong or TCP fallback for WAN gates)
7. Stream results back, record provenance (sweetGrass braid)

For multi-gate pipelines: toadStool coordinates a DAG where each stage
routes to the optimal gate. The 10G backbone makes inter-stage data
transfer negligible for datasets <1 GB.

---

*6 springs, 11 gates, 1 dispatch model. The science routes to where
the hardware is. The 10G backbone makes the cluster one machine.*
