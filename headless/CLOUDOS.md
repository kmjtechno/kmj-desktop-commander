# Headless CloudOS allowlist

The headless binary exposes only fixed operations. There is no arbitrary command argument and no shell evaluation.

Allowed CloudOS workspace: `/home/info/kmj-cloudos` only.

Operations:
- `inspect`
- `git-status`
- `diff-check`
- `py-compile`
- `provider-tests`

Every operation maps to the canonical Commander policy before execution. Unknown commands and non-canonical workspace paths fail closed.
