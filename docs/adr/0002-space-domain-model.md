# 0002: Space domain as projected ground tracks

- **Status:** Accepted as the design basis for milestone M11
- **Date:** 2026-11-13

## Context

The roadmap requires a space domain with orbital sensing and counter-space weapons. The simulation core is deterministic, two-dimensional (x/z plane), and advances by clamped time steps; contacts carry bearing/range estimates with uncertainty, and all weapon legality uses planar range. Full orbital fidelity would conflict with determinism, the existing uncertainty model, and the command-focused design pillars.

## Decision

Represent satellites as ground tracks projected onto the tactical plane: space contacts are ordinary moving points in the existing 2D world with an optional heading and ground-track speed. Orbital availability is expressed through scenario-defined pass windows rather than propagated orbits. Rendering may offset space markers vertically for readability; height carries no simulation meaning. See [space domain design notes](../game-design/space-domain.md).

## Alternatives considered

- **True orbital propagation:** physically credible but adds a third dimension, non-deterministic-friendly math, and complexity that does not create player decisions.
- **Pass events without entities:** satellites modeled only as scheduled report sources with no targetable contact; simpler, but counter-space weapons need a targetable track, so this cannot carry the full milestone.

## Consequences

Space reuses all existing position, bearing/range uncertainty, fusion, weapon legality, and engagement lifecycle code unchanged. Scenario validation needs separate speed bounds for ground-track motion. The renderer gains one presentation-only height convention. If future milestones need altitude-aware effects (e.g., space-based attack on maritime targets), this model can carry them as range modifiers without reworking world state.
