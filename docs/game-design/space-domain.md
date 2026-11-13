# Space domain design notes

This document defines the space and counter-space rules required by the roadmap before any implementation begins. It is the design source for milestone M11; behavior that contradicts these notes should update this document first.

## Design intent

Space adds one decision to the game: **who can see the battlespace**. Orbital sensing is a powerful but intermittent information source, and attacking it is an escalatory act with consequences. The domain must stay command-focused — no orbital mechanics homework.

## Domain model

- `space` becomes a fourth value in every existing domain list (unit, contact, sensor, weapon target domains).
- Space contacts are satellite **ground tracks** projected onto the tactical plane. They carry an optional motion vector (heading and ground-track speed) so they cross the area of operations; all position, bearing, range, and uncertainty math stays two-dimensional (see ADR 0002).
- The renderer may draw space markers at elevated height for visual separation. Height is presentation only; the simulation remains planar.

## Sensing from orbit

- Satellite sensors use a new sensor kind `space-based` with target domains limited to `surface` and `air`. Subsurface contacts cannot be sensed from orbit, preserving the existing domain asymmetry.
- A satellite reports only during scenario-defined **passes**: explicit windows of `{ startSeconds, durationSeconds }` in scenario data. Sweeps outside a pass window are rejected. Windows make orbital value intermittent without simulating orbits.
- Reports feed the same track fusion and uncertainty pipeline as other sensors; no special-case track rules.

## Counter-space engagement

- New weapon kind `asat`. It may only target `space` domain contacts, follows existing launch legality (identified hostile contact, in estimated range, ammunition available), and reuses the in-flight → impact lifecycle.
- A successful ASAT effect **disables** the satellite's sensing for a bounded duration, mirroring the cyber disruption pattern: stacking extends the window from the later of current end or new impact; recovery emits an attributed restore event. Hull damage is zero by validation rule, as with cyber weapons.
- Destruction is deliberately not modeled in M11: disabling keeps outcomes reversible and avoids debris consequences this milestone cannot yet represent.

## Countermeasures (required before implementation)

- Satellites defend with the existing defense-system model (charges, intercept chance). An intercepted ASAT effect consumes a charge and resolves as `intercepted`.
- **Hardening:** after surviving an attack, a satellite is immune to further effects for a scenario-defined cooldown. This prevents blind-spamming and creates a timing decision.

## Escalation and campaign consequences (required before implementation)

- The after-action report records counterspace actions alongside existing outcome counts: `spaceStrikes` launched and `satellitesDisabled`.
- Campaign progression stores an integer `escalationLevel`, increased by cyber and counterspace first use against a faction, with diminishing repeats. Scenario data may gate weapon availability or allied sensor support at escalation thresholds; the campaign slice only needs one authored branch to prove the mechanism.

## Validation limits

- Ground-track speeds bounded separately from weapon speed limits (orbital motion is far faster than any tactical vehicle); pass windows must be non-overlapping per satellite and inside scenario time bounds.
- Additive schema changes only: new enum values plus optional fields with defaults, so existing scenarios remain valid without a `schemaVersion` bump, following the established save-compatibility pattern.

## Out of scope for M11

Realistic orbital mechanics or propagation, constellation management, persistent global campaign map, kinetic kill vehicles and debris modeling, and space-based offensive weapons against maritime targets.
