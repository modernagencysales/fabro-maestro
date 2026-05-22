#!/usr/bin/env bash
set -euo pipefail

mkdir -p .factory/self
changed=".factory/self/changed-files.txt"
{
  git diff --name-only
  git diff --cached --name-only
  if git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    git diff --name-only HEAD~1..HEAD
  fi
} | sort -u > "$changed"

if grep -E '(^|/)maestro-v2(/|$)|docs/maestro-v2|technical-development-spec-v1' "$changed"; then
  echo "FAIL: Maestro V2 files changed. Scope is fabro-maestro factory only."
  exit 1
fi

if grep -E '(^|/)(node_modules|target|dist|coverage|\.next|\.turbo)/' "$changed"; then
  echo "FAIL: generated/vendor artifact changed."
  exit 1
fi

if grep -E '(^|/)\.env($|\.|/)' "$changed"; then
  echo "FAIL: env file staged."
  exit 1
fi

added="$({
  git diff --numstat
  git diff --cached --numstat
  if git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    git diff --numstat HEAD~1..HEAD
  fi
} | awk '{s+=$1} END {print s+0}')"
removed="$({
  git diff --numstat
  git diff --cached --numstat
  if git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    git diff --numstat HEAD~1..HEAD
  fi
} | awk '{s+=$2} END {print s+0}')"

cat > .factory/self/scope-guard.json <<JSON
{
  "decision": "pass",
  "lines_added": ${added},
  "lines_removed": ${removed}
}
JSON

cat .factory/self/scope-guard.json
