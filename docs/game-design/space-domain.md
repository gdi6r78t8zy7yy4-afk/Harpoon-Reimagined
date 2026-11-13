# Space domain design notes

This document defines the space and counter-space rules required by the roadmap before any implementation begins. It is the design source for milestone M11; behavior that contradicts these notes should update this document first.

## Design intent

Space adds one decision to the game: **who can see the battlespace**. Orbital sensing is a powerful but intermittent information source, and attacking it is an escalatory act with consequences. The domain must stay command-focused — no orbital mechanics homework.

## Domain model

- `space` becomes a fourth value in every existing domain list (unit, contact, sensor, weapon target domains).
- Space contacts are satellite **ground tracks** projected onto the tactical plane. They carry an optional motion vector (heading and ground-track speed) so they cross the area of operations; all position, bearing, range, and uncertainty math stays two-dimensional (see ADR 0002). The drift represents averaged sub-satellite motion over the scenario horizon, not instantaneous orbital velocity — true revisit geometry is out of scope.
- The renderer may draw space markers at elevated height for visual separation. Height is presentation only; the simulation remains planar.

## Sensing from orbit

- Satellite sensors use a new sensor kind `space-based` with target domains limited to `surface` and `air`. Subsurface contacts cannot be sensed from orbit, preserving the existing domain asymmetry.
- Satellites are contacts, not commandable units, so no operator sweeps them by hand. Instead each satellite carries scenario-defined **passes**: explicit windows of `{ startSeconds, durationSeconds }` in scenario data. At the moment simulation time crosses a pass window's start, the satellite automatically generates one report batch — refreshing every non-identified contact of an eligible domain within sensor range — and no reports are produced outside pass windows. Windows make orbital value intermittent without simulating orbits.
- A disabled satellite (see counter-space) misses its pass: the report batch is skipped and a single event records the missed window, so ASAT timing decisions have visible feedback.
- Reports feed the same track fusion and uncertainty pipeline as other sensors; no special-case track rules. Space-based reports are bearing-only with moderate accuracy, like ESM.

## Counter-space engagement

- New weapon kind `asat`. It may only target `space` domain contacts, follows existing launch legality (identified hostile contact, in estimated range, ammunition available), and reuses the in-flight → impact lifecycle.
- A successful ASAT effect **disables** the satellite's sensing for a bounded duration, mirroring the cyber disruption pattern: stacking extends the window from the later of current end or new impact; recovery emits an attributed restore event. Hull damage is zero by validation rule, as with cyber weapons.
- Destruction is deliberately not modeled in M11: disabling keeps outcomes reversible and avoids debris consequences this milestone cannot yet represent.

## Countermeasures (required before implementation)

- Satellites defend with the existing defense-system model (charges, intercept chance). An intercepted ASAT effect consumes a charge and resolves as `intercepted`.
- **Hardening:** after surviving an attack, a satellite is immune to further effects for a scenario-defined cooldown. This prevents blind-spamming and creates a timing decision.

## Escalation and campaign consequences (required before implementation)

- The after-action report records counterspace actions alongside existing outcome counts: `spaceStrikes` launched and `satellitesDisabled`.
- Campaign progression stores an integer `escalationLevel`, recomputed from recorded scenario results: each completed scenario contributes 1 for any cyber disruption and 2 for any satellite disabled — first use only, so repeat strikes within a scenario do not inflate it further.
- The campaign definition gains an optional `consequences` list of `{ id, minEscalation, title, description }` entries. Entries whose threshold is met are surfaced in the campaign panel as unlocked consequences; the starter campaign authors one branch (escalated allied ISR posture) to prove the mechanism.

## Validation limits

- Ground-track drift speed is bounded separately from weapon and vehicle speed limits (1–600 kt) because it represents averaged sub-satellite motion, not orbital velocity. Heading stays in `[0, 360)`.
- Pass windows: at most 8 per satellite, each 60–3600 seconds long, sorted and non-overlapping, ending within a 24-hour scenario horizon. Orbital sensor range stays within the existing `(0, 1000]` NM bound and its domains are limited to `surface` and `air`.
- ASAT weapons follow the cyber pattern: zero hull damage, mandatory `effectDurationSeconds` between 60 and 7200 seconds, and target domains exactly `["space"]`. No other weapon kind may list `space`, and no unit sensor may sense it. Ground-track motion, orbital sensors, and hardening cooldowns are only valid on `space` domain contacts; hardening defaults to 600 seconds and is capped at 3600.
- Additive schema changes only: new enum values plus optional fields with defaults, so existing scenarios remain valid without a `schemaVersion` bump, following the established save-compatibility pattern.

## Out of scope for M11

Realistic orbital mechanics or propagation, constellation management, persistent global campaign map, kinetic kill vehicles and debris modeling, and space-based offensive weapons against maritime targets.
