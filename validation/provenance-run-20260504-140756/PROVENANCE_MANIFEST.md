# Provenance Manifest — abg-pipeline-20260504-140807

**Date**: 2026-05-04T14:11:58-04:00
**Composition**: Nest Atomic + ToadStool (9 primals)
**Session**: 019df42d-0fba-7170-a216-2f3b282e3fb9
**Spine**: 019df41b-40c5-7b93-bf35-79dbd95a2cb3

## Provenance Chain

| Layer | Identifier | Purpose |
|-------|-----------|---------|
| DAG Session | `019df42d-0fba-7170-a216-2f3b282e3fb9` | Ephemeral pipeline tracking (24 events) |
| Merkle Root | `292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e` | Content hash of all DAG events |
| LoamSpine Commit | index=55 | Permanent ledger entry |
| SweetGrass Braid | `urn:braid:292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e` | Attribution with ed25519 witness |

## Data Artifacts

ncbi:SRR7760408:R1  6250f200f9ff45e0f3aa52ede78dbe4ad4a68dd1a55b355d7502b02afeaa672a  2223544784B
ncbi:SRR7760408:R2  cd89f43d74d09c64b4c832040f0cc04837c30bf7bb897f083dcd89ee6ece1d7c  2373406079B
ncbi:SRR5534045:R1  096878541679cd066ffa873ac024c7ca3089f4e5df0e6c81dbe05ed64acaeb30  444311690B
ncbi:SRR5534045:R2  bee510af71ac914a5442492574f57b02b6a490eabeecce9d06242c333d9e1d7d  451026452B

## Workload Results

wetspring-r-industry-parity          53/53  0ms      PASS  blake3:cdcbb6da792f8a54
wetspring-fajgenbaum-pathway         8/8    0ms      PASS  blake3:a40d155f06ccdac7
wetspring-diversity-rust-validation  27/27  0ms      PASS  blake3:a90fa663188a0cbd
wetspring-gonzales-cpu-parity        43/43  0ms      PASS  blake3:7a64a239b77340f0
wetspring-algae-16s-rust             34/34  23000ms  PASS  blake3:4e84cf2a1cb2b85c
wetspring-16s-rust-validation        37/37  0ms      PASS  blake3:e46672a7fd06edc0
wetspring-cold-seep-pipeline         8/8    0ms      PASS  blake3:a5faa92507a17449
wetspring-real-ncbi-pipeline         25/25  0ms      PASS  blake3:84906edea315cf08
wetspring-16s-python-baseline        0/0    1000ms   RUN   blake3:db4cb37b55dbe641
wetspring-benchmark-python-baseline  0/0    4000ms   RUN   blake3:01818fcb50d08dcb

## Verification

To verify this pipeline:
1. Confirm BLAKE3 hashes of NCBI FASTQs match the values above
2. Re-run the same workload TOMLs through toadStool
3. Query loamSpine for spine `019df41b-40c5-7b93-bf35-79dbd95a2cb3` to see the full audit trail
4. Query sweetGrass for braid `urn:braid:292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e` to verify the ed25519 witness

## NCBI Data Provenance

| Accession | BioProject | Source | DOI |
|-----------|-----------|--------|-----|
| SRR7760408 | PRJNA488170 | Nannochloropsis outdoor 16S (Wageningen) | 10.1007/s00253-022-11815-3 |
| SRR5534045 | PRJNA382322 | Nannochloropsis extended pilots | 10.1007/s00253-022-11815-3 |
