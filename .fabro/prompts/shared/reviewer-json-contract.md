# Reviewer JSON Contract

Reviewers may write Markdown, but every finding should be representable as:

```json
{
  "severity": "BLOCKER | HIGH | MEDIUM | LOW",
  "file": "path/to/file",
  "line": 1,
  "issue": "What is wrong",
  "recommendation": "Concrete fix",
  "blocks_release": true
}
```

Use `blocks_release=true` for BLOCKER and HIGH findings.
