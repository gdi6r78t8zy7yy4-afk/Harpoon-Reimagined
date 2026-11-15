# Cyber domain design notes

These rules describe the cyber weapon slice delivered in M10. They record implemented behavior so later work (sensor and command disruption, countermeasures, escalation) extends a defined baseline instead of re-litigating it.

## Implemented rules

- **Weapon definition:** a scenario weapon with `kind: "cyber"` carries an `effectDurationSeconds` between 60 and 7200 seconds and must have zero hull damage. Kinetic weapons keep the existing rules and must not carry an effect duration.
- **Launch legality:** cyber launches follow the same validation as kinetic weapons — identified hostile contact, matching target domain, estimated range within weapon range, ammunition available.
- **Impact outcome:** a successful cyber effect sets the engagement status to `disrupted`, leaves hull integrity untouched, and opens a disruption window on the target. A failed roll resolves as an ordinary miss; a destroyed or departed target aborts the effect like any other weapon.
- **Defense suppression:** while a target's disruption window is open at impact time, its defensive systems cannot intercept incoming weapons and consume no charges. This applies to kinetic weapons only; cyber effects do not stack through defenses.
- **Window stacking:** a new successful cyber effect starts from the later of the current window end or the new impact time, so overlapping effects extend rather than shorten suppression.
- **Recovery:** when simulation time passes the window end, suppression clears and a restore event is emitted, attributed to the weapon that caused it (falling back to `CYBER DEFENSE` for saves without attribution). Restored state persists in campaign saves; older saves load with no active disruption.

## Explicitly out of scope for M10

- Disruption of sensors or command systems — cyber effects currently suppress only defensive systems. (Delivered in M12 below.)
- Defensive cyber countermeasures, escalation rules, and campaign consequences. Escalation and consequences shipped with M11; defensive countermeasures remain open.

# M12 — Sensor and command disruption

M12 extends the cyber weapon family from defense suppression to two additional effect
targets. The full tactical value of blinding an adversary's sensors is realized when hostiles
act on their own tracks (enemy attacks are not yet simulated); these rules establish the
state model, recovery semantics, and player-visible attribution that future threat behavior
will build on, while giving de-escalation tools real consequences today.

## Weapon schema extension

- **Effect target:** a scenario weapon may carry `effectTarget` of `"defenses"`, `"sensors"`, or `"command"`. The field is optional and defaults to `"defenses"`, so every pre-M12 cyber weapon keeps its current behavior without editing content.
- **Legality:** only `kind: "cyber"` weapons may declare an effect target; kinetic, torpedo, gun, and ASAT weapons must not carry the field (ASAT keeps its implicit disable-and-harden semantics). Duration bounds are unchanged: 60–7200 seconds with zero hull damage.

## Contact state model

Each of the three cyber effects owns a separate window on the target contact, so a
contact can be defense-suppressed and sensor-disrupted at the same time:

- `defensesDisruptedUntilSeconds` (existing) — unchanged.
- `cyberSensorsDisruptedUntilSeconds` + source attribution (new).
- `cyberCommandDisruptedUntilSeconds` + source attribution (new).

All new fields are additive with serde defaults; older saves load with no active windows,
and expired windows clear on load like existing disruption state.

## Impact outcomes and engagement status

A successful cyber roll opens the window for its effect target using the M10 stacking rule:
the window starts at the later of the current end for that system or the impact time, then
extends by the weapon's `effectDurationSeconds`. Engagement status records which system was
affected: existing `"disrupted"` (defenses), new `"sensors_disrupted"`, and new
`"command_disrupted"`. Impact events read e.g. "…blinded {contact}'s sensor systems for N min"
or "…severed command control of {contact} for N min", attributed to `CYBER OPERATIONS`.

## Sensor disruption rules

While a contact's sensor-disruption window is open:

- **Identified tracks degrade.** Confidence decays at triple the normal unrefreshed rate and range/bearing uncertainty grows at triple the normal rate. This is the visible consequence of blinding a target: its emissions discipline collapses, our ESM-derived picture rots, and the track visibly ages on the panel and chart even though it was classified.
- **Classification loss.** If degraded confidence falls below 25, the contact reverts to unidentified with classification `Unknown` and an event records the loss. The contact can be re-acquired through normal sweeps or orbital passes, but cannot be engaged until reclassified — blinding a target you intend to strike costs you the firing solution.
- **Space-based sensing is unaffected.** Orbital SAR passes refresh tracks regardless of the target's own systems; only the target's emissions discipline degrades.

## Command disruption rules

While a contact's command-disruption window is open, its authored ground-track motion halts:
the contact holds station at its current position (speed treated as zero). Helm and comms
loss stops maneuvering but not sensing or defenses. When the window closes, normal motion
resumes from the held position.

## Moving contacts (schema relaxation)

Ground-track motion was previously restricted to space contacts. M12 allows an optional
`groundTrack` on any contact domain so command disruption has something to freeze and the
tactical picture can include underway hostiles:

- Heading 0–360, speed at least 1 knot; surface and subsurface capped at 50 knots (matching friendly order limits), air capped at 600 knots. Space keeps its existing 600-knot bound.
- Motion uses the existing ground-track integration for all domains, including map-edge wrap so a contact that leaves the chart re-enters on the opposite side.

## Escalation and campaign accounting

All successful cyber effects count toward `cyberDisruptions` regardless of effect target, so
the escalation formula is unchanged: any cyber use adds one level per scenario. No new
result counters are introduced in M12.

## Recovery and attribution

When simulation time passes a window end, that system's state clears and a restore event is
emitted attributed to the weapon that caused it ("Sensor systems restored on {label}",
"Command control restored on {label}"), falling back to `CYBER OPERATIONS` for saves without
attribution. Restored windows persist in campaign saves; expired windows never load as active.

## Explicitly out of scope for M12

- Enemy weapons or enemy sensing of player task groups — sensor disruption's defensive value awaits that threat model.
- Defensive cyber countermeasures and hardening against cyber attack (space hardening does not extend to cyber in this slice).
- Partial degradation models (e.g., halved intercept chance); each effect is binary for its window.
