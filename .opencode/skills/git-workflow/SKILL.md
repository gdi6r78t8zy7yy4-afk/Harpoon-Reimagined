---
name: git-workflow
description: Commit message format, branch naming and PR rules. Load when asked to commit, branch or prepare a PR.
---
- Branches: `feat/<short-name>`, `fix/<short-name>`
- Commits: Conventional Commits (`feat: ...`, `fix: ...`, `chore: ...`)
- Run @tester (all steps) before every commit.
- Never commit to main directly. Never push unless asked.