#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
config_dir="${HOME}/.config/kmj-commander"
state_dir="${HOME}/.local/state/kmj-commander"
bin_dir="${HOME}/.local/bin"
service_dir="${HOME}/.config/systemd/user"
env_file="${config_dir}/gateway.env"

cd "$repo_root"
cargo build --release --manifest-path headless/Cargo.toml
install -d -m 0700 "$config_dir" "$state_dir/audit" "$bin_dir" "$service_dir"
install -m 0755 headless/target/release/kmj-commander-headless "$bin_dir/kmj-commander-headless"
install -m 0644 ops/systemd/kmj-commander-headless.service "$service_dir/kmj-commander-headless.service"

if [[ ! -f "$env_file" ]]; then
  secret="$(openssl rand -hex 32)"
  {
    printf 'KMJ_COMMANDER_SIGNING_SECRET=%s\n' "$secret"
    printf 'KMJ_COMMANDER_SERVER_ID=%s\n' "$(hostname -s)"
    printf 'KMJ_COMMANDER_BIND=127.0.0.1:8770\n'
    printf 'KMJ_COMMANDER_AUDIT_PATH=%s/.local/state/kmj-commander/audit/events.jsonl\n' "$HOME"
    printf 'KMJ_COMMANDER_DEVICES_PATH=%s/.local/state/kmj-commander/devices.json\n' "$HOME"
  } > "$env_file"
  chmod 0600 "$env_file"
fi

systemctl --user daemon-reload
systemctl --user enable --now kmj-commander-headless.service
curl --fail --silent --show-error http://127.0.0.1:8770/v1/health

status="$(curl --silent --output /dev/null --write-out '%{http_code}' -X POST -H 'content-type: application/json' -d '{}' http://127.0.0.1:8770/v1/execute)"
[[ "$status" == "401" ]] || { echo "expected unauthenticated request to return 401, got $status" >&2; exit 1; }

set -a
# shellcheck disable=SC1090
source "$env_file"
set +a
device_id="chatgpt-local-verifier"
"$bin_dir/kmj-commander-headless" pair-device "$device_id" >/dev/null
token="$("$bin_dir/kmj-commander-headless" mint-token chatgpt "$device_id" 'commander:read,cloudos:read,cloudos:test,audit:read')"
request_id="$(cat /proc/sys/kernel/random/uuid)"
nonce="$(openssl rand -hex 16)"
timestamp="$(date +%s)"
payload="$(printf '{"protocol":"KMJ-COMMANDER/1","request_id":"%s","timestamp":%s,"nonce":"%s","principal":"chatgpt","operation":"commander.probe"}' "$request_id" "$timestamp" "$nonce")"
curl --fail --silent --show-error -X POST \
  -H "authorization: Bearer $token" \
  -H 'content-type: application/json' \
  --data "$payload" \
  http://127.0.0.1:8770/v1/execute

echo
"$bin_dir/kmj-commander-headless" revoke-device "$device_id" >/dev/null
revoked_status="$(curl --silent --output /dev/null --write-out '%{http_code}' -X POST \
  -H "authorization: Bearer $token" \
  -H 'content-type: application/json' \
  --data "$payload" \
  http://127.0.0.1:8770/v1/execute)"
[[ "$revoked_status" == "403" ]] || { echo "expected revoked device to return 403, got $revoked_status" >&2; exit 1; }

echo "KMJ Commander headless gateway verification PASS (pair + auth + execute + revoke)"
