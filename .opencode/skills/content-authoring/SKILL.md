---
name: content-authoring
description: Step-by-step for adding or editing scenario and campaign JSON, including bundle sync and validation. Load when touching scenarios/, campaigns/ or apps/desktop/src/data/.
---
1. Read `docs/game-design/scenario-authoring.md` for the schema.
2. Create or edit `scenarios/<id>.json` (or `campaigns/<id>.json`).
3. Copy the file byte-identical to `apps/desktop/src/data/` (use `cp`, never retype).
4. Run `./scripts/validate-content.sh` and fix every error.
5. Add or update tests in `src/simulation-core/tests/`.

Constraints:
- Units 1-16, unique IDs, side in {friendly, hostile, neutral}, speed <= 50 kt
- Contacts max 32, confidence <= 100%
- Coastlines 0-12 landmasses, non-overlapping, units start outside