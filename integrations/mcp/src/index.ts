import { spawn } from "node:child_process";
import { McpServer } from "@modelcontextprotocol/server";
import { serveStdio } from "@modelcontextprotocol/server/stdio";
import * as z from "zod/v4";

const profile = z.object({
  host: z.string().min(1).max(253).regex(/^[A-Za-z0-9.:-]+$/),
  username: z.string().min(1).max(64).regex(/^[A-Za-z0-9_.-]+$/),
  port: z.number().int().min(1).max(65535).default(22),
  projectRoot: z.string().min(1).max(512).regex(/^\/[A-Za-z0-9/._-]+$/),
});

const relativePath = z.string().min(1).max(384).regex(/^[A-Za-z0-9/._-]+$/).refine((value) => !value.startsWith("-") && !value.split("/").includes(".."), "Unsafe relative path");
const serviceName = z.string().min(1).max(128).regex(/^[A-Za-z0-9@_.-]+$/);
const gate = z.enum(["git_diff_check", "php_test", "frontend_typecheck", "frontend_build", "rust_test"]);
type Gate = z.infer<typeof gate>;

const commands: Record<Gate, string> = {
  git_diff_check: "git diff --check && git diff --stat",
  php_test: "php artisan test",
  frontend_typecheck: "pnpm typecheck",
  frontend_build: "pnpm build",
  rust_test: "cargo test",
};

async function ssh(input: z.infer<typeof profile>, command: string) {
  const remote = `cd -- ${input.projectRoot} && ${command}`;
  const args = [
    "-o", "BatchMode=yes",
    "-o", "StrictHostKeyChecking=yes",
    "-o", "ConnectTimeout=10",
    "-p", String(input.port),
    "-l", input.username,
    input.host,
    remote,
  ];

  return await new Promise<{ success: boolean; exitCode: number | null; output: string }>((resolve, reject) => {
    const child = spawn("ssh", args, { shell: false, windowsHide: true });
    let output = "";
    const append = (chunk: Buffer) => {
      if (output.length < 262144) output += chunk.toString("utf8");
    };
    child.stdout.on("data", append);
    child.stderr.on("data", append);
    child.once("error", (error) => reject(new Error(`OpenSSH failed to start: ${error.message}`)));
    child.once("close", (code) => resolve({ success: code === 0, exitCode: code, output: output.slice(0, 262144) }));
  });
}

function result(value: Awaited<ReturnType<typeof ssh>>) {
  return {
    content: [{ type: "text" as const, text: value.output || `exit=${value.exitCode ?? "unknown"}` }],
    structuredContent: value,
    isError: !value.success,
  };
}

function createServer() {
  const server = new McpServer({ name: "KMJ Desktop Commander", version: "0.1.0" });

  server.registerTool("commander_project_inspect", {
    title: "Inspect project",
    description: "Read the branch, Git status and detected stack on a customer-controlled server through strict OpenSSH.",
    inputSchema: profile,
    annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: true },
  }, async (input) => result(await ssh(input, "printf 'BRANCH='; git branch --show-current 2>/dev/null || true; printf '\\nSTATUS\\n'; git status --short --branch 2>/dev/null || true; printf '\\nSTACK\\n'; test -f composer.json && echo PHP; test -f package.json && echo NODE; test -f Cargo.toml && echo RUST")));

  server.registerTool("commander_git_status", {
    title: "Git status",
    description: "Read the current branch and changed files without modifying the project.",
    inputSchema: profile,
    annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: true },
  }, async (input) => result(await ssh(input, "git status --short --branch")));

  server.registerTool("commander_run_quality_gate", {
    title: "Run quality gate",
    description: "Run one fixed, allow-listed quality gate. Arbitrary shell text is not accepted.",
    inputSchema: profile.extend({ gate }),
    annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: true },
  }, async ({ gate: selected, ...input }) => result(await ssh(input, commands[selected])));

  server.registerTool("commander_read_project_file", {
    title: "Read project file",
    description: "Read up to the first 400 lines of a validated relative file inside the configured project root.",
    inputSchema: profile.extend({ path: relativePath }),
    annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: true },
  }, async ({ path, ...input }) => result(await ssh(input, `sed -n '1,400p' -- ./${path}`)));

  server.registerTool("commander_write_project_file", {
    title: "Write project file",
    description: "Atomically replace one validated file inside the project root. This is a reversible project-workspace operation; no sudo is used.",
    inputSchema: profile.extend({ path: relativePath, content: z.string().max(1048576) }),
    annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: true },
  }, async ({ path, content, ...input }) => {
    const encoded = Buffer.from(content, "utf8").toString("base64");
    return result(await ssh(input, `umask 077; mkdir -p -- ./$(dirname -- ${path}); printf '%s' '${encoded}' | base64 -d > ./${path}.kmj-tmp && mv -- ./${path}.kmj-tmp ./${path}`));
  });

  server.registerTool("commander_service_control", {
    title: "Control system service",
    description: "Inspect or restart a validated systemd service. Restart requires explicit approval and passwordless sudo on the customer-controlled server.",
    inputSchema: profile.extend({ service: serviceName, action: z.enum(["status", "restart"]), approved: z.boolean().default(false) }),
    annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: true },
  }, async ({ service, action, approved, ...input }) => {
    if (action === "restart" && !approved) {
      return { content: [{ type: "text" as const, text: "DENIED: service restart requires explicit approval." }], isError: true };
    }
    const command = action === "status"
      ? `systemctl --no-pager --full status ${service} || true`
      : `sudo -n systemctl restart ${service} && systemctl is-active ${service}`;
    return result(await ssh(input, command));
  });

  return server;
}

void serveStdio(createServer);
console.error("KMJ Desktop Commander MCP server ready on stdio");
