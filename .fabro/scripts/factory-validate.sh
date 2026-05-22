#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"

mode="${1:-full}"
mkdir -p .factory/self
log=".factory/self/validation.log"
: > "$log"

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

write_result() {
  local decision="$1"
  local note="$2"
  cat > .factory/self/validation.json <<JSON
{
  "decision": "$(json_escape "$decision")",
  "mode": "$(json_escape "$mode")",
  "note": "$(json_escape "$note")"
}
JSON
}

fail() {
  local message="$1"
  echo "FAIL: ${message}" | tee -a "$log"
  write_result "fail" "$message"
  exit 1
}

safe_remote() {
  git remote get-url origin 2>/dev/null \
    | sed -E \
      -e 's#(https://x-access-token:)[^@]+@#\1REDACTED@#' \
      -e 's#(https://)[^/@:]+:[^@]+@#\1REDACTED@#' \
      -e 's#(token=)[^&]+#\1REDACTED#g' \
    || true
}

run() {
  local name="$1"
  shift
  echo "=== ${name} ===" | tee -a "$log"
  "$@" 2>&1 | tee -a "$log"
}

echo "repo=$(safe_remote)" | tee -a "$log"
echo "mode=${mode}" | tee -a "$log"

if [[ "$mode" == "--quick" || "$mode" == "quick" ]]; then
  echo "=== quick preflight checks ===" | tee -a "$log"
  test -f docs/factory/maestro-fabro-perfect-software-factory.md \
    || fail "missing docs/factory/maestro-fabro-perfect-software-factory.md"
  test -f .fabro/workflows/factory-self-improve/workflow.fabro \
    || fail "missing factory-self-improve workflow"
  test -f .fabro/workflows/factory-self-improve/workflow.toml \
    || fail "missing factory-self-improve run config"
  if [[ "${FACTORY_VALIDATE_ALLOW_MISSING_PLAN:-0}" != "1" ]]; then
    test -f .factory/self/plan.md \
      || fail "missing required planner artifact: .factory/self/plan.md"
  else
    echo "WARN: skipping plan artifact check (FACTORY_VALIDATE_ALLOW_MISSING_PLAN=1)" | tee -a "$log"
  fi
  while IFS= read -r -d '' script; do
    test -x "$script" || fail "script is not executable: $script"
  done < <(find .fabro/scripts -maxdepth 1 -name '*.sh' -print0 | sort -z)

  # Assert required eval schemas exist and are valid JSON
  required_schemas=(
    ".fabro/evals/schemas/validation.schema.json"
    ".fabro/evals/schemas/risk-report.schema.json"
    ".fabro/evals/schemas/review-finding.schema.json"
    ".fabro/evals/schemas/consolidated-review.schema.json"
    ".fabro/evals/schemas/spec-eval.schema.json"
    ".fabro/evals/schemas/release-readiness.schema.json"
  )
  echo "=== eval schema checks ===" | tee -a "$log"
  for schema in "${required_schemas[@]}"; do
    test -f "$schema" || fail "missing required eval schema: $schema"
    if command -v python3 >/dev/null 2>&1; then
      python3 -c "import json,sys; json.load(open('$schema'))" 2>&1 | tee -a "$log" \
        || fail "invalid JSON in eval schema: $schema"
    fi
    echo "  ok: $schema" | tee -a "$log"
  done

  write_result "pass" "Quick filesystem preflight passed."
  echo "validation_passed_quick" | tee -a "$log"
  exit 0
fi

if command -v fabro >/dev/null 2>&1; then
  FABRO_BIN="$(command -v fabro)"
elif [[ -x target/debug/fabro ]]; then
  FABRO_BIN="target/debug/fabro"
elif command -v cargo >/dev/null 2>&1; then
  run "build repo-local fabro cli" cargo build -q -p fabro-cli
  FABRO_BIN="target/debug/fabro"
else
  fail "neither fabro nor cargo is available"
fi

run "fabro version" "$FABRO_BIN" --version

while IFS= read -r -d '' workflow; do
  run "fabro validate ${workflow}" "$FABRO_BIN" validate "$workflow"
done < <(find .fabro/workflows -name workflow.fabro -print0 | sort -z)

if command -v cargo >/dev/null 2>&1; then
  if rustup toolchain list 2>/dev/null | grep -q "nightly-2026-04-14"; then
    run "cargo fmt check" cargo +nightly-2026-04-14 fmt --check --all
  else
    echo "WARN: nightly-2026-04-14 missing; skipping fmt check" | tee -a "$log"
  fi
  run "cargo check workspace" cargo check -q --workspace
else
  echo "WARN: cargo missing; skipping Rust checks" | tee -a "$log"
fi

if command -v bun >/dev/null 2>&1 && [[ -d apps/fabro-web ]]; then
  run "fabro-web typecheck" bash -lc 'cd apps/fabro-web && bun install --frozen-lockfile && bun run typecheck'
else
  echo "WARN: bun or apps/fabro-web missing; skipping TypeScript checks" | tee -a "$log"
fi

write_result "pass" "Full factory validation passed."
echo "validation_passed" | tee -a "$log"