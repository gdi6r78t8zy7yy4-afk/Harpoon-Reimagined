# Scenario authoring guide

How to create and review scenario and campaign content with repeatable checks. The authoritative rules below mirror the validation in `src/scenario-format/src/lib.rs`; if they ever disagree, the code wins and this page should be corrected.

## Files and workflow

- Authored scenarios live in `scenarios/*.json`, campaigns in `campaigns/*.json`.
- The desktop app bundles copies of these files under `apps/desktop/src/data/`. Keep them byte-identical to their authored sources; the validator now fails if a bundled copy drifts.
- Iterate with:

```sh
./scripts/validate-content.sh   # validates every scenario and campaign, then checks bundle sync
npm --prefix apps/desktop run dev  # preview in a browser against the same data
cargo test --workspace          # regression tests over the starter content
```

## Compatibility policy

Schema changes are additive: new fields get serde defaults so older scenarios and saves keep loading without a `schemaVersion` bump. Bump `schemaVersion` only for breaking rewrites, which are not planned.

## Scenario schema limits

| Field | Rule |
| --- | --- |
| `schemaVersion` | Must be `1`. |
| `id`, `title`, `theatre` | Non-empty. |
| `units` | 1–16 entries; ids unique across units and contacts. |
| `contacts` | At most 32. |
| `objectives` | 1–8; only kind `classify_contacts`; `required` between 1 and the contact count. |
| `coastlines` | Optional, at most 12 (see below). |

**Units.** `side` is `friendly`, `hostile`, or `neutral`. Heading in `[0, 360)`, speed ≤ 50 kt, health 1–100. Positions are finite and within ±50 map units.

**Sensors.** Ids unique per unit; kind one of `radar`, `sonar`, `esm`; range in `(0, 1000]` NM; domains a non-empty subset of `surface`, `air`, `subsurface`.

**Weapons.** Ids unique per unit; kind one of `missile`, `torpedo`, `gun`, `cyber`, `asat`; range in `(0, 1000]` NM; ammunition ≥ 1; hit chance 1–100%; speed 1–5000 kt. Kinetic weapons deal damage (`damagePercent` 1–100, default 100) and must not carry `effectDurationSeconds`. Cyber weapons deal zero hull damage and require `effectDurationSeconds` between 60 and 7200 seconds (see [cyber domain notes](cyber-domain.md)). ASAT weapons follow the same effect rules but their target domains are exactly `[space]`; no other weapon kind may list `space` (see [space domain notes](space-domain.md)).

**Contacts.** Confidence ≤ 100; bearing ≤ 359; range > 0 NM; domain one of `surface`, `air`, `subsurface`, `space`. Defensive systems: ids unique per contact, charges 1–20, intercept chance 1–100%. Space contacts may carry a `groundTrack` (heading in `[0, 360)`, drift speed 1–600 kt — averaged sub-satellite motion, not orbital velocity), an `orbitalSensor` (range in `(0, 1000]` NM, domains a subset of `surface`/`air`, and 1–8 pass windows each 60–3600 seconds long, sorted, non-overlapping, ending within 24 hours), and `hardeningSeconds` (default 600, max 3600). These three fields are only valid on space contacts.

**Coastlines.** Each landmass has a non-empty unique `id`, a non-empty `name`, and a closed polygon of 3–64 points (the last point connects back to the first). Points use the same coordinate space as unit positions — finite, within ±50 map units. No unit or contact may start inside a coastline polygon; the validator rejects landlocked starts. Coastlines are presentation data: they render in the tactical chart and gate starting positions, but do not yet block movement, sensing, or weapons.

## Campaign schema limits

`schemaVersion` must be `1`; campaign id, title, and description non-empty; 1–32 scenario ids, unique and non-empty. Scenario files are referenced by their `id`. An optional `consequences` list (at most 8) records escalation-gated outcomes: each entry has a unique non-empty `id`, `title`, and `description`, plus `minEscalation` ≥ 1. The campaign panel surfaces entries whose threshold the recorded escalation level has reached.
