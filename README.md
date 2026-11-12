# Harpoon Reimagined

A modern, 3D maritime warfare simulation focused on clear command decisions, credible sensor and weapon behavior, and replayable scenarios alongside longer campaigns.

## Project direction

- **Setting:** Near future
- **Game modes:** Standalone scenarios and persistent campaigns
- **Presentation:** Full 3D, with a command-focused interface
- **Desktop target:** macOS on Apple Silicon first
- **Desktop shell:** Tauri 2
- **Client:** React, TypeScript, Vite, and Babylon.js
- **Simulation:** Rust, independent of the UI and renderer
- **Initial persistence:** SQLite
- **Scenario data:** Versioned JSON

These are project decisions, not a promise that all systems or platforms are already implemented. See [the architecture](docs/architecture/overview.md), [game design](docs/game-design/vision.md), and [roadmap](docs/development/roadmap.md).

## Repository layout

```text
apps/desktop/       Desktop client and native Tauri shell
src/simulation-core/ Deterministic simulation domain and rules
src/scenario-format/ Scenario data model, validation, and versioning
docs/               Architecture, design decisions, and development plan
scenarios/          Standalone scenario content
campaigns/          Campaign definitions and progression data
assets/             Models, materials, audio, and interface resources
tools/               Content and developer utilities
scripts/             Repeatable repository maintenance scripts
.github/             Contribution templates and CI workflows
```

A first interactive desktop prototype is in place. The frontend has a procedural Babylon.js tactical scene and controls; the Rust core owns simulation state. The app is not yet a complete game.

## Development principles

1. Keep simulation rules independent of React, Tauri, and Babylon.js.
2. Make simulation steps reproducible from explicit state and inputs.
3. Keep presentation and simulation time separate; support pause and time acceleration.
4. Validate and version external scenario and campaign data.
5. Prefer small, reviewable changes with documented design decisions.

## Status

The repository contains the project foundation and an early interactive desktop prototype. Core game systems, authored scenarios, campaigns, and persistence are not implemented yet.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before proposing changes. Architecture decisions are recorded in [docs/adr](docs/adr/).
