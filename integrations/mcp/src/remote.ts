import { timingSafeEqual } from "node:crypto";
import { createServer as createHttpServer } from "node:http";
import { toNodeHandler } from "@modelcontextprotocol/node";
import { createMcpHandler } from "@modelcontextprotocol/server";
import { createServer as createCommanderServer } from "./index.js";

const bind = process.env.KMJ_COMMANDER_BIND ?? "127.0.0.1";
const port = Number.parseInt(process.env.KMJ_COMMANDER_PORT ?? "8765", 10);
const token = process.env.KMJ_COMMANDER_BEARER_TOKEN ?? "";
const allowedHost = process.env.KMJ_COMMANDER_ALLOWED_HOST?.toLowerCase();

if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error("Invalid KMJ_COMMANDER_PORT.");
if (token.length < 32) throw new Error("KMJ_COMMANDER_BEARER_TOKEN must be at least 32 characters.");

function authorized(header: string | undefined): boolean {
  if (!header?.startsWith("Bearer ")) return false;
  const left = Buffer.from(header.slice(7));
  const right = Buffer.from(token);
  return left.length === right.length && timingSafeEqual(left, right);
}

const handler = createMcpHandler(createCommanderServer, { maxRequestBodySize: 1_048_576 });
const mcp = toNodeHandler(handler, {
  maxRequestBodySize: 1_048_576,
  onerror: (error) => console.error("KMJ MCP adapter error", error),
});

const http = createHttpServer((req, res) => {
  const url = new URL(req.url ?? "/", "http://localhost");

  if (url.pathname === "/healthz") {
    res.writeHead(200, { "content-type": "application/json", "cache-control": "no-store" });
    res.end(JSON.stringify({ ok: true, service: "kmj-desktop-commander-mcp" }));
    return;
  }
  if (url.pathname !== "/mcp") {
    res.writeHead(404, { "cache-control": "no-store" });
    res.end("Not found");
    return;
  }

  const host = (req.headers.host ?? "").split(":")[0]?.toLowerCase();
  if (allowedHost && host !== allowedHost && host !== "127.0.0.1" && host !== "localhost") {
    res.writeHead(403, { "cache-control": "no-store" });
    res.end("Forbidden host");
    return;
  }
  if (!authorized(req.headers.authorization)) {
    res.writeHead(401, {
      "content-type": "application/json",
      "cache-control": "no-store",
      "www-authenticate": 'Bearer realm="KMJ Desktop Commander"',
    });
    res.end(JSON.stringify({ error: "unauthorized" }));
    return;
  }

  void mcp(req, res);
});

http.requestTimeout = 30_000;
http.headersTimeout = 10_000;
http.keepAliveTimeout = 5_000;
http.listen(port, bind, () => console.error("KMJ Desktop Commander remote MCP ready on loopback."));

async function shutdown(): Promise<void> {
  http.close();
  await handler.close();
}
process.once("SIGINT", () => void shutdown());
process.once("SIGTERM", () => void shutdown());
