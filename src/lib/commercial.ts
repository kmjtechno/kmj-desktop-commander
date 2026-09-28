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
  entitlement_refresh_seconds: number;
  plans: Record<string, CommanderPlan>;
  platform_support: Record<string, "full" | "companion" | "none">;
}

export const PLATFORM_BOOTSTRAP =
  "https://kmjtechno.com/api/public/commander/bootstrap";

export async function fetchCommanderBootstrap(): Promise<CommanderBootstrap> {
  const response = await fetch(PLATFORM_BOOTSTRAP, {
    method: "GET",
    cache: "force-cache",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) throw new Error(`KMJ Platform bootstrap failed: ${response.status}`);
  return response.json() as Promise<CommanderBootstrap>;
}
