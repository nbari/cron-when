#!/usr/bin/env bash
set -euo pipefail

echo "Rust development container ready"
rustc --version
cargo --version
just --version
sops --help >/dev/null
echo "sops installed at $(command -v sops)"
age --version

if [[ -s "${SOPS_AGE_KEY_FILE}" ]]; then
    echo "✓ SOPS age identity mounted read-only at ${SOPS_AGE_KEY_FILE}"
else
    echo "ℹ No age identity was provided; encryption with public recipients still works."
fi

echo "Run 'just test' for the fast local validation suite."
