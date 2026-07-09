# AGENTS.md - MPS

Context for agents working on the Memellow Programming System.

## Project

MPS is a deterministic Pilates class plan generator for Total Body Pilates
instructors. It turns a structured `ClassRequest` into a canonical `ClassPlan`
with movement strategy, benchmarks, seven journey phases, selected exercises,
safety summary, and retest guidance.

Core principle: no LLM is allowed in the generation loop. The canonical plan is
created by deterministic Rust rules and seed metadata.

## Architecture

```text
Rust workspace
  crates/mps-core  domain model, scoring, strategy, generator, markdown
  crates/mps-db    SQLite migrations, seed loading, repository adapter
  crates/mps-cli   thin command-line wrapper

Static web app
  web/index.html   app shell
  web/styles.css   instructor-console UI
  web/app.js       browser UI state and rendering
  web/mps-engine.js deterministic JS port of the Rust rules
  web/seed-data.js clean browser seed dataset
```

## Current Quality Gates

- `cargo test -q` passes 41 tests.
- `node --check web/seed-data.js web/mps-engine.js web/app.js` passes.
- CLI smoke works with a fresh SQLite DB and clean seed file.
- Web smoke works with jsdom: 7 phases, JSON view, presets, and Chair-only
  equipment filtering.

## Generation Rules

1. Validate request: students, duration, equipment, and primary apparatus.
2. Load base strategy for the movement experience.
3. Match observations to modifiers and normalize emphasis to 100%.
4. Load benchmarks and exercises from the repository.
5. For each phase, map phase to target role:
   - ARRIVE -> Assess
   - PREPARE -> Prepare
   - BUILD -> Prime
   - INTEGRATE -> Integrate
   - CHALLENGE -> Challenge
   - TRANSFER -> Transfer
   - RESET & RETEST -> Restore
6. Filter by safety, phase/equipment allowance, and level.
7. Score by role, difficulty fit, movement experience, movement systems,
   preferred objectives, equipment match, and safety penalty.
8. Select bounded phase candidates. BUILD must contain 2-4 Prime exercises.
9. Return `ClassPlan` and render Markdown from that same struct.

## Equipment Rules

- `Reformer` and `Chair` are primary apparatus and must be requested.
- `Mat` and `Standing` are movement contexts.
- Movement contexts may appear in ARRIVE, PREPARE, TRANSFER, and RESET & RETEST.
- Unrequested primary apparatus must never appear in the generated plan.

## Safety Rules

- `HardExclude`: exercise is removed and reported in safety summary.
- `RequireRegression`: exercise can be selected only with a regression note.
- `Caution`: exercise can be selected with safety notes and score penalty.

Safety is group-level only in this MVP. Per-student safety is future scope.

## Commands

PowerShell:

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\mps-target"
cargo test -q

$db = Join-Path $env:TEMP "mps-$([guid]::NewGuid()).sqlite"
cargo run -p mps-cli -- init-db --database $db
cargo run -p mps-cli -- seed --database $db --file data/seed/mps_seed.sql
cargo run -p mps-cli -- generate examples/shoulder_freedom_60.json --database $db --format markdown
```

Deployment:

```powershell
vercel --yes --prod
```

Production URL:

```text
https://mps-cli-mvp.vercel.app
```

## Files To Keep Clean

- `data/seed/mps_seed.sql` is canonical SQLite seed data.
- `web/seed-data.js` mirrors seed data for the static browser app.
- Keep source and seed files ASCII unless there is a deliberate reason not to.
- Avoid changing generated/artifact directories such as `target/`, `output/`,
  `deploy/`, and `.superpowers/` unless the user explicitly asks.
