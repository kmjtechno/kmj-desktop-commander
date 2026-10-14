import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const configPath = path.join(root, "performance-budgets.json");
const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
const mode = process.argv[2] ?? "config";
const profileName = mode === "metrics" ? process.argv[4] : undefined;
const profile = profileName ? config.ci_profiles?.[profileName] : undefined;
if (profileName && !profile) {
  console.error(`PERF_BUDGET_FAIL: unknown CI profile: ${profileName}`);
  process.exit(1);
}
const b = structuredClone(config.budgets);
for (const [group, values] of Object.entries(profile?.budgets ?? {})) {
  b[group] = { ...(b[group] ?? {}), ...values };
}

function fail(message) {
  console.error(`PERF_BUDGET_FAIL: ${message}`);
  process.exitCode = 1;
}
function finite(name, value) {
  if (!Number.isFinite(value) || value < 0) fail(`${name} must be a non-negative finite number`);
}
function walkBytes(dir) {
  let total = 0;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    total += entry.isDirectory() ? walkBytes(p) : fs.statSync(p).size;
  }
  return total;
}
function assertMax(name, actual, max) {
  finite(name, actual);
  if (actual > max) fail(`${name}=${actual} exceeds ${max}`);
  else console.log(`PERF_BUDGET_OK: ${name}=${actual} <= ${max}`);
}

for (const [name, value] of [
  ["idle_cpu.p95", b.idle_cpu_percent?.p95_max],
  ["idle_cpu.hard", b.idle_cpu_percent?.hard_max],
  ["idle_rss.p95", b.idle_rss_mib?.p95_max],
  ["idle_rss.hard", b.idle_rss_mib?.hard_max],
  ["idle_network", b.idle_network_requests?.max_per_10_minutes],
  ["startup.warm", b.startup_ms?.warm_p95_max],
  ["startup.cold", b.startup_ms?.cold_p95_max],
  ["startup.hard", b.startup_ms?.hard_max],
  ["binary", b.binary_size_mib?.stripped_executable_max],
  ["installer", b.binary_size_mib?.installer_max],
  ["frontend", b.frontend_dist_mib?.raw_max],
]) finite(name, value);

if (mode === "config") {
  if (b.idle_cpu_percent.p95_max > b.idle_cpu_percent.hard_max) fail("CPU p95 budget exceeds hard ceiling");
  if (b.idle_rss_mib.p95_max > b.idle_rss_mib.hard_max) fail("RSS p95 budget exceeds hard ceiling");
  if (b.startup_ms.warm_p95_max > b.startup_ms.hard_max || b.startup_ms.cold_p95_max > b.startup_ms.hard_max) fail("startup p95 budget exceeds hard ceiling");
  if (!process.exitCode) console.log("Performance budget configuration is valid.");
} else if (mode === "frontend") {
  const dir = path.join(root, "dist");
  if (!fs.existsSync(dir)) fail("dist/ is missing; build before checking frontend budget");
  else assertMax("frontend_dist_mib", walkBytes(dir) / 1048576, b.frontend_dist_mib.raw_max);
} else if (mode === "metrics") {
  const metricsPath = process.argv[3];
  if (!metricsPath) fail("metrics mode requires a JSON metrics path");
  else {
    const m = JSON.parse(fs.readFileSync(path.resolve(root, metricsPath), "utf8"));
    const required = [
      "idle_cpu_p95_percent", "idle_cpu_max_percent", "idle_rss_p95_mib",
      "idle_rss_max_mib", "idle_network_requests_10m", "startup_warm_p95_ms",
      "startup_cold_p95_ms", "startup_max_ms", "stripped_executable_mib", "installer_mib"
    ];
    for (const key of required) if (!(key in m)) fail(`missing required metric: ${key}`);
    if (!process.exitCode) {
      assertMax("idle_cpu_p95_percent", m.idle_cpu_p95_percent, b.idle_cpu_percent.p95_max);
      assertMax("idle_cpu_max_percent", m.idle_cpu_max_percent, b.idle_cpu_percent.hard_max);
      assertMax("idle_rss_p95_mib", m.idle_rss_p95_mib, b.idle_rss_mib.p95_max);
      assertMax("idle_rss_max_mib", m.idle_rss_max_mib, b.idle_rss_mib.hard_max);
      assertMax("idle_network_requests_10m", m.idle_network_requests_10m, b.idle_network_requests.max_per_10_minutes);
      assertMax("startup_warm_p95_ms", m.startup_warm_p95_ms, b.startup_ms.warm_p95_max);
      assertMax("startup_cold_p95_ms", m.startup_cold_p95_ms, b.startup_ms.cold_p95_max);
      assertMax("startup_max_ms", m.startup_max_ms, b.startup_ms.hard_max);
      assertMax("stripped_executable_mib", m.stripped_executable_mib, b.binary_size_mib.stripped_executable_max);
      assertMax("installer_mib", m.installer_mib, b.binary_size_mib.installer_max);
    }
  }
} else {
  fail(`unknown mode: ${mode}`);
}
