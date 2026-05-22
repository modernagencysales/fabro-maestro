# Factory Output Contracts

Factory runs should write stable artifacts under `.factory/`:

- `.factory/self/plan.md`: current improvement plan
- `.factory/self/validation.json`: deterministic validation result
- `.factory/self/scope-guard.json`: changed-file and line-count guard result
- `.factory/reviews/<reviewer>.md`: individual read-only review
- `.factory/reviews/consolidated.md`: merged review decision
- `.factory/release/release.md`: release notes and PR summary

Runtime logs may exist locally, but must not be committed.
