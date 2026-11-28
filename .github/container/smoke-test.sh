#!/usr/bin/env bash
set -euo pipefail
image=${1:?Expected an image tag}
version=${2:?Expected a release version}
engine=${CONTAINER_ENGINE:-docker}
options=(--rm --read-only --network=none --cap-drop=ALL --security-opt=no-new-privileges)

user="$("$engine" image inspect --format '{{.Config.User}}' "$image")"
[[ "$user" == 65532:65532 ]] || { echo "Expected a non-root image, got user '$user'." >&2; exit 1; }

actual="$("$engine" run "${options[@]}" "$image" -V)"
[[ "$actual" == "cron-when $version" ]] || { echo "Unexpected image version: $actual" >&2; exit 1; }
output="$("$engine" run "${options[@]}" "$image" --no-color '*/5 * * * *')"
grep -Fq 'Next:' <<<"$output"
grep -Fq 'UTC' <<<"$output"

fixture="$(mktemp)"
trap 'rm -f -- "$fixture"' EXIT
printf '%s\n' '*/5 * * * * /bin/true' > "$fixture"
chmod 0644 "$fixture"
volume_options=ro
if [[ "$engine" == podman ]]; then volume_options=ro,Z; fi
output="$("$engine" run "${options[@]}" -v "$fixture:/schedule:$volume_options" "$image" --no-color --file /schedule)"
grep -Fq 'Next:' <<<"$output"
echo "$actual runs as $user and can read a mounted crontab."
