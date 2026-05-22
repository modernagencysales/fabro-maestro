#!/usr/bin/env bash
set -euo pipefail

mode="${1:-full}"
mkdir -p .factory/self
log=".factory/self/validation.log"
: > "$log"

run() {
  local name="$1"
  shift
  echo "=== ${name} ===" | tee -a "$log"
  "$@" 2>&1 | tee -a "$log"
}

echo "repo=$(git remote get-url origin 2>/dev/null || true)" | tee -a "$log"
echo "mode=${mode}" | tee -a "$log"

if command -v fabro >/dev/null 2>&1; then
  FABRO_BIN="$(command -v fabro)"
elif [[ -x target/debug/fabro ]]; then
  FABRO_BIN="target/debug/fabro"
elif command -v cargo >/dev/null 2>&1; then
  run "build repo-local fabro cli" cargo build -q -p fabro-cli
  FABRO_BIN="target/debug/fabro"
else
  echo "FAIL: neither fabro nor cargo is available" | tee -a "$log"
  exit 1
fi

run "fabro version" "$FABRO_BIN" --version

while IFS= read -r -d '' workflow; do
  run "fabro validate ${workflow}" "$FABRO_BIN" validate "$workflow"
done < <(find .fabro/workflows -name workflow.fabro -print0 | sort -z)

if [[ "$mode" == "--quick" || "$mode" == "quick" ]]; then
  echo "validation_passed_quick" | tee -a "$log"
  exit 0
fi

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

echo "validation_passed" | tee -a "$log"
