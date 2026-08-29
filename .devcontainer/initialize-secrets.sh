#!/usr/bin/env sh
set -eu

# This script runs on the host before the container is created. It stages an
# optional age identity outside the repository so devcontainer.json can mount a
# stable, read-only file without embedding secret values in configuration.
secret_dir="${HOME}/.cache/devcontainer-secrets/cron-when"
secret_file="${secret_dir}/sops-age-key"

mkdir -p "$secret_dir"
chmod 700 "$secret_dir"

temporary_file="$(mktemp "${secret_file}.tmp.XXXXXX")"
trap 'rm -f "$temporary_file"' EXIT HUP INT TERM
chmod 600 "$temporary_file"

if [ -n "${SOPS_AGE_KEY:-}" ]; then
    printf '%s' "$SOPS_AGE_KEY" >"$temporary_file"
elif [ -n "${SOPS_AGE_KEY_FILE:-}" ]; then
    if [ ! -r "$SOPS_AGE_KEY_FILE" ]; then
        echo "Error: SOPS_AGE_KEY_FILE is not readable: $SOPS_AGE_KEY_FILE" >&2
        exit 1
    fi
    cp "$SOPS_AGE_KEY_FILE" "$temporary_file"
elif [ -n "${SOPS_AGE_KEY_OP_REF:-}" ]; then
    if ! command -v op >/dev/null 2>&1; then
        echo "Error: the 1Password CLI is required for SOPS_AGE_KEY_OP_REF" >&2
        exit 1
    fi
    op read --no-newline "$SOPS_AGE_KEY_OP_REF" >"$temporary_file"
else
    echo "No age identity configured; SOPS decryption will be unavailable."
    echo "Set SOPS_AGE_KEY_FILE (recommended), SOPS_AGE_KEY, or SOPS_AGE_KEY_OP_REF."
fi

chmod 600 "$temporary_file"
mv -f "$temporary_file" "$secret_file"
trap - EXIT HUP INT TERM
