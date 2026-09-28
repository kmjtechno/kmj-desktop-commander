import assert from "node:assert/strict";
import test from "node:test";
import { FixedWindowLimiter, ReplayGuard } from "./security.js";

test("rate limiter denies excess requests and resets", () => {
  const limiter = new FixedWindowLimiter(2, 1000);
  assert.equal(limiter.allow("device", 0), true);
  assert.equal(limiter.allow("device", 1), true);
  assert.equal(limiter.allow("device", 2), false);
  assert.equal(limiter.allow("device", 1000), true);
});

test("replay guard rejects duplicate and malformed operation ids", () => {
  const guard = new ReplayGuard(1000);
  const id = "operation_1234567890";
  assert.equal(guard.accept(id, 0), true);
  assert.equal(guard.accept(id, 1), false);
  assert.equal(guard.accept("short", 1), false);
  assert.equal(guard.accept(id, 1000), true);
});
