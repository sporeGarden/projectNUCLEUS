# Provenance Manifest — abg-pipeline-20260514-142713

**Date**: 2026-05-14T14:30:33-04:00
**Composition**: Full NUCLEUS (13 primals)
**Session**: 019e27be-261f-7652-8ee2-fe075e7bea1f
**Spine**: 019e27ba-156a-7e33-9a65-d87a6146095a

## Provenance Chain

| Layer | Identifier | Purpose |
|-------|-----------|---------|
| DAG Session | `019e27be-261f-7652-8ee2-fe075e7bea1f` | Ephemeral pipeline tracking (28 events) |
| Merkle Root | `4846a70e781ed1d905ec023f53e2260b369c21a5f9f81cbf02be0b32210ce32e` | Content hash of all DAG events |
| LoamSpine Commit | index=16 | Permanent ledger entry |
| SweetGrass Braid | `urn:braid:4846a70e781ed1d905ec023f53e2260b369c21a5f9f81cbf02be0b32210ce32e` | Attribution with ed25519 witness |

## Data Artifacts

ncbi:SRR7760408:R1  6250f200f9ff45e0f3aa52ede78dbe4ad4a68dd1a55b355d7502b02afeaa672a  2223544784B
ncbi:SRR7760408:R2  cd89f43d74d09c64b4c832040f0cc04837c30bf7bb897f083dcd89ee6ece1d7c  2373406079B
ncbi:SRR5534045:R1  096878541679cd066ffa873ac024c7ca3089f4e5df0e6c81dbe05ed64acaeb30  444311690B
ncbi:SRR5534045:R2  bee510af71ac914a5442492574f57b02b6a490eabeecce9d06242c333d9e1d7d  451026452B

## Workload Results

wetspring-16s-python-baseline            ERROR  -        -     -
wetspring-16s-rust-validation            ERROR  -        -     -
wetspring-algae-16s-rust                 34/34  22000ms  PASS  blake3:93ff7700ca39218a
wetspring-benchmark-python-baseline      ERROR  -        -     -
wetspring-cold-seep-pipeline             8/8    0ms      PASS  blake3:e5a8f4b9dffc8b8b
wetspring-diversity-rust-validation      ERROR  -        -     -
wetspring-exp001-python-baseline         ERROR  -        -     -
wetspring-fajgenbaum-pathway             8/8    0ms      PASS  blake3:7643b7870ecb6cf5
wetspring-gonzales-cpu-parity            43/43  0ms      PASS  blake3:bd41047b1ad316fc
wetspring-ltee-b7-mutation-accumulation  ERROR  -        -     -
wetspring-real-ncbi-pipeline             25/25  0ms      PASS  blake3:267a8cef868f43a6
wetspring-r-industry-parity              53/53  0ms      PASS  blake3:2f7dec1bd07c8ee2

## Verification

To verify this pipeline:
1. Confirm BLAKE3 hashes of NCBI FASTQs match the values above
2. Re-run the same workload TOMLs through toadStool
3. Query loamSpine for spine `019e27ba-156a-7e33-9a65-d87a6146095a` to see the full audit trail
4. Query sweetGrass for braid `urn:braid:4846a70e781ed1d905ec023f53e2260b369c21a5f9f81cbf02be0b32210ce32e` to verify the ed25519 witness

## NCBI Data Provenance

| Accession | BioProject | Source | DOI |
|-----------|-----------|--------|-----|
| SRR7760408 | PRJNA488170 | Nannochloropsis outdoor 16S (Wageningen) | 10.1007/s00253-022-11815-3 |
| SRR5534045 | PRJNA382322 | Nannochloropsis extended pilots | 10.1007/s00253-022-11815-3 |
