# Development roadmap

Milestones are ordered to reduce technical risk. Scope is provisional and should be revised as prototypes reveal constraints.

## M0 — Project foundation

- Establish repository structure, product direction, architecture boundaries, and contribution workflow.
- **Exit:** project decisions and next milestone are documented.

## M1 — Desktop and simulation foundation (in progress)

- [x] Create the Tauri 2 + React/TypeScript/Vite desktop application.
- [x] Create the Rust workspace and simulation-core crate.
- [x] Display a Babylon.js scene and connect a minimal command/event loop.
- [x] Add CI checks for the frontend build and Rust workspace.
- [ ] Build and launch the native app on Apple Silicon macOS.
- **Exit:** a native app opens a simple 3D scene and displays state produced by Rust.

## M2 — First playable scenario (in progress)

- [x] Define versioned JSON scenario data with Rust validation.
- [x] Load the North Atlantic starter scenario into the Rust simulation.
- [x] Add unit movement, course and speed orders, contact classification, pause, and time acceleration.
- [x] Track the first objective: classify two contacts, stop the simulation on completion, and offer replay.
- [ ] Complete a native macOS play-through and refine scenario pacing and feedback.
- **Exit:** a complete short scenario can be loaded, played, and completed.

## M3 — Saves and campaign slice (in progress)

- [x] Add versioned SQLite-backed campaign save/load for the active simulation and interface settings; restores start paused.
- [x] Define a versioned campaign with ordered scenarios and retain completed scenario progress.
- [ ] Confirm native save/load across an application restart and refine the campaign flow.
- **Exit:** a campaign can be resumed after closing the application.

## M4 — Simulation depth and content tools

- [x] Add data-driven unit sensor profiles and deterministic sensor sweeps that refresh in-range contact tracks.
- [x] Add a repeatable CLI workflow for validating scenario and campaign content.
- Expand sensor, weapon, and domain behavior based on validated design priorities.
- Improve scenario authoring, validation, and 3D presentation.
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

## M8 — 3D combat visualization (in progress)

- [x] Render in-flight weapon tracks and moving projectile markers in the tactical scene.
- [ ] Show clear impact, interception, and damage feedback in the 3D view.
- [ ] Keep engagement visuals synchronized with simulation time and saved sessions.
- **Exit:** players can follow each active engagement from launch through its resolved outcome in the tactical view.

## Ongoing quality

Keep simulation behavior testable, preserve save compatibility intentionally, profile before optimizing, and document changes to foundational decisions.
