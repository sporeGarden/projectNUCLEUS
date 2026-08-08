# pappusCast — DEPRECATED

**Status**: DEPRECATED — vestigial Python observer pipeline
**Filed**: 2026-08-08 Wave 157a role refinement
**Superseded by**: biomeOS composition.deploy + static observer via sporePrint

## What This Was

pappusCast was the tiered auto-propagation daemon for pushing notebook content
from the shared JupyterHub workspace to the public observer surface. It handled:

- Light/Medium/Heavy validation tiers
- Adaptive rate-limiting based on active users
- HTML export for static observer surface
- Snapshot architecture (public/ holds managed copies)

## Why Deprecated

1. Static observer replaced dynamic Voila rendering (2026-05-10)
2. biomeOS `composition.deploy` handles service lifecycle
3. sporePrint owns the public website surface
4. toadStool dispatch handles workload orchestration
5. No active development on this Python code since Wave 136

## Migration Path

| Component | Absorbing System |
|-----------|-----------------|
| Observer HTML export | sporePrint static build pipeline |
| Workspace propagation | biomeOS signal graphs or toadStool dispatch |
| Tier validation logic | primalSpring certification engine |

## Files

- `daemon.py`, `export.py`, `publisher.py`, `tiers.py`, `state.py`, `config.py`
- Entry point: `deploy/pappusCast.py`
- Related: `deploy/observer_server.py` (static HTTP server on :8866)

## Action

Do not invest in this code. When the observer surface is fully absorbed by
sporePrint, move this directory to fossilRecord.
