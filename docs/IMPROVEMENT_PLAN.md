# MPS Improvement Plan

Date: 2026-07-09

## Completed Clean Rebuild

- Rebuilt `mps-core` around explicit validation, scoring, equipment rules,
  safety handling, and journey validation.
- Rebuilt `mps-cli` with strict format parsing and optional `--output`.
- Rebuilt SQLite seed data as clean ASCII with explicit movement systems and
  movement experience tags.
- Rebuilt `web/seed-data.js` and `web/mps-engine.js` as a clean browser-side
  port of the deterministic rules.
- Replaced stale/mojibake agent handoff docs.

## Current Quality Gates

- `cargo test -q` passes.
- `node --check web\seed-data.js`, `web\mps-engine.js`, and `web\app.js` pass.
- CLI smoke with a fresh SQLite database passes.
- Web jsdom smoke passes.

## Next Backlog

1. Generate `web/seed-data.js` from the same source as SQLite seed data.
2. Add CI for Rust tests and JS syntax checks.
3. Add markdown snapshot tests.
4. Add data validation tests for missing roles, objectives, cues, systems, and
   experience tags.
5. Expand exercise metadata beyond the MVP 24 exercises.
6. Add visual browser smoke tests before Vercel deploys.
7. Add production deploy automation after CI passes.
