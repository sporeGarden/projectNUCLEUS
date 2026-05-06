# ABG Shared Workspace — Visibility and Access Model

All work is visible. No hidden files, no private directories. This is open
science infrastructure, not a personal file server.

---

## Principle

The shared workspace is a Google Doc for science: everyone in ABG can see
what everyone else is doing. When work is polished, it moves to showcase/
where external reviewers (PIs, HPC admins) can see it and decide whether to
allocate resources. The point is transparency — every computation is
provenance-tracked, every notebook is visible, every result is reproducible.

---

## Directory Structure

```
/home/irongate/shared/abg/
├── commons/           Scratch notebooks, experiments, exploration
├── projects/          Per-project spaces (created via abg_accounts.sh create-project)
│   └── {name}/
│       ├── notebooks/
│       ├── data/      Symlinks to shared data or NestGate storage
│       └── results/   Workload outputs, provenance manifests
├── data/              Shared datasets (NCBI FASTQs, reference genomes)
├── templates/         Starter notebooks and workload TOMLs (copy, don't modify)
└── showcase/          Polished work for external review
```

All directories use group-sticky permissions (`chmod 2775`). Files created
by any user inherit the group, ensuring visibility across the group.

---

## Access Tiers

| Tier | Linux Group | See commons/ | See projects/ | See showcase/ | Write | Execute | External Share |
|------|-------------|:---:|:---:|:---:|:---:|:---:|:---:|
| **admin** | `abg-admin` | Yes | Yes | Yes | Everywhere | Yes | Manage showcase/ |
| **compute** | `abg-compute` | Yes | Yes | Yes | commons/, projects/ | Yes (ToadStool) | No |
| **observer** | `abg-observer` | Yes | Yes | Yes | No | No | No |
| **reviewer** | `abg-reviewer` | No | No | Yes | No | No | Copy only |

### Tier Details

**admin**: Full access. Can create projects (`abg_accounts.sh create-project`),
manage showcase/, add/remove users. JupyterHub admin. 48 GB / 16 cores.

**compute**: The working tier. Can write to commons/ and assigned projects/.
Submit workloads via ToadStool. Access shared data. Run validation notebooks.
32 GB / 8 cores.

**observer**: Read everything, write nothing. Intended for ABG members who
want to follow the work but aren't actively computing. Can copy notebooks
to run elsewhere. `NUCLEUS_READONLY=1`. 8 GB / 4 cores.

**reviewer**: External-facing. Sees only showcase/ — the polished work that
members have decided is ready for review. Intended for PIs evaluating HPC
allocation requests, collaborators reviewing methodology, or anyone who needs
to see "what you want to run on big systems." `NUCLEUS_READONLY=1`. 4 GB / 2 cores.

---

## Relationship to sporePrint (primals.eco)

sporePrint is the public display case. The shared workspace is the lab.

```
commons/ → member works on notebooks
  ↓ validates via ToadStool + provenance pipeline
projects/ → organized per-project results
  ↓ polishes for review
showcase/ → ready for PI / HPC admin
  ↓ nbconvert → HTML
primals.eco/lab → public, rendered, read-only
```

The `render_notebooks.sh` script in sporePrint converts showcase notebooks
to static HTML pages for the `/lab` section of primals.eco. This pipeline
is manual (operator runs the script), not automatic. The public site shows
only what has been explicitly elevated.

---

## Reviewer Access Model

Reviewers are external to ABG. They don't participate in day-to-day work.
They need to answer: "Is this worth allocating HPC time for?"

**What reviewers see**:
- Rendered notebooks with results and visualizations
- Provenance manifests (Merkle roots, braid URNs, ed25519 witnesses)
- Methodology descriptions and workload TOML specs

**What reviewers cannot do**:
- Execute workloads or access primal APIs
- See work-in-progress in commons/ or projects/
- Modify any files

**Access methods** (in order of implementation):
1. Static export: `nbconvert` → HTML/PDF → email or primals.eco/lab
2. Reviewer tunnel token: Cloudflare Tunnel URL → JupyterHub read-only
3. NestGate public read key: content-addressed storage query (sovereign endgame)

---

## Project Lifecycle

```bash
# Admin creates a project
sudo bash abg_accounts.sh create-project scrna-castleman

# Member copies a template and starts working
cp /home/irongate/shared/abg/templates/abg-wetspring-validation.ipynb \
   /home/irongate/shared/abg/projects/scrna-castleman/notebooks/

# Member submits workloads from the project
toadstool execute /home/irongate/shared/abg/projects/scrna-castleman/workload.toml

# When ready for review, copy to showcase/
cp -r /home/irongate/shared/abg/projects/scrna-castleman/notebooks/final.ipynb \
      /home/irongate/shared/abg/showcase/

# Render for public site
cd /path/to/sporePrint && bash scripts/render_notebooks.sh --all
```

---

## Provenance Integration

All workloads submitted from the shared workspace run through the same
provenance pipeline as validation workloads:

1. ToadStool dispatch (execution tracking)
2. rhizoCrypt DAG session (ephemeral event graph)
3. loamSpine ledger (permanent commit)
4. sweetGrass braid (ed25519-witnessed attribution)

Results in `projects/{name}/results/` include `PROVENANCE_MANIFEST.md`
and `braid.json` — the full audit trail for the computation.

---

## What Belongs in Each Directory

| Directory | Put Here | Don't Put Here |
|-----------|----------|----------------|
| commons/ | Quick experiments, scratch analysis, "is this idea worth pursuing?" | Large datasets, final results |
| projects/ | Organized work with clear scope, workload TOMLs, analysis notebooks | Random scratch (use commons/) |
| data/ | Shared reference data, NCBI downloads, calibration sets | Per-run outputs (use projects/results/) |
| templates/ | Starter notebooks, workload TOML templates | Modified files (copy first) |
| showcase/ | Polished work ready for external eyes, clear methodology | Work in progress |

---

## Implementation

- Directory structure: `/home/irongate/shared/abg/`
- Permissions managed by `deploy/abg_accounts.sh`
- JupyterHub integration via `pre_spawn_hook` in `jupyterhub_config.py`
- Symlinks created at user account setup (shared → user's notebook dir)
- sporePrint rendering via `scripts/render_notebooks.sh`
