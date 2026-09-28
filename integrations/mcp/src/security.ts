export class FixedWindowLimiter {
  private readonly hits = new Map<string, { count: number; resetAt: number }>();
  constructor(private readonly limit: number, private readonly windowMs: number) {}
  allow(key: string, now = Date.now()): boolean {
    const current = this.hits.get(key);
    if (!current || now >= current.resetAt) {
      this.hits.set(key, { count: 1, resetAt: now + this.windowMs });
      return true;
    }
    if (current.count >= this.limit) return false;
    current.count += 1;
    return true;
  }
}

export class ReplayGuard {
  private readonly seen = new Map<string, number>();
  constructor(private readonly ttlMs: number, private readonly maxEntries = 10_000) {}
  accept(id: string, now = Date.now()): boolean {
    if (!/^[A-Za-z0-9_-]{16,128}$/.test(id)) return false;
    for (const [key, expires] of this.seen) if (expires <= now) this.seen.delete(key);
    if (this.seen.has(id)) return false;
    if (this.seen.size >= this.maxEntries) return false;
    this.seen.set(id, now + this.ttlMs);
    return true;
  }
}
