# Development roadmap

Milestones are ordered to reduce technical risk. Scope is provisional and should be revised as prototypes reveal constraints.

## M0 — Project foundation

- Establish repository structure, product direction, architecture boundaries, and contribution workflow.
- **Exit:** project decisions and next milestone are documented.

## M1 — Desktop and simulation foundation (complete)

- [x] Create the Tauri 2 + React/TypeScript/Vite desktop application.
- [x] Create the Rust workspace and simulation-core crate.
- [x] Display a Babylon.js scene and connect a minimal command/event loop.
- [x] Add CI checks for the frontend build and Rust workspace.
- [x] Build and launch the native app on Apple Silicon macOS.
- **Exit:** a native app opens a simple 3D scene and displays state produced by Rust.

## M2 — First playable scenario (complete)

- [x] Define versioned JSON scenario data with Rust validation.
- [x] Load the North Atlantic starter scenario into the Rust simulation.
- [x] Add unit movement, course and speed orders, contact classification, pause, and time acceleration.
- [x] Track the first objective: classify two contacts, stop the simulation on completion, and offer replay.
- [x] Complete a native macOS play-through and refine scenario pacing and feedback.
- **Exit:** a complete short scenario can be loaded, played, and completed.

The native play-through confirmed pacing but surfaced presentation gaps: the tactical view has no authored coastlines or terrain, and camera panning is not discoverable. These are recorded under M4.

## M3 — Saves and campaign slice (complete)

- [x] Add versioned SQLite-backed campaign save/load for the active simulation and interface settings; restores start paused.
- [x] Define a versioned campaign with ordered scenarios and retain completed scenario progress.
- [x] Confirm native save/load across an application restart and refine the campaign flow.
- **Exit:** a campaign can be resumed after closing the application.

## M4 — Simulation depth and content tools (in progress)

- [x] Add data-driven unit sensor profiles and deterministic sensor sweeps that refresh in-range contact tracks.
- [x] Add a repeatable CLI workflow for validating scenario and campaign content.
- [x] Render authored coastlines for land/water separation in the tactical chart and make camera pan controls discoverable.
- [x] Document scenario authoring rules, validation limits, and the repeatable content workflow in [authoring notes](../game-design/scenario-authoring.md).
- Expand sensor, weapon, and domain behavior based on validated design priorities.
- **Exit:** content can be authored and reviewed with repeatable workflows.

## M5 — Situational awareness (complete)

- [x] Keep sensor reports in persistent, reviewable contact tracks.
- [x] Decay confidence on unrefreshed tracks and mark stale tracks in the interface.
- [x] Preserve older saves by initializing missing track metadata when loaded.
- **Exit:** sensor reports form persistent, reviewable tracks with visible uncertainty.

Follow-up work can improve multi-sensor report fusion and provide richer uncertainty estimates and contact management.

## M6 — Multi-sensor track fusion (complete)

- [x] Combine bearing reports with circular, confidence-weighted fusion.
- [x] Fuse radar range estimates and retain an uncertainty estimate for range and bearing.
- [x] Grow track uncertainty while reports age and display the estimate bounds and report count.
- [x] Keep a bounded source history for sensor reports on each track.
- [x] Let operators correlate and separate tracks, preserving the relationship in saves.
- **Exit:** reports from different sensor types improve the same saved track and expose how precise its position estimate is.

## M7 — Weapon employment (complete)

- [x] Define scenario-loaded weapon stores with domain, range, ammunition, and base hit chance.
- [x] Require an identified hostile contact and validate estimated range and target domain before firing.
- [x] Consume ammunition, resolve a repeatable hit/miss outcome, and persist engagement records.
- [x] Model time to impact, resolve hits and misses during simulation advance, and show in-flight engagements.
- [x] Add defensive responses that intercept incoming weapons and consume limited charges.
- [x] Apply graduated damage until a target is disabled or destroyed.
- **Exit:** engagements progress from a legal launch through impact and produce a persistent, reviewable outcome.

## M8 — 3D combat visualization (complete)

- [x] Render in-flight weapon tracks and moving projectile markers in the tactical scene.
- [x] Show distinct visual feedback for hits, damage, interceptions, misses, and aborted engagements.
- [x] Rebuild engagement visuals from saved simulation state and keep in-flight positions synchronized with simulation time.
- **Exit:** players can follow each active engagement from launch through its resolved outcome in the tactical view.

## M9 — After-action review and campaign records (complete)

- [x] Record a scenario result with classifications, target losses, and weapon outcomes.
- [x] Capture completion whether it occurs during an order or after pending engagements resolve.
- [x] Show the saved result as an after-action summary and keep older saves readable.
- **Exit:** each completed scenario has a reviewable result that remains part of its campaign save.

## M10 — Cyber defense suppression (complete)

- [x] Define scenario-loaded cyber weapons with a bounded disruption duration and no hull damage.
- [x] Suppress a target's defensive systems when a cyber effect lands, letting follow-on kinetic weapons bypass them while the window holds.
- [x] Recover defenses after the disruption window expires and attribute the effect to its source in events and saves.
- [x] Show distinct in-flight and outcome visuals for cyber effects, suppression status on the target panel, and cyber outcomes in the after-action report.
- **Exit:** a cyber launch visibly suppresses a hostile's defensive systems for a bounded time and restores them afterward.

## M11 — Space domain and counter-space (complete)

- [x] Define the space domain model, engagement rules, countermeasures, escalation, and campaign consequences in [design notes](../game-design/space-domain.md) and [ADR 0002](../adr/0002-space-domain-model.md).
- [x] Add `space` as a contact and weapon target domain with moving ground-track contacts validated against orbital motion bounds.
- [x] Add scenario-defined satellite passes that refresh tracks through the existing sensor fusion pipeline on deterministic revisit windows.
- [x] Add anti-satellite weapons that disable a satellite's sensing for a bounded window, defended by the existing interception model and post-attack hardening cooldowns.
- [x] Record counterspace actions in the after-action report and expose at least one escalation-gated campaign consequence.
- **Exit:** a scenario can be played where blinding or restoring orbital sensing changes the tactical picture and is remembered by the campaign save.

## Future weapon domains (planned)

- Extend cyber weapons beyond defense suppression to sensor and command-system disruption, with clear duration and recovery rules and player-visible attribution. Define these rules before implementation.

## Ongoing quality

Keep simulation behavior testable, preserve save compatibility intentionally, profile before optimizing, and document changes to foundational decisions.
