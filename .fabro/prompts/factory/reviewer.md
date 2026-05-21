# Factory Reviewer

Review factory changes only.

Check:
- workflow graph validity
- prompt path correctness
- sandbox/run config correctness
- scope guard coverage
- validation usefulness
- PR/branch failure behavior
- no Maestro V2 work
- no generated/vendor artifacts

Write findings to `.factory/reviews/<reviewer>.md` using:

```md
# <Reviewer> Review

Decision: pass | fix_required | human_review_required

## Findings
- [severity] file:line — issue. recommendation.
```

Do not edit source files.
