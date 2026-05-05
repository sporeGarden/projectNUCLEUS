# Composition Validation Log — wetSpring Through Node Atomic

**Date**: 2026-05-04
**Gate**: ironGate (Intel i9-14900K, 96 GB DDR5, RTX 5070)
**Composition**: Node Atomic (beardog, songbird, toadstool, barracuda, coralreef)

## Objective

Validate wetSpring bioinformatics science running through toadStool dispatch
on a live Node Atomic composition. This closes the third arrow in the
validation pattern: **Published results → Python/standard tools → Rust →
primal compositions**.

## Node Atomic Status

All 5 primals running at time of execution:

| Primal     | PID    | Port  | Status |
|------------|--------|-------|--------|
| beardog    | 608859 | 9100  | OK     |
| songbird   | 608938 | 9200  | OK     |
| toadstool  | 609984 | 9400  | OK     |
| barracuda  | 609106 | 9500  | OK     |
| coralreef  | 610064 | 9730  | OK     |

Family ID: `9b32f3a8` (irongate-sovereign)

## Rust Validation Workloads — Through toadStool Dispatch

### 16S Pipeline Validation

- **Workload**: `wetspring-16s-rust-validation.toml`
- **Execution ID**: `93115af5-347f-410a-bb57-da5bd97c991d`
- **Status**: SUCCESS
- **Duration**: 0.002s
- **Checks**: 37/37 PASS
- **Domains covered**:
  - DADA2 denoising (6 checks): ASV counts, read conservation, FASTA headers
  - Chimera detection / UCHIME-style (5 checks): input sequences, parent preservation, pass-through
  - Taxonomy / Naive Bayes (6 checks): trained taxa, kingdom confidence, phylum assignment
  - UniFrac distance (10 checks): tree structure, branch length, symmetry, weighted/unweighted
  - End-to-end 16S pipeline (10 checks): derep, DADA2, chimera, diversity indices, Shannon analytical

### Diversity Metrics Validation

- **Workload**: `wetspring-diversity-rust-validation.toml`
- **Execution ID**: `426f894b-cb86-487d-b58c-da93d141a748`
- **Status**: SUCCESS
- **Duration**: 0.001s
- **Checks**: 27/27 PASS
- **Domains covered**:
  - Analytical unit tests (5 checks): Shannon, Simpson, Bray-Curtis symmetry
  - Simulated marine microbiome (4 checks): observed features, diversity indices, Chao1
  - Bray-Curtis distance matrix (2 checks): ordered distances, self-distance
  - K-mer counting (3 checks): 4-mer and 8-mer exact counts
  - Evenness + rarefaction (4 checks): Pielou, monotonic rarefaction
  - Exp002 QIIME2/Galaxy baseline (9 checks): low/high diversity parity, PCoA eigenvalues

### Gonzales CPU Parity (Immunological / Paper 12)

- **Workload**: `wetspring-gonzales-cpu-parity.toml`
- **Execution ID**: `3990f1ae-4e42-4dea-8653-c1087d7758a3`
- **Status**: SUCCESS
- **Duration**: 0.048s
- **Checks**: 43/43 PASS
- **Domains covered**:
  - D01 Hill equation (9 checks): barracuda vs manual, edge cases (0, 1M concentration)
  - D02 Mean & R² (2 checks): exact parity with manual computation
  - D03 Exponential regression (9 checks): parameter recovery, R², prediction parity, PK dose-duration
  - D04 Diversity metrics (4 checks): Shannon, Simpson, Pielou, Chao1 vs textbook
  - D05 Anderson spectral (10 checks): deterministic seed parity (2D/3D), cross-seed divergence
  - D06 IC50→barrier mapping (9 checks): monotonicity, round-trip precision < 1e-10

### Algae 16S Validation (Exp012)

- **Workload**: `wetspring-algae-16s-rust.toml`
- **Execution ID**: `7b98e8ac-0c0e-444a-9519-e34ebcd8aaed`
- **Status**: SUCCESS
- **Duration**: 0.002s
- **Checks**: 26/26 PASS
- **Domains covered**:
  - Synthetic algae-pond pipeline (17 checks): full 16S on synthetic reads, taxonomy, UniFrac
  - Humphrey 2023 reference points (4 checks): OTU count, diversity indices, core genera
  - Python control parity / PRJNA488170 (5 checks): Shannon/Simpson agreement, QC retention

## Python Baseline Workloads — Through toadStool Dispatch

### Benchmark Python Baseline

- **Workload**: `wetspring-benchmark-python-baseline.toml`
- **Execution ID**: `a32bbcc1-3539-4030-994f-c716baf308e8`
- **Status**: SUCCESS
- **Duration**: 4.147s
- **Output**: JSON results saved to `benchmarks/results/python_baseline_2026-05-04T15-00-52.json`
- **Coverage**: Shannon entropy, Simpson diversity, variance, dot product, Bray-Curtis, cosine, PCoA
- **Note**: This establishes the Python timing baseline for Rust speedup comparison

### 16S Python Baseline

- **Workload**: `wetspring-16s-python-baseline.toml`
- **Execution ID**: `511f6c23-d797-4363-a83e-20e5691c3063`
- **Status**: PARTIAL — dispatched and ran, 0 datasets processed
- **Duration**: 0.195s
- **Root cause**: FASTQ data files not downloaded locally (PRJNA488170, PRJNA382322)
- **Finding**: Dispatch pipeline fully operational; data dependency is external

### Exp001 Python Baseline

- **Workload**: `wetspring-exp001-python-baseline.toml`
- **Execution ID**: `cab3bfe1-2379-4425-94b1-8d4ccf42f647`
- **Status**: PARTIAL — dispatched, data dir missing
- **Duration**: 0.020s
- **Root cause**: `/tmp/MiSeq_SOP` not present (sandbox path interference + missing data)
- **Finding**: Sandbox overrides `working_dir` to `/tmp`, breaking relative data paths

## Summary

| Workload                   | Type   | Checks | Status                  | Duration |
|----------------------------|--------|--------|-------------------------|----------|
| 16S Pipeline (Rust)        | native | 37/37  | PASS                    | 0.002s   |
| Diversity Metrics (Rust)   | native | 27/27  | PASS                    | 0.001s   |
| Gonzales CPU Parity (Rust) | native | 43/43  | PASS                    | 0.048s   |
| Algae 16S (Rust)           | native | 26/26  | PASS                    | 0.002s   |
| Benchmark (Python)         | native | —      | PASS (full benchmark)   | 4.147s   |
| 16S (Python)               | native | —      | PARTIAL (no data files) | 0.195s   |
| Exp001 (Python)            | native | —      | PARTIAL (no data files) | 0.020s   |

**133 total Rust checks passed, 0 failed.**
**Composition dispatch validated: wetSpring science runs correctly inside Node Atomic via toadStool.**

## Build Notes

- **CPU binaries**: 4 built successfully (validate_16s_pipeline, validate_diversity, validate_gonzales_cpu_parity, validate_algae_16s)
- **GPU binaries**: Build failed — `submit_and_poll` API removed from `WgpuDevice` (replaced by `submit_and_map`). Affects `validate_anderson_2d_qs`, `validate_qs_disorder_real`.
- **Python runtime**: pyo3 FFI removed per ecoBin v3.0. Python dispatched as `type = "native"` with explicit interpreter path (conda bioinfo environment).

## Dispatch Observations

1. toadStool correctly loaded all 7 TOML workload specs without error
2. Runtime selection (Native) automatic and correct for all workloads
3. Execution IDs assigned with UUIDs — provides audit trail
4. toadStool registered both Native and WASM runtimes at startup
5. Python runtime noted as "delegate to AI/routing service via IPC" (not directly registered)
6. Sandbox isolation applies `working_dir` override to `/tmp` — needs `[security] isolation_level = "None"` for local dev
7. Absolute paths required in TOML args due to sandbox; relative paths resolve against `/tmp`
