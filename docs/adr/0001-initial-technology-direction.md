# 0001: Initial technology direction

- **Status:** Accepted for the project foundation
- **Date:** 2026-11-12

## Context

The project needs a native desktop shell, a rich 3D presentation, and a simulation model that remains independent of rendering and interface code.

## Decision

Start with Tauri 2, React, TypeScript, and Vite for the desktop client; Babylon.js for 3D presentation; Rust for simulation; versioned JSON for scenario definitions; and SQLite for initial local persistence.

## Consequences

The frontend and simulation communicate through explicit commands and snapshots/events. The initial target is macOS on Apple Silicon. The chosen stack is a direction to validate in the first implementation milestone, not evidence that the app is already built.
