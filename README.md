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

The first interactive desktop prototype loads a validated scenario, renders a Babylon.js tactical scene, and connects player orders to the Rust simulation. The app is not yet a complete game.

## Development principles

1. Keep simulation rules independent of React, Tauri, and Babylon.js.
2. Make simulation steps reproducible from explicit state and inputs.
3. Keep presentation and simulation time separate; support pause and time acceleration.
4. Validate and version external scenario and campaign data.
5. Prefer small, reviewable changes with documented design decisions.

## Validate scenario and campaign content

Run `./scripts/validate-content.sh` to validate every JSON scenario and campaign before committing content changes. CI runs the same validation alongside the Rust checks.

## Status

The repository contains the project foundation and an early interactive prototype with one playable objective. Broader combat systems and multiple authored scenarios remain future work. The first campaign slice now tracks progression and local save/load, verified across an application restart in the native macOS build, and cyber weapons can suppress a hostile's defensive systems for a bounded window.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before proposing changes. Architecture decisions are recorded in [docs/adr](docs/adr/).
