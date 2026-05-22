I investigated the failure and found no persistent factory code issue to fix.

### What I checked
- Read `.factory/self/validation.log` (it stopped at):
  - `mode=full`
  - `=== build repo-local fabro cli ===`
- Checked current git diff/status:
  - clean (no pending changes)

### Re-runs performed
- ✅ `.fabro/scripts/factory-validate.sh --quick` (as requested) — **passed**
- ✅ Reran failed full command from the log:
  - `cargo build -q -p fabro-cli` — **passed**
- ✅ Also ran full validation end-to-end to confirm:
  - `.fabro/scripts/factory-validate.sh` — **passed**

No code changes were necessary; the prior failure appears to have been transient/interrupted rather than a script/workflow defect.