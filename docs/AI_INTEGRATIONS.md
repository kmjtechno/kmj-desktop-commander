# AI integrations

KMJ Desktop Commander exposes a Model Context Protocol (MCP) v2 integration under `integrations/mcp`.

## Supported AI hosts

The stdio integration follows the current MCP 2026-07-28 SDK line and can be used by MCP hosts that can launch a local process. It intentionally exposes only allow-listed Commander operations:

- `commander_project_inspect`
- `commander_git_status`
- `commander_run_quality_gate`

There is no arbitrary shell tool.

## ChatGPT

ChatGPT custom apps connect to remote MCP servers rather than directly to a local stdio process. For private/local Commander installations, use OpenAI's supported Secure MCP Tunnel or a separately deployed authenticated Streamable HTTP bridge. Do not expose the local Commander MCP port directly to the public Internet.

Directory visibility is a separate publication/review step. Shipping this MCP implementation does not automatically create a public ChatGPT Plugins Directory listing.

## Other MCP clients

MCP clients that support local stdio can launch the integration directly after installing its dependencies. The integration uses the operating-system OpenSSH client and the same strict host-key/key-only assumptions as the desktop application.

## Safety

AI clients receive fixed operation IDs, not shell authority. Production, destructive, privilege escalation, firewall, secret and force-push operations are intentionally absent from this integration until explicit approval and policy protocols are implemented.
