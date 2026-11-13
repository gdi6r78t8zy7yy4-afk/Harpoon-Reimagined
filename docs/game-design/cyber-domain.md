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

- Disruption of sensors or command systems — cyber effects currently suppress only defensive systems.
- Defensive cyber countermeasures, escalation rules, and campaign consequences. These remain design prerequisites before further cyber work (see the roadmap's future weapon domains).
