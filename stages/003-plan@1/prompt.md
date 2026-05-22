Goal: Improve the Fabro software factory inside modernagencysales/fabro-maestro using docs/factory/maestro-fabro-perfect-software-factory.md. Do not work on Maestro V2.
Run ID: 01KS6HRJR3E0A5FZDTJP44DGM7
Pipeline progress: 1 of 15 stages completed

## Stage: preflight
- Status: succeeded
- Handler: command
- Script: `set -euo pipefail; remote=$(git remote get-url origin); origin_safe=$(printf '%s' "$remote" | sed -E 's#(https://x-access-token:)[^@]+@#\1REDACTED@#; s#(https://)[^/@:]+:[^@]+@#\1REDACTED@#'); echo "origin=${origin_safe}"; case "$remote" in *modernagencysales/fabro-maestro*|*fabro-sh/fabro*) ;; *) echo "FAIL: expected fabro-maestro/fabro origin"; exit 1;; esac; test -f docs/factory/maestro-fabro-perfect-software-factory.md; .fabro/scripts/factory-validate.sh --quick`
- Output:
  ```
  origin=https://REDACTED@github.com/modernagencysales/fabro-maestro
  repo=https://REDACTED@github.com/modernagencysales/fabro-maestro
  mode=--quick
  === quick preflight checks ===
  validation_passed_quick
  ```


# Factory Self-Improvement Planner

You are improving the Fabro factory inside `modernagencysales/fabro-maestro`.

Hard scope:
- Work on this repo only.
- Do not work on Maestro V2.
- Treat `docs/factory/maestro-fabro-perfect-software-factory.md` as product/design source.
- Improve the factory incrementally: workflows, run configs, prompts, scripts, eval contracts, docs, tests.
- Prefer small, verifiable patches.

Read:
- `AGENTS.md`
- `.fabro/project.toml`
- `.fabro/workflows/**/workflow.fabro`
- `docs/factory/maestro-fabro-perfect-software-factory.md`

Write `.factory/self/plan.md` with:
- current factory gaps
- first three concrete patches
- validation commands
- risks and guardrails

Then summarize plan.
