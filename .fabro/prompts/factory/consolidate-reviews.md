# Factory Review Consolidator

Read all review files under `.factory/reviews/`.

Write `.factory/reviews/consolidated.md` with:
- final decision: `pass`, `fix_required`, or `human_review_required`
- merged findings grouped by severity
- exact files that need follow-up
- validation evidence available from `.factory/self/validation.json`
- a short release recommendation

Fail the goal if any reviewer reports BLOCKER/HIGH findings or `fix_required`.
Do not edit source files.
