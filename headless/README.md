# KMJ Commander Headless

Minimal Linux/headless entry point that compiles without Tauri/GTK/WebKit.

It reuses the canonical deny-by-default policy module from `src-tauri/src/policy.rs`.

## Build

```bash
cd headless
cargo test
cargo build --release
./target/release/kmj-commander-headless probe
./target/release/kmj-commander-headless policy project.inspect
./target/release/kmj-commander-headless project-inspect /home/info/kmj-cloudos
```

This slice intentionally exposes only probe, policy evaluation, and read-only project inspection. It does not expose an arbitrary shell, privileged operations, or a network listener.
