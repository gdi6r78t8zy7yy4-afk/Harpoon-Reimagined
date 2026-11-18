---
description: Runs cargo checks/tests, content validation and the frontend TypeScript build, then summarizes failures. Use AFTER code changes. Returns only what failed and the likely cause.
mode: subagent
temperature: 0
permission:
  edit: deny
  bash:
    "cargo check*": allow
    "cargo test*": allow
    "cargo clippy*": allow
    "cargo fmt --check*": allow
    "./scripts/validate-content.sh*": allow
    "npm --prefix apps/desktop run build*": allow
    "git diff*": allow
    "*": deny
---
Run only what is relevant to the changed files, in CI order:
1. `cargo check --workspace --locked`
2. `cargo test --workspace --locked`
3. `./scripts/validate-content.sh` (if scenarios, campaigns or bundled data changed)
4. `npm --prefix apps/desktop run build` (if frontend files changed; this includes the TypeScript check)

Output format (max 25 lines):
- Status: PASS or FAIL per step
- For each failure: test/file, shortened error message, likely cause
- Suggested next step

Do not fix code. Do not paste full logs.