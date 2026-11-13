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

while IFS= read -r -d '' bundled; do
  name="$(basename "$bundled")"
  for source_dir in scenarios campaigns; do
    source_file="$repo_root/$source_dir/$name"
    if [ -f "$source_file" ]; then
      cmp -s "$source_file" "$bundled" || {
        echo "Bundled frontend copy out of sync: apps/desktop/src/data/$name differs from $source_dir/$name" >&2
        exit 1
      }
      break
    fi
  done
done < <(find "$repo_root/apps/desktop/src/data" -type f -name '*.json' -print0)

echo "Bundled frontend data copies match their authored sources."
