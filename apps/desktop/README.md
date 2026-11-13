# Desktop application

The first interactive prototype uses Tauri 2, React, TypeScript, Vite, and Babylon.js. The frontend can also run in a browser with an in-memory simulation fallback.

## Requirements

- Node.js 20 or newer
- Rust stable
- For native macOS builds, Xcode Command Line Tools and the macOS WebKit framework

## Run the frontend

```sh
cd apps/desktop
npm install
npm run dev
```

## Run the native app

```sh
cd apps/desktop
npm install
npm run tauri dev
```

## Current slice

The prototype loads a versioned North Atlantic scenario, displays a procedural 3D tactical map, and supports selection, course/speed orders, sensor sweeps, contact classification, objective progress, pause/resume, time scaling, and one-minute stepping. Sensor reports fuse into persistent tracks with source histories, uncertainty estimates, and operator correlation controls. Scenario-defined weapon stores can engage identified hostile contacts after range and domain checks; ammunition is consumed and engagement outcomes resolve after weapon flight time. Rust validates scenario data and owns simulation state. Campaign saves include progress, active simulation state, sensors, weapon stores, engagement records, time scale, and map-layer visibility; desktop saves are stored in SQLite and browser previews use localStorage. Native save/load across a full application restart still needs a manual confirmation. Defensive responses and weapon interception are not implemented yet.
