# Validation Run Archive

Raw validation run artifacts (logs, headers, ports, stdout files) are archived
here after their evidence has been captured in summary documents. The summary
markdown files in `validation/` are the canonical record; these raw logs are
preserved for forensic reference only.

## Archive Policy

Move a run directory here when:
1. The evidence is fully captured in a summary markdown (e.g., `SECURITY_HANDBACK`, `EXTERNAL_PIPELINE_VALIDATION`)
2. The raw logs are no longer needed for active work
3. The directory name contains a timestamp identifying the run
