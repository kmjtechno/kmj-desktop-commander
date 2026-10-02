export interface CommanderPlan {
  label: string;
  price_enabled: boolean;
  server_profiles: number;
  devices: number;
  manual_operations: "unlimited" | number;
  autopilot_runs_per_month: number;
  execution_location: "customer";
}

export interface CommanderBootstrap {
  schema: number;
  product: "kmj-desktop-commander";
  channel: string;
  default_plan: string;
  billing_enabled: boolean;
  execution_model: "local-first";
  license_authority: "kmj-main-platform";
  entitlement_protocol: "KSLP-v1";
  entitlement_verification: "offline-ed25519";
  entitlement_refresh_seconds: number;
  bootstrap_refresh_seconds: number;
  bootstrap_refresh_jitter_seconds: number;
  idle_license_heartbeat: false;
  plans: Record<string, CommanderPlan>;
  platform_support: Record<string, "full" | "companion" | "none">;
}

export const PLATFORM_BOOTSTRAP =
  "https://kmjtechno.com/commander/bootstrap.json";

const CACHE_KEY = "kmj.commander.bootstrap.v1";
const FALLBACK_REFRESH_MS = 24 * 60 * 60 * 1000;
const FALLBACK_JITTER_MS = 6 * 60 * 60 * 1000;

interface CachedBootstrap {
  value: CommanderBootstrap;
  refresh_after: number;
}

function readCache(): CachedBootstrap | null {
  try {
    const raw = globalThis.localStorage?.getItem(CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as CachedBootstrap;
    if (!parsed.value || !Number.isFinite(parsed.refresh_after)) return null;
    return parsed;
  } catch {
    return null;
  }
}

function writeCache(value: CommanderBootstrap) {
  try {
    const refreshMs = Math.max(
      FALLBACK_REFRESH_MS,
      (value.bootstrap_refresh_seconds || 0) * 1000,
    );
    const jitterWindowMs = Math.max(
      0,
      (value.bootstrap_refresh_jitter_seconds || 0) * 1000,
    ) || FALLBACK_JITTER_MS;
    const jitter = Math.floor(Math.random() * jitterWindowMs);
    const cached: CachedBootstrap = {
      value,
      refresh_after: Date.now() + refreshMs + jitter,
    };
    globalThis.localStorage?.setItem(CACHE_KEY, JSON.stringify(cached));
  } catch {
    // Cache failure must never block local Commander operations.
  }
}

async function fetchNetworkBootstrap(): Promise<CommanderBootstrap> {
  const response = await fetch(PLATFORM_BOOTSTRAP, {
    method: "GET",
    cache: "force-cache",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    throw new Error(`KMJ Platform bootstrap failed: ${response.status}`);
  }
  const value = (await response.json()) as CommanderBootstrap;
  writeCache(value);
  return value;
}

export async function fetchCommanderBootstrap(): Promise<CommanderBootstrap> {
  const cached = readCache();
  if (cached && cached.refresh_after > Date.now()) {
    return cached.value;
  }

  try {
    return await fetchNetworkBootstrap();
  } catch (error) {
    if (cached) return cached.value;
    throw error;
  }
}
