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
- `.factory/self/plan.md` (if it exists — carry forward any unimplemented patches)

## Output contract

Write `.factory/self/plan.md`. The file **must** contain all four of these sections in order — the goal gate will fail without them:

### Required sections

```
## Current Factory Gaps
```
List every significant gap between the current `.fabro/` state and the design spec.
For each gap: name it, explain why it matters, and classify it (BLOCKER / HIGH / MEDIUM / LOW).

```
## First Three Patches
```
Describe exactly three concrete, ordered patches for the implementer to apply next.
Each patch must specify:
- Files to create or modify (exact paths)
- What change to make (precise description)
- Why it is safe (no behavior regression risk)
- Validation command to confirm it worked

Pick patches that are independently verifiable and do not require more than ~200 lines of new content each.
Prefer patches that close BLOCKER or HIGH gaps first.
Do not describe patches already implemented in the current `.fabro/` state.

```
## Validation Commands
```
List the shell commands to verify the three patches are correctly applied.
Always include `.fabro/scripts/factory-validate.sh --quick` as the first command.

```
## Risks and Guardrails
```
For each patch: what could go wrong and how is it mitigated?
Always include the Maestro V2 scope guard and the max_node_visits bound as guardrails.

## Prioritization rules

1. Gaps that make the factory measurable (eval schemas, validation checks) > gaps that improve speed.
2. Gaps that prevent silent failures (missing artifacts, missing goal conditions) > gaps that add features.
3. Gaps that improve inspector clarity (prompts, contracts) > gaps that expand reviewer coverage.
4. Anything that writes to Maestro V2 paths is out of scope and must not appear in the plan.

## After writing the plan

Summarize the three patches in one short paragraph (≤ 5 sentences) so the implementer node has a quick orientation before reading the full plan.
