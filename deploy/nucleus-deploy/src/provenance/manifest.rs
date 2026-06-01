//! Phase 9: Write the provenance manifest document.

use chrono::Local;
use std::path::Path;
use tokio::fs;

use super::{log, ProvenanceError, ProvenanceState};

pub(super) async fn phase_write_manifest(
    results_dir: &Path,
    session_name: &str,
    state: &ProvenanceState,
    merkle_hex: &str,
    commit_index: &str,
    braid_id: &str,
    braid_witness: &str,
) -> Result<(), ProvenanceError> {
    log("");
    log("── Phase 9: Write Provenance Manifest ──");

    let artifact_table: String = state
        .artifact_hashes
        .iter()
        .map(|a| {
            format!(
                "| `{}` | `{}` | {}B |",
                a.key,
                &a.hash[..a.hash.len().min(32)],
                a.size
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let workload_table: String = state
        .workload_results
        .iter()
        .map(|w| {
            format!(
                "| {} | {} | {}ms | {} | `{}` |",
                w.name, w.checks, w.duration_ms, w.status, w.hash_prefix
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let manifest = format!(
        "# Provenance Manifest — {session_name}\n\n\
        **Date**: {date}\n\
        **Composition**: Full NUCLEUS (13 primals)\n\
        **Session**: {session_id}\n\
        **Spine**: {spine_id}\n\n\
        ## Provenance Chain\n\n\
        | Layer | Identifier | Purpose |\n\
        |-------|-----------|----------|\n\
        | DAG Session | `{session_id}` | Ephemeral pipeline tracking ({event_count} events) |\n\
        | Merkle Root | `{merkle_hex}` | Content hash of all DAG events |\n\
        | LoamSpine Commit | index={commit_index} | Permanent ledger entry |\n\
        | SweetGrass Braid | `{braid_id}` | Attribution with ed25519 witness |\n\n\
        ## Data Artifacts\n\n\
        | Key | BLAKE3 | Size |\n\
        |-----|--------|------|\n\
        {artifact_table}\n\n\
        ## Workload Results\n\n\
        | Workload | Checks | Duration | Status | Hash |\n\
        |----------|--------|----------|--------|------|\n\
        {workload_table}\n\n\
        ## Verification\n\n\
        1. Confirm BLAKE3 hashes of NCBI FASTQs match the values above\n\
        2. Re-run the same workload TOMLs through toadStool\n\
        3. Query loamSpine for spine `{spine_id}` to see the full audit trail\n\
        4. Query sweetGrass for braid `{braid_id}` to verify the ed25519 witness\n",
        date = Local::now().to_rfc3339(),
        session_id = state.session_id,
        spine_id = state.spine_id,
        event_count = state.event_idx,
    );

    fs::write(results_dir.join("PROVENANCE_MANIFEST.md"), &manifest).await?;

    log(&format!(
        "  [OK] Manifest: {}/PROVENANCE_MANIFEST.md",
        results_dir.display()
    ));
    let _ = braid_witness;

    Ok(())
}
