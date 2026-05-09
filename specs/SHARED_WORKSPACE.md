# ABG Shared Workspace — Visibility and Access Model

All work is visible to members. Reviewers see only the showcase.
This is open science infrastructure, not a personal file server.

---

## Principle

The shared workspace is the lab: everyone in ABG can see what everyone
else is doing. When work is polished, it moves to showcase/ where external
reviewers (PIs, HPC admins) can see it and decide whether to allocate
resources. The point is transparency — every computation is
provenance-tracked, every notebook is visible, every result is reproducible.

---

## Directory Structure

```
/home/irongate/shared/abg/
├── commons/           Scratch notebooks, experiments, exploration
├── pilot/             Structured experiments (between scratch and formal projects)
│   └── {name}/
│       ├── notebooks/
│       ├── data/
│       └── README.md  Hypothesis, decision criteria, timeline
├── projects/          Formal project spaces (created via abg_accounts.sh create-project)
│   └── {name}/
│       ├── notebooks/
│       ├── data/      Symlinks to shared data or NestGate storage
│       └── results/   Workload outputs, provenance manifests
├── data/              Shared datasets (NCBI FASTQs, reference genomes)
├── templates/         Starter notebooks, workload TOMLs, welcome notebooks
├── showcase/          Polished work for external review + Voila dashboards
├── validation/        Security and system validation results (darkforest JSON reports)
├── envs/              Shared conda environments (bioinfo, r-bioinfo)
└── wheelhouse/        Cached Python packages for offline installation
```

All directories use group-sticky permissions (`chmod 2775`). Files created
by any user inherit the group, ensuring visibility across the group.

---

## Per-User Landing Zone

Each user's `~/notebooks/` directory is their JupyterLab root:

```
~/notebooks/                        (JupyterLab landing zone)
├── Welcome.ipynb                   Tier-appropriate welcome notebook (symlinked)
├── scratch/                        Personal scratch space (compute/admin only, chmod 700)
├── results/                        Personal results
└── shared -> /home/irongate/shared/abg    (compute/admin/observer)
    OR
└── showcase -> .../shared/abg/showcase    (reviewer — showcase only)
```

---

## Access Tiers

| Tier | Linux Group | See commons/ | See pilot/ | See projects/ | See showcase/ | Write | Execute | Personal scratch/ |
|------|-------------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **admin** | `abg-admin` | Yes | Yes | Yes | Yes | Everywhere | Yes | Yes |
| **compute** | `abg-compute` | Yes | Yes | Yes | Yes | commons/, pilot/, projects/ | Yes (ToadStool) | Yes |
| **observer** | `abg-observer` | Yes | Yes | Yes | Yes | No | No | No |
| **reviewer** | `abg-reviewer` | No | No | No | Yes | No | No | No |

### Tier Details

**admin**: Full access. Can create pilots and projects
(`abg_accounts.sh create-pilot`, `create-project`), manage showcase/,
add/remove users. JupyterHub admin. 48 GB / 16 cores. Personal scratch/.

**compute**: The working tier. Can write to commons/, pilot/, and assigned
projects/. Submit workloads via ToadStool. Access shared data. Run
validation notebooks. 32 GB / 8 cores. Personal scratch/.

**observer**: Read everything in the shared tree, write nothing. Intended
for ABG members who want to follow the work but aren't actively computing.
Can copy notebooks to run elsewhere. `NUCLEUS_READONLY=1`. 4 GB / 2 cores.

**reviewer**: External-facing. Sees only showcase/ — polished work that
members have curated for review. Runs Voila dashboards (server-side
execution). No kernels, no terminals, no filesystem writes. Intended for
PIs evaluating HPC allocation requests. `NUCLEUS_READONLY=1`. 8 GB / 4 cores.

---

## Work Lifecycle

```
scratch/   (your private desk — quick tests, throwaway experiments)
    ↓
commons/   (group whiteboard — "is this idea worth pursuing?")
    ↓
pilot/     (structured experiment — hypothesis, decision criteria, timeline)
    ↓
projects/  (formal project — organized data, workloads, provenance-tracked results)
    ↓
showcase/  (polished work — ready for PI/HPC admin review)
    ↓
primals.eco/lab  (public — rendered HTML via sporePrint)
```

### Creating a Pilot

```bash
sudo bash abg_accounts.sh create-pilot scrna-feasibility
```

Creates `pilot/scrna-feasibility/` with `notebooks/`, `data/`, and a
`README.md` template (hypothesis, decision criteria, timeline).

### Promoting Pilot to Project

```bash
sudo bash abg_accounts.sh create-project scrna-castleman
cp -r /home/irongate/shared/abg/pilot/scrna-feasibility/notebooks/* \
      /home/irongate/shared/abg/projects/scrna-castleman/notebooks/
```

### Promoting Project to Showcase

```bash
cp /home/irongate/shared/abg/projects/scrna-castleman/notebooks/final.ipynb \
   /home/irongate/shared/abg/showcase/
chmod 444 /home/irongate/shared/abg/showcase/final.ipynb
```

---

## Relationship to sporePrint (primals.eco)

sporePrint is the public display case. The shared workspace is the lab.

```
commons/ → member works on notebooks
  ↓ validates via ToadStool + provenance pipeline
pilot/ → structured experiment with hypothesis
  ↓ promotes when hypothesis validates
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
- Welcome notebook with orientation and context
- Polished notebooks with results and visualizations
- Validation dashboard (security posture, test results)
- Provenance manifests (Merkle roots, braid URNs, ed25519 witnesses)
- Voila interactive dashboards (server-side execution, no kernel needed)

**What reviewers cannot do**:
- Execute arbitrary code or access primal APIs
- See work-in-progress in commons/, pilot/, or projects/
- Modify any files
- Access terminals

**Access methods** (in order of implementation):
1. Static export: `nbconvert` → HTML/PDF → email or primals.eco/lab
2. Reviewer tunnel token: Cloudflare Tunnel URL → JupyterHub read-only
3. NestGate public read key: content-addressed storage query (sovereign endgame)

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
| scratch/ (personal) | Throwaway tests, personal experiments | Anything others need to see |
| commons/ | Quick shared experiments, "is this idea worth pursuing?" | Large datasets, final results |
| pilot/ | Structured experiments with hypothesis and decision criteria | Random scratch (use commons/) |
| projects/ | Organized work with clear scope, workload TOMLs, analysis | Unstructured experiments (use pilot/) |
| data/ | Shared reference data, NCBI downloads, calibration sets | Per-run outputs (use projects/results/) |
| templates/ | Starter notebooks, workload TOML templates | Modified files (copy first) |
| showcase/ | Polished work ready for external eyes, clear methodology | Work in progress |
| validation/ | darkforest reports, security scan results | Ad-hoc scripts (use commons/) |

---

## Implementation

- Directory structure: `/home/irongate/shared/abg/`
- Permissions managed by `deploy/abg_accounts.sh`
- JupyterHub integration via `pre_spawn_hook` in `jupyterhub_config.py`
- Tier-aware symlinks: reviewer gets `showcase/` only, others get full `shared/`
- Welcome notebooks: `templates/welcome-{tier}.ipynb` symlinked as `~/notebooks/Welcome.ipynb`
- Personal scratch: `~/notebooks/scratch/` (compute/admin only, chmod 700)
- Validation results: `validation/darkforest-latest.json` updated by admin scans
- sporePrint rendering via `scripts/render_notebooks.sh`
