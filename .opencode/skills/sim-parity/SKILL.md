---
name: sim-parity
description: Rules for changing simulation behavior (movement, sensors, weapons, escalation) so Rust core and the frontend simulation stay in sync. Load before editing anything in src/simulation-core or apps/desktop/src/**/simulation.ts.
---
The browser runs its own simulation (`simulation.ts`) when `__TAURI_INTERNALS__` is absent.
The desktop app uses the Rust core via `src-tauri/src/lib.rs`. They must behave identically.

Steps:
1. Use @explore to find the Rust implementation AND the frontend counterpart.
2. Change the Rust core first and add or update a test in `src/simulation-core/tests/`.
3. Mirror the exact same logic in `simulation.ts` (same formulas, thresholds, order of operations).
4. Keep the core deterministic: no wall-clock time, no unseeded randomness, no I/O.
5. Run @tester, then @reviewer.

Known thresholds (re-verify in code before relying on them):
- Track degradation below 25% confidence after cyber disruption
- Escalation: cyber +1, space strike +2 each