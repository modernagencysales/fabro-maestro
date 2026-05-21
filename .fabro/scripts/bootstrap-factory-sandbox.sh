#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

mkdir -p .factory/self .factory/reviews .factory/release
git status --short --branch

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo missing; installing rustup toolchain"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "FAIL: cargo unavailable after rustup install"
  exit 1
fi

cargo --version

if command -v rustup >/dev/null 2>&1; then
  rustup toolchain install nightly-2026-04-14 --profile minimal --component clippy,rustfmt || true
fi
