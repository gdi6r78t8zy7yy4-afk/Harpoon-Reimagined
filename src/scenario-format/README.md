# Scenario and campaign format

Rust crate for versioned scenario and campaign JSON, schema checks, and validation before a session starts. Version 1 supports units, uncorrelated sensor contacts, classify-contact objectives, campaign metadata, and ordered scenario IDs. Starter content lives under `scenarios/` and `campaigns/`.

## Validate content

Run `./scripts/validate-content.sh` from any directory to validate every scenario and campaign JSON file. To check one file, run:

```sh
cargo run --locked -p harpoon-scenario-format --bin scenario-check -- scenario scenarios/north-atlantic-intercept.json
cargo run --locked -p harpoon-scenario-format --bin scenario-check -- campaign campaigns/northern-watch.json
```
