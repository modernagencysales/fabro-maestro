# Factory Risk Policy

Classify factory changes by operational risk:

- BLOCKER: secret exposure, destructive commands, wrong repository, or changes outside the requested scope.
- HIGH: workflow routing that ignores failed validation or reviewer decisions.
- MEDIUM: weak validation, missing artifacts, fragile sandbox bootstrap, or unrecoverable PR helper behavior.
- LOW: documentation drift, naming clarity, or missing optional metrics.

BLOCKER and HIGH findings require fixes before release. MEDIUM findings require either a fix or an explicit follow-up in release notes.
