All three patches applied and validated. Here is the full plan summary:

---

## Plan Summary

### Current Factory Gaps (8 identified)

| # | Gap | Severity |
|---|-----|----------|
| 1 | Missing `spec-eval.schema.json` and `release-readiness.schema.json` | HIGH |
| 2 | `factory-validate.sh --quick` doesn't check eval schemas exist/are valid JSON | HIGH |
| 3 | Planner prompt has no explicit output contract (goal gate fires on any output) | MEDIUM |
| 4 | No `factory-retro.schema.json` or `maestro-retro.fabro` workflow | MEDIUM |
| 5 | `workflow.toml` has hardcoded local `/Users/ajmal/...` path | LOW |
| 6 | Scope guard doesn't verify reviewer output files were actually written | MEDIUM |
| 7 | Missing migration/performance/observability reviewer prompts (3 of 6) | LOW |
| 8 | No `detect-risk-surfaces.sh` script (used by risk classifier + scope guard) | LOW |

---

### Three Patches Implemented

**Patch 1 — Added two missing eval schemas**  
Created `.fabro/evals/schemas/spec-eval.schema.json` and `.fabro/evals/schemas/release-readiness.schema.json` from the design spec's §12.2 and §12.8 contracts. These define machine-checkable output contracts for the spec evaluator (8 scoring dimensions, overall_score ≥ 4.0 to pass) and the release readiness stage (blockers, rollback plan, manual QA steps). Factory now has 6 complete eval schemas.

**Patch 2 — Wired schema presence+validity into `factory-validate.sh --quick`**  
Added an "eval schema checks" block to the quick preflight path. It asserts all 6 required schemas exist and are valid JSON (using Python's `json.load`). Any missing or malformed schema now causes a hard preflight failure, so silent schema drift is caught before the implement stage begins.

**Patch 3 — Strengthened planner prompt + removed hardcoded local path**  
The planner prompt now specifies four required sections (`## Current Factory Gaps`, `## First Three Patches`, `## Validation Commands`, `## Risks and Guardrails`) with explicit content rules. The `goal_gate=true` on the plan node now has structural meaning. Separately, the hardcoded `working_dir = "/Users/ajmal/Maestro/fabro-maestro"` was removed from `workflow.toml` — it was a developer-local artefact that was ignored by Daytona but confusing to read.

**Validation:** `.fabro/scripts/factory-validate.sh --quick` passes with all 6 schemas confirmed valid.