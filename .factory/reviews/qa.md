# QA Review

Decision: fix_required

## Findings
- [HIGH] .fabro/scripts/factory-validate.sh:52 — Validation checks for `.factory/self/plan.md` and eval schemas only run in `--quick` mode, which then exits early. Because the `validate` workflow node executes `.fabro/scripts/factory-validate.sh` in `full` mode, it entirely skips these new checks. This effectively nullifies the implementation's goal of asserting artifact presence in the `implement -> validate` path. Move the preflight and schema checks outside the `if [[ "$mode" == "--quick" ... ]]` conditional block so they are executed in both `quick` and `full` validation modes, keeping the `exit 0` only for when `--quick` is specified.
