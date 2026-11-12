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

The prototype displays a procedural 3D tactical map, a small friendly force and uncorrelated contacts, plus selection, simulation pause/resume, time scaling, and one-minute stepping. Rust owns the simulation snapshot and movement; browser preview uses a matching local mock. Orders, sensor behavior, campaign persistence, and authored scenarios remain future work.
