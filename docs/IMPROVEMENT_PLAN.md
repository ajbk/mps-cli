# MPS Improvement Plan

Date: 2026-07-09

## Review Findings

1. The Rust generator scored selected equipment but did not strictly filter
   unselected primary apparatus. This could produce a class using Reformer when
   the request asked for Chair only.
2. The web engine filtered only selected equipment, which could remove Mat and
   Standing context work from ARRIVE, TRANSFER, and RESET & RETEST.
3. There was no README quickstart, making the project harder to hand off.
4. SQLite-backed generation had repository tests, but no direct integration test
   proving a seeded database can generate a plan while respecting equipment.
5. `mps-db` emitted warnings from row fields selected but never read.

## Pass 1 Plan

- Enforce selected primary apparatus in `mps-core`.
- Preserve Mat and Standing only for context phases.
- Mirror the same equipment rule in `web/mps-engine.js`.
- Add core tests for selected apparatus and movement context behavior.
- Add a SQLite seed integration test for real generation.
- Remove warning-producing unused row fields.
- Add README, flashcards, and this improvement plan.

## Next Backlog

1. Add a web/Rust parity check that compares generated plans for the same seed
   request.
2. Add CLI `--output` support so generated plans can be written directly to a
   file.
3. Pre-generate static JSON plans for common combinations and serve them in the
   web app.
4. Add GitHub Actions for `cargo test` on every push.
5. Add CI deployment to Vercel after tests pass.
6. Expand seed data beyond 24 exercises and add coverage targets per movement
   experience, level, and equipment.
7. Add data validation tests that detect exercises without roles, objectives, or
   teaching cues.
8. Add snapshot tests for markdown output so instructor-facing formatting stays
   stable.
9. Add structured warnings when a phase is underfilled by more than a threshold.
10. Consider replacing per-method `Runtime::block_on()` with an async repository
    boundary if the app grows beyond CLI/static use.

## Quality Gates

- `cargo test` passes.
- No compiler warnings from touched Rust crates.
- Chair-only requests never include Reformer exercises.
- Reformer-only requests never include Chair exercises.
- Mat and Standing appear only in context phases.
- BUILD fails loudly if no Prime exercise can be selected.
