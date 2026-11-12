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

## M3 — Saves and campaign slice

- Add SQLite-backed settings and save/load for a documented save schema.
- Carry scenario outcomes into a short campaign progression.
- **Exit:** a campaign can be resumed after closing the application.

## M4 — Simulation depth and content tools

- Expand sensor, weapon, and domain behavior based on validated design priorities.
- Improve scenario authoring, validation, and 3D presentation.
- **Exit:** content can be authored and reviewed with repeatable workflows.

## Ongoing quality

Keep simulation behavior testable, preserve save compatibility intentionally, profile before optimizing, and document changes to foundational decisions.
