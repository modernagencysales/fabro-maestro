# Factory Self-Improvement Plan

**Generated:** 2026-05-22  
**Run ID:** 01KS6HRJR3E0A5FZDTJP44DGM7  
**Design source:** `docs/factory/maestro-fabro-perfect-software-factory.md`

---

## Current Factory State

The factory-self-improve workflow exists and is structurally sound. It implements:

- ✅ Preflight guard (origin + quick filesystem checks)
- ✅ Plan stage (`.fabro/prompts/factory/planner.md`)
- ✅ Implement stage with coding model + full fidelity + thread_id
- ✅ Scope guard (Maestro V2 + env + artifact exclusions)
- ✅ Validate stage (`factory-validate.sh` — quick + full modes)
- ✅ Fix validation loop (max_visits=3, same thread)
- ✅ Parallel review fan-out (QA / Security / Architecture)
- ✅ Merge + consolidate reviews
- ✅ Fix review findings loop (max_visits=2)
- ✅ Release notes stage
- ✅ Open PR (commit + push + gh pr create)
- ✅ Eval schemas: validation, risk-report, review-finding, consolidated-review
- ✅ Shared prompts: risk-policy, output-contracts, reviewer-json-contract
- ✅ Bootstrap script + Dockerfile sandbox definition

---

## Current Factory Gaps

### Gap 1 — No `spec-eval.schema.json` or `release-readiness.schema.json`

The design spec (§12.2, §12.8) defines two critical eval schemas:
- `spec-eval.schema.json` — scores spec quality (clarity, acceptance criteria, non-goals, etc.)
- `release-readiness.schema.json` — scores release readiness (blockers, rollback plan, manual QA steps)

These are missing from `.fabro/evals/schemas/`. Without them:
- The plan/consolidate stages have no machine-checkable output contract.
- The `factory-validate.sh` full-mode cannot assert schema conformance.
- Future eval/retro loops have no baseline schema.

### Gap 2 — `factory-validate.sh` full mode doesn't validate eval schemas

The full validation path (`factory-validate.sh` without `--quick`) checks:
- `fabro validate` on each workflow
- cargo fmt + cargo check
- bun typecheck

It does **not**:
- Validate that `.fabro/evals/schemas/*.schema.json` are valid JSON Schema.
- Check that reviewer output files (if present) conform to their contracts.
- Assert that the plan artifact (`.factory/self/plan.md`) was written.

This means the factory can silently skip writing required artifacts.

### Gap 3 — Planner prompt has no explicit output contract

`.fabro/prompts/factory/planner.md` tells the planner what to write but does not specify:
- The exact required sections of `plan.md`.
- A machine-checkable "done" condition.
- Which next slice to pick from the design spec (prioritization logic).

The `goal_gate=true` on the plan node fires on any non-empty output — there is no structural assertion. This means a vague plan passes the gate.

### Gap 4 — No `factory-retro.schema.json` and no `maestro-retro.fabro` workflow

The design spec (§23) defines a retrospective loop that every run should produce:
- `.factory/retro/run-retro.md`
- `.factory/retro/metrics.json`

Neither the retro schema nor a retro workflow exist. Without these:
- Each run produces no structured metrics (token cost, fix loop counts, scores).
- The "optimizable" property (#8 in §2) of the perfect factory is unmet.
- There is no baseline for comparing prompt/model changes.

### Gap 5 — `workflow.toml` has `working_dir` hardcoded to a local path

`.fabro/workflows/factory-self-improve/workflow.toml` contains:
```toml
working_dir = "/Users/ajmal/Maestro/fabro-maestro"
```

This is a developer-local absolute path. In cloud/Daytona sandboxes, this is ignored (Daytona uses the sandbox workspace root), but it is misleading and could cause confusion when running locally on other machines or CI. The field should either be removed or set to a relative/placeholder value.

### Gap 6 — No `perform-review-findings` enforcement in scope-guard

`factory-scope-guard.sh` blocks Maestro V2 files and generated artifacts but does **not** verify that reviewer output files (`.factory/reviews/*.md`) were actually written by the review stages before consolidation. A reviewer stage that silently writes nothing (e.g. due to a prompt error) will make consolidation produce a vacuous "pass."

### Gap 7 — Missing prompts from the design spec's full prompt inventory

The design spec (§6) lists these build-phase prompts that do not yet exist:
- `.fabro/prompts/build/migration-reviewer.md`
- `.fabro/prompts/build/performance-reviewer.md`
- `.fabro/prompts/build/observability-reviewer.md`

The factory-self-improve workflow currently runs 3 parallel reviewers (QA, security, architecture). The full design calls for 6. The missing three are low-risk for the factory itself, but they represent the gap between the current 3-reviewer fan-out and the target 6-reviewer fan-out.

### Gap 8 — No `detect-risk-surfaces.sh` script

The design spec (§13) lists `detect-risk-surfaces.sh` as a required factory script. This script should inspect changed files and output a structured risk surface classification (used by the scope guard and risk classifier). Currently the scope-guard does a simple grep; there is no dedicated risk detection script that writes a `risk-report.json`.

---

## Priority Ordering

Given the design spec's Phase 2 goal ("move from agent completed to factory measured"), the highest leverage gaps are:

1. **Gap 1 + 2 together**: add missing eval schemas AND wire them into `factory-validate.sh` — makes the factory measurable.
2. **Gap 3**: strengthen the planner prompt with explicit output contract — prevents silent drift.
3. **Gap 5**: remove hardcoded local path from `workflow.toml` — cleanup that unblocks other contributors.

Gaps 4, 6, 7, 8 are follow-on slices (higher complexity / lower immediate risk).

---

## Patch 1 — Add missing eval schemas (`spec-eval` and `release-readiness`)

**Files to create:**
- `.fabro/evals/schemas/spec-eval.schema.json`
- `.fabro/evals/schemas/release-readiness.schema.json`

**Design source:** §12.2 (spec eval), §12.8 (release readiness)

**What:** JSON Schema files defining the machine-checkable output contracts for the spec evaluator and release readiness check. These schemas document what a passing spec or a release-ready build looks like.

**Why this slice is safe:** Pure additions, no behavior changes, no script logic, no model interaction.

**Validation command:**
```bash
python3 -c "
import json, pathlib, sys
schemas = list(pathlib.Path('.fabro/evals/schemas').glob('*.schema.json'))
errors = []
for s in schemas:
    try:
        obj = json.loads(s.read_text())
        assert '\$schema' in obj or 'type' in obj or 'properties' in obj, f'{s.name} looks empty'
    except Exception as e:
        errors.append(f'{s}: {e}')
if errors:
    print('\n'.join(errors)); sys.exit(1)
print(f'OK: {len(schemas)} schemas valid JSON')
"
```

---

## Patch 2 — Wire schema existence check into `factory-validate.sh --quick`

**Files to modify:**
- `.fabro/scripts/factory-validate.sh`

**What:** Add a check to the `--quick` preflight mode that asserts all required eval schemas are present and are valid JSON. Also assert that `.factory/self/plan.md` exists when running outside of the plan stage (i.e., in the implement→validate→review path).

**Why this slice is safe:** Additive check only. Cannot break existing passing runs — if schemas are present and valid, the check passes silently.

**Validation command:**
```bash
.fabro/scripts/factory-validate.sh --quick
```

---

## Patch 3 — Strengthen planner prompt with explicit output contract and fix `workflow.toml` path

**Files to modify:**
- `.fabro/prompts/factory/planner.md`
- `.fabro/workflows/factory-self-improve/workflow.toml`

**What:**
1. Add a required-sections contract to `planner.md` so the plan node's `goal_gate=true` can be meaningfully evaluated. The plan must contain: `## Current Factory Gaps`, `## First Three Patches`, `## Validation Commands`, `## Risks and Guardrails`.
2. Remove the hardcoded `working_dir = "/Users/ajmal/Maestro/fabro-maestro"` from `workflow.toml`.

**Why this slice is safe:**
- Prompt strengthening only changes what the LLM is asked to write — does not change workflow routing.
- Removing `working_dir` from a Daytona-based run config has no effect on sandbox execution (Daytona uses the workspace root regardless).

**Validation command:**
```bash
.fabro/scripts/factory-validate.sh --quick
```

---

## Validation Commands (for this plan)

```bash
# Quick preflight (should always pass)
.fabro/scripts/factory-validate.sh --quick

# Validate all schemas are present and valid JSON
python3 -c "
import json, pathlib, sys
schemas = list(pathlib.Path('.fabro/evals/schemas').glob('*.schema.json'))
for s in schemas:
    json.loads(s.read_text())
print(f'OK: {len(schemas)} schemas')
"

# Full factory validate (requires fabro binary or cargo)
.fabro/scripts/factory-validate.sh
```

---

## Risks and Guardrails

| Risk | Mitigation |
|---|---|
| Eval schema changes break existing passing runs | Schemas are additive; they document contracts, not enforce them at runtime yet |
| Strengthened planner prompt causes the plan stage to loop | `goal_gate=true` + `retry_target="plan"` already handles this; max_node_visits=10 bounds it |
| Removing `working_dir` breaks local dev | Local runs use the working directory of `fabro run` invocation; this field is advisory only |
| Schema check in `--quick` creates a false negative on fresh clones | Check must be `test -f` only — validate JSON only if file exists |
| Scope creep into Maestro V2 code | `factory-scope-guard.sh` blocks any change to maestro-v2 paths |

---

## Follow-on Slices (future pipeline stages)

- **Slice 4:** Add `factory-retro.schema.json` + `maestro-retro.fabro` workflow stub
- **Slice 5:** Add `detect-risk-surfaces.sh` and wire into scope guard JSON output
- **Slice 6:** Add migration/performance/observability reviewer prompts (expand fan-out to 6)
- **Slice 7:** Add `check-migrations.sh` and `check-rls.sh` scripts from design spec §13
- **Slice 8:** Add `maestro-spec.fabro` two-pass spec workflow
