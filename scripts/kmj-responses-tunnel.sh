#!/usr/bin/env bash
set -Eeuo pipefail

TUNNEL_ID="${KMJ_TUNNEL_ID:-tunnel_6abaeed07a988191b81413b6e9da1153}"
MODEL="${OPENAI_MODEL:-gpt-5.6}"
API_BASE="${OPENAI_API_BASE:-https://api.openai.com/v1}"
MODE="${1:-verify}"

readonly READ_ONLY_TOOLS=(
  commander_project_inspect
  commander_git_status
  commander_read_project_file
)

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v python3 >/dev/null 2>&1 || die "python3 is required"
[[ -n "${OPENAI_API_KEY:-}" ]] || die "OPENAI_API_KEY is not set in this shell"

case "$MODE" in
  verify|inspect) ;;
  *) die "usage: $0 [verify|inspect]" ;;
esac

request_file="$(mktemp)"
response_file="$(mktemp)"
trap 'rm -f "$request_file" "$response_file"' EXIT

export KMJ_RESPONSES_TUNNEL_ID="$TUNNEL_ID"
export KMJ_RESPONSES_MODEL="$MODEL"
export KMJ_RESPONSES_MODE="$MODE"

if [[ "$MODE" == "inspect" ]]; then
  : "${KMJ_VERIFY_HOST:?KMJ_VERIFY_HOST is required for inspect mode}"
  : "${KMJ_VERIFY_USER:?KMJ_VERIFY_USER is required for inspect mode}"
  : "${KMJ_VERIFY_PROJECT_ROOT:?KMJ_VERIFY_PROJECT_ROOT is required for inspect mode}"
  export KMJ_VERIFY_PORT="${KMJ_VERIFY_PORT:-22}"
fi

python3 - "$request_file" <<'PY'
import json
import os
import sys

out = sys.argv[1]
mode = os.environ["KMJ_RESPONSES_MODE"]

tool = {
    "type": "mcp",
    "server_label": "kmj_desktop_commander",
    "server_description": "Private KMJ Desktop Commander over OpenAI Secure MCP Tunnel. Read-only verification surface.",
    "tunnel_id": os.environ["KMJ_RESPONSES_TUNNEL_ID"],
    "allowed_tools": [
        "commander_project_inspect",
        "commander_git_status",
        "commander_read_project_file",
    ],
    "require_approval": "never",
}

if mode == "verify":
    prompt = (
        "Connection verification only. Discover the MCP tools from "
        "kmj_desktop_commander, but do not call any MCP tool. "
        "Reply only that discovery completed."
    )
else:
    prompt = (
        "Read-only verification only. Call commander_project_inspect exactly once "
        "with host={host}, username={user}, port={port}, projectRoot={root}. "
        "Do not call any other tool and do not request or perform any write, restart, "
        "quality gate, sudo, or configuration change."
    ).format(
        host=os.environ["KMJ_VERIFY_HOST"],
        user=os.environ["KMJ_VERIFY_USER"],
        port=int(os.environ["KMJ_VERIFY_PORT"]),
        root=os.environ["KMJ_VERIFY_PROJECT_ROOT"],
    )

payload = {
    "model": os.environ["KMJ_RESPONSES_MODEL"],
    "input": prompt,
    "tools": [tool],
}
with open(out, "w", encoding="utf-8") as f:
    json.dump(payload, f, separators=(",", ":"))
PY

http_status="$(
  curl --silent --show-error     --output "$response_file"     --write-out '%{http_code}'     --connect-timeout 15     --max-time 120     -X POST "$API_BASE/responses"     -H "Content-Type: application/json"     -H "Authorization: Bearer $OPENAI_API_KEY"     --data-binary "@$request_file"
)"

[[ "$http_status" =~ ^2 ]] || {
  printf 'Responses API HTTP %s\n' "$http_status" >&2
  python3 - "$response_file" <<'PY' >&2
import json, sys
try:
    data=json.load(open(sys.argv[1], encoding="utf-8"))
    err=data.get("error", {})
    print(err.get("message") or "Responses API request failed")
except Exception:
    print("Responses API request failed; response was not valid JSON")
PY
  exit 1
}

python3 - "$response_file" "$MODE" <<'PY'
import json
import sys

path, mode = sys.argv[1], sys.argv[2]
data = json.load(open(path, encoding="utf-8"))

if data.get("error"):
    raise SystemExit("ERROR: " + str(data["error"].get("message", data["error"])))

output = data.get("output") or []
lists = [x for x in output if x.get("type") == "mcp_list_tools" and x.get("server_label") == "kmj_desktop_commander"]
if not lists:
    raise SystemExit("ERROR: no mcp_list_tools result was returned for kmj_desktop_commander")

expected = {
    "commander_project_inspect",
    "commander_git_status",
    "commander_read_project_file",
}
names = {t.get("name") for item in lists for t in (item.get("tools") or [])}
missing = expected - names
unexpected = names - expected
if missing:
    raise SystemExit("ERROR: read-only discovery incomplete; missing: " + ", ".join(sorted(missing)))
if unexpected:
    raise SystemExit("ERROR: unexpected MCP tools escaped the read-only allowlist: " + ", ".join(sorted(unexpected)))

calls = [x for x in output if x.get("type") == "mcp_call"]
if mode == "verify":
    if calls:
        raise SystemExit("ERROR: verify mode unexpectedly executed an MCP tool")
    print("VERIFIED: Secure MCP Tunnel discovery succeeded.")
    print("READ_ONLY_TOOLS=" + ",".join(sorted(names)))
    print("MCP_CALLS=0")
else:
    if len(calls) != 1 or calls[0].get("name") != "commander_project_inspect":
        called = ",".join(str(x.get("name")) for x in calls) or "none"
        raise SystemExit("ERROR: inspect mode expected exactly one commander_project_inspect call; got: " + called)
    call = calls[0]
    if call.get("error"):
        raise SystemExit("ERROR: commander_project_inspect failed: " + str(call["error"]))
    print("VERIFIED: Secure MCP Tunnel discovery and read-only project inspection succeeded.")
    print("READ_ONLY_TOOLS=" + ",".join(sorted(names)))
    print("MCP_CALL=commander_project_inspect")
    result = call.get("output")
    if result:
        print("--- READ-ONLY RESULT ---")
        print(result)

print("RESPONSE_ID=" + str(data.get("id", "")))
PY
