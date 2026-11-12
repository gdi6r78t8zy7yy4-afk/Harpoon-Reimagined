# Architecture overview

## System boundaries

```text
React interface ── commands/events ── Rust simulation core
       │                                      │
 Babylon.js renderer                       scenario data
       │                                      │
       └──────────── Tauri desktop shell ─ SQLite save data
```

The diagram describes intended boundaries, not implemented integrations.

### Desktop application

Tauri 2 hosts the desktop application. React and TypeScript own menus, command panels, and HUD. Babylon.js renders the 3D world from simulation snapshots. The renderer may interpolate between snapshots but must not decide combat outcomes or authoritative entity state.

### Simulation core

Rust owns world state, entity behavior, sensors, weapons, and time advancement. Keep this crate usable without a window or graphics engine so it can be tested and replayed independently. Prefer deterministic updates from explicit inputs and a controlled random seed.

### Scenario and campaign data

Scenario definitions use versioned JSON, validated before a session starts. Campaign progression refers to scenarios and persists outcomes and player choices. SQLite is the initial local store for saves and settings; it is not the source of truth for an active simulation tick.

## Data flow

1. The interface sends a player command to the desktop backend.
2. The backend validates the command and applies it through the simulation core.
3. The core advances according to the selected simulation clock and emits events/snapshots.
4. The interface presents events and updates the Babylon.js scene.
5. Save operations serialize an explicit save model to local storage.

Transport shapes, tick rates, coordinate systems, and save compatibility rules remain to be specified during implementation.
