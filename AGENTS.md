# Harpoon Reimagined — Agent Guide

This repo is a near-future maritime warfare simulation with Tauri desktop app, Rust simulation core, React/Vite frontend, and Babylon.js rendering.

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `apps/desktop/` | Tauri 2 desktop shell + React frontend (Vite) |
| `apps/desktop/src-tauri/` | Rust backend exposing simulation to Tauri via `invoke` |
| `src/simulation-core/` | Deterministic Rust simulation (units, sensors, weapons, time) |
| `src/scenario-format/` | JSON schema validation for scenarios/campaigns |
| `scenarios/` | Standalone scenario definitions (JSON) |
| `campaigns/` | Campaign definitions with ordered scenarios |
| `docs/` | Architecture, design decisions (ADR), and game design |

**Key Boundary:** Simulation is pure Rust, independent of UI/renderer. Frontend talks to it via Tauri commands.

## Development Commands

| Task | Command |
|------|---------|
| Build Rust workspace | `cargo check --workspace --locked` or `cargo test --workspace --locked` |
| Validate content | `./scripts/validate-content.sh` (scenarios, campaigns, and bundled copies) |
| Frontend dev | `npm --prefix apps/desktop run dev` (runs on port 1420) |
| Frontend build | `npm --prefix apps/desktop run build` (runs TypeScript + Vite) |
| Desktop dev | `npm --prefix apps/desktop run taur -- dev` |

**CI order:** Rust checks → Rust tests → content validation.

## Workflow & Conventions

## Workflow (always follow)
1. For any change touching more than 2 files, delegate to @explore first.
2. Make small, focused edits. Do not refactor code you were not asked to touch.
3. After changes, delegate to @tester. Fix failures before continuing.
4. Before finishing a larger task, delegate to @reviewer.
5. Changing simulation behavior? Load skill `sim-parity` first.
6. Adding or editing scenarios/campaigns? Load skill `content-authoring` first.

## Hard rules
- Simulation core (`src/simulation-core/`) must stay pure and deterministic:
  no UI, no wall-clock time, no unseeded randomness, no I/O.
- Frontend never reimplements simulation rules without mirroring them in Rust (and vice versa).
- Bundled JSON in `apps/desktop/src/data/` must be byte-identical to the authored copy.
- Save format `harpoon.campaign-save.v1`: schema changes are additive only.
- Never run destructive commands (rm -rf, git push --force, git reset --hard).
- Never commit or push unless explicitly asked.
- Never invent APIs, fields or file paths. Search the codebase first.
- Do not add dependencies without asking.
- If docs conflict with Rust or JSON, trust the executable source.

### Content Authoring
- Scenario and campaign JSON lives in `scenarios/` and `campaigns/`
- Desktop app bundles copies under `apps/desktop/src/data/` (must match exactly—validator checks)
- Validate before committing: run `./scripts/validate-content.sh`

### Simulation Behavior
- **Time:** Advance via `advance(seconds)`; supports pause/time acceleration (1× to 30×)
- **Units:** Move deterministically based on course/speed orders
- **Sensors:** Radar/sonar/ESM sweeps detect and track contacts; confidence decays on unrefreshed tracks
- **Weapons:** Launch → flight time → hit/miss resolution with defensive intercepts, cyber suppression
- **Campaign persistence:** SQLite on desktop, localStorage in browser

### Communication Flow
1. Frontend → Tauri command (`invoke`)
2. Tauri backend calls Rust simulation
3. Simulation advances, emits events
4. State serialized to SQLite or localStorage

### Data Constraints (validate with schema)
- Units: 1–16, unique IDs, side ∈ {friendly, hostile, neutral}, speed ≤50 kt
- Contacts: max 32, domain ∈ {surface, air, subsurface, space}, confidence ≤100%
- Weapons: domain must match; cyber/ASAT have special rules
- Space contacts: may include groundTrack, orbitalSensor (passes), hardening
- Coastlines: 0–12 landmasses, non-overlapping, units must start outside

## Common Tasks

### Adding a New Scenario
1. Create `scenarios/{id}.json` with valid schema (see `docs/game-design/scenario-authoring.md`)
2. Run validator: `./scripts/validate-content.sh`
3. Copy to `apps/desktop/src/data/` (byte-identical)
4. Add tests in `src/simulation-core/tests/`

### Debugging Simulation
- Frontend uses browser-side simulation when `__TAURI_INTERNALS__` is absent
- Backend uses Rust simulation via `src-tauri/src/lib.rs`
- Check event log in UI for engagement outcomes and system status

### Working with Campaign Saves
- Desktop: SQLite at app-specific location
- Browser: `localStorage` key `harpoon.campaign-save.v1`
- Saves include simulation state, campaign progress, settings (timeScale, layersVisible)
- Escalation is recomputed on load from scenario results

## Gotchas & Traps

| Issue | Why it happens | Fix |
|-------|----------------|-----|
| Campaign save incompatible after schema update | Old saves lack new fields | Save loader initializes defaults (see `normalizeBrowserContacts`, `recomputeEscalation`) |
| Unit doesn’t move | Check speed and course commands; space units use groundTrack | Frontend simulation matches backend logic (see `simulation.ts`) |
| Weapon misses unexpectedly | Hit chance includes confidence, range penalty, roll | Check engagement.log for hitChance and roll |
| Track classification lost | Cyber sensors disruption causes degradation → 25% confidence threshold | Event shows “track degraded” with source attribution |
| Bundle sync fails | `scripts/validate-content.sh` detects mismatch between authored and bundled JSON | Copy new version to `apps/desktop/src/data/` |

## Testing

- **Unit tests:** `cargo test --workspace --locked`
- **Frontend tests:** Not yet in CI (TBD)
- **Playthrough testing:** Native macOS app recommended for campaign save/load validation

## Notes on AI Agents

- **Do not modify `AGENTS.md` in `.opencode/agents/`**—those are subagent configs for this session, not the project’s instruction file
- **Simulation code is authoritative**—if docs conflict with Rust or JSON, trust the executable source
- **Escalation matters**—cyber operations (+1) and space strikes (+2 each) gate campaign consequences
- **Save format version** is `v1`; schema changes are additive only

## Links

- Architecture: `docs/architecture/overview.md`
- Scenario authoring: `docs/game-design/scenario-authoring.md`
- Cyber domain: `docs/game-design/cyber-domain.md`
- Space domain: `docs/game-design/space-domain.md`
- Roadmap: `docs/development/roadmap.md`
