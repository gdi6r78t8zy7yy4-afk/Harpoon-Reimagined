#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

while IFS= read -r -d '' scenario; do
  cargo run --quiet --locked -p harpoon-scenario-format --bin scenario-check -- scenario "$scenario"
done < <(find "$repo_root/scenarios" -type f -name '*.json' -print0)

while IFS= read -r -d '' campaign; do
  cargo run --quiet --locked -p harpoon-scenario-format --bin scenario-check -- campaign "$campaign"
done < <(find "$repo_root/campaigns" -maxdepth 1 -type f -name '*.json' -print0)
