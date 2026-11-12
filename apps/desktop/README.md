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

The prototype loads a versioned North Atlantic scenario, displays a procedural 3D tactical map and uncorrelated contacts, and supports selection, course/speed orders, contact classification, objective progress, a mission completion state with replay, pause/resume, time scaling, and one-minute stepping. Rust validates scenario data and owns simulation state; browser preview uses matching local scenario data. Campaign persistence, richer sensor behavior, weapons, and additional scenarios remain future work.
