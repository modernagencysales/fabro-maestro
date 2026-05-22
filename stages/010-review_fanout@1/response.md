I have successfully completed the QA review. I found a HIGH severity issue in the validation logic and documented it in `.factory/reviews/qa.md`.

### Summary of findings:
The recent implementation was supposed to enforce the presence of `.factory/self/plan.md` and check the newly added evaluation JSON schemas. However, these checks were placed inside the `--quick` mode conditional in `.fabro/scripts/factory-validate.sh` (which ends in `exit 0`). Because the `validate` workflow node executes this script in `full` mode (without arguments), these newly implemented assertions are completely skipped during the main `implement -> validate` cycle.

I have set `Decision: fix_required` to block the release until the validation checks are moved so they execute unconditionally for both `quick` and `full` runs.