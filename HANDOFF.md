# MPS Clean Rebuild Handoff

Date: 2026-07-09

## What Changed

This session rebuilt the project around a clean deterministic MVP instead of
continuing to patch the old implementation.

The main cleanup goals were:

- remove mojibake/encoding-corrupted source and seed content
- make the Rust engine the canonical planning authority
- make data metadata explicit enough for meaningful scoring
- align the static web app with the same deterministic rules
- make CLI behavior explicit and useful for real output files

## Current State

- Rust workspace remains three crates: `mps-core`, `mps-db`, `mps-cli`.
- `mps-core` validates requests, builds strategy, scores candidates, enforces
  safety/equipment/level rules, and validates 7-phase output.
- `mps-db` loads clean SQLite seed data and maps it into core repository records.
- `mps-cli` supports `init-db`, `seed`, and `generate`.
- `mps-cli generate` supports `--format markdown|md|json` and optional
  `--output`.
- `web/` is a clean static instructor console with a JS port of the generator.

## Verification

Latest checks:

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\mps-target-clean"
cargo test -q
node --check web\seed-data.js
node --check web\mps-engine.js
node --check web\app.js
```

Result:

- 41 Rust tests pass.
- Web JavaScript syntax checks pass.
- CLI smoke against a fresh SQLite database passes.
- Web jsdom smoke passes for 7 phases, JSON view, preset switching, and
  Chair-only apparatus filtering.

## Important Invariants

- Every generated plan has exactly 7 phases.
- BUILD must contain 2-4 Prime exercises.
- Reformer exercises appear only when Reformer is requested.
- Chair exercises appear only when Chair is requested.
- Mat and Standing may appear only as movement-context work.
- Hard-excluded exercises are not selected and are reported.
- RequireRegression exercises include a regression note when selected.
- Movement strategy emphasis totals 100%.

## Known Tradeoffs

- SQLite repository still uses a sync trait with a Tokio runtime internally.
  This is acceptable for CLI/static MVP but can be revisited for an API server.
- `web/seed-data.js` mirrors `data/seed/mps_seed.sql` manually. A future script
  should generate the browser dataset from SQLite or one canonical JSON source.
- Browser smoke currently uses jsdom rather than full visual browser automation.

## Next Suggested Work

1. Add a data parity generator so web seed data cannot drift from SQLite seed.
2. Add GitHub Actions for `cargo test -q` and JS syntax checks.
3. Add markdown snapshot tests.
4. Add more exercises per apparatus and experience to reduce repetition.
5. Deploy after the next commit if production should reflect this rebuild.
