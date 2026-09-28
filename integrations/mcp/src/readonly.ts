import { spawn } from "node:child_process";

type Check = { name: string; command: string };

const host = process.env.KMJ_COMMANDER_HOST ?? "";
const username = process.env.KMJ_COMMANDER_USER ?? "";
const port = Number(process.env.KMJ_COMMANDER_PORT ?? "22");
const projectRoot = process.env.KMJ_COMMANDER_PROJECT_ROOT ?? "";

if (!/^[A-Za-z0-9.:-]{1,253}$/.test(host)) throw new Error("Invalid KMJ_COMMANDER_HOST");
if (!/^[A-Za-z0-9_.-]{1,64}$/.test(username)) throw new Error("Invalid KMJ_COMMANDER_USER");
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error("Invalid KMJ_COMMANDER_PORT");
if (!/^\/[A-Za-z0-9/._-]{1,511}$/.test(projectRoot)) throw new Error("Invalid KMJ_COMMANDER_PROJECT_ROOT");

const checks: Check[] = [
  { name: "SERVER_PROBE", command: "printf 'HOSTNAME='; hostname; printf 'KERNEL='; uname -srm; printf 'UPTIME='; uptime -p 2>/dev/null || true" },
  { name: "CONTROL_CAPABILITIES", command: "printf 'USER='; id -un; printf 'UID='; id -u; printf 'PROJECT_WRITABLE='; test -w . && echo yes || echo no; printf 'SYSTEMCTL='; command -v systemctl >/dev/null && echo yes || echo no; printf 'SUDO_NONINTERACTIVE='; if sudo -n true >/dev/null 2>&1; then echo yes; else echo no; fi; printf 'GIT='; command -v git >/dev/null && echo yes || echo no; printf 'PHP='; command -v php >/dev/null && echo yes || echo no; printf 'NODE='; command -v node >/dev/null && echo yes || echo no" },
  { name: "PROJECT_INSPECT", command: "printf 'BRANCH='; git branch --show-current 2>/dev/null || true; printf '\\nSTATUS\\n'; git status --short --branch 2>/dev/null || true; printf '\\nSTACK\\n'; test -f composer.json && echo PHP; test -f package.json && echo NODE; test -f Cargo.toml && echo RUST; true" },
  { name: "GIT_STATUS", command: "git status --short --branch" },
  { name: "GIT_DIFF_CHECK", command: "git diff --check && git diff --stat" },
];

function run(command: string): Promise<{ code: number | null; output: string }> {
  const remote = `cd -- ${projectRoot} && ${command}`;
  const args = ["-o","BatchMode=yes","-o","StrictHostKeyChecking=yes","-o","ConnectTimeout=15","-p",String(port),"-l",username,host,remote];
  return new Promise((resolve, reject) => {
    const child = spawn("ssh", args, { shell: false, windowsHide: true });
    let output = "";
    const append = (chunk: Buffer) => { if (output.length < 262144) output += chunk.toString("utf8"); };
    child.stdout.on("data", append);
    child.stderr.on("data", append);
    child.once("error", reject);
    child.once("close", (code) => resolve({ code, output: output.slice(0, 262144) }));
  });
}

let failed = false;
for (const check of checks) {
  const result = await run(check.command);
  console.log(`===== ${check.name} =====`);
  process.stdout.write(result.output || `exit=${result.code ?? "unknown"}\n`);
  console.log(`===== ${check.name}_EXIT=${result.code ?? "unknown"} =====`);
  if (result.code !== 0) failed = true;
}
process.exitCode = failed ? 1 : 0;
