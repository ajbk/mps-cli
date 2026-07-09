# MPS CLI

MPS, the Memellow Programming System, is a deterministic Pilates class plan
generator for Total Body Pilates instructors.

Given a structured class request, it produces a complete class plan with:

- movement strategy
- before-class benchmarks
- seven journey phases
- scored exercise selection
- safety notes and excluded exercises
- after-class retest guidance

The generation loop is deterministic. It does not call an LLM.

## Workspace

This repository is a Rust workspace with three crates:

- `mps-core`: domain types, validation, scoring, strategy, generator, markdown
- `mps-db`: SQLite repository, migrations, seed loading
- `mps-cli`: command-line interface

It also includes a static web prototype in `web/` that ports the deterministic
engine to browser-side JavaScript.

## Requirements

- Rust toolchain
- SQLite support through `sqlx`
- On Windows, set `CARGO_TARGET_DIR` outside the project path to avoid path and
  file-lock friction.

PowerShell:

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\mps-target"
```

## Quickstart

Build and test:

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\mps-target"
cargo test
cargo build -p mps-cli
```

Create and seed a local database:

```powershell
$db = Join-Path $env:TEMP "mps-$([guid]::NewGuid()).sqlite"
cargo run -p mps-cli -- init-db --database $db
cargo run -p mps-cli -- seed --database $db --file data/seed/mps_seed.sql
```

Generate a markdown class plan:

```powershell
cargo run -p mps-cli -- generate examples/shoulder_freedom_60.json --database $db --format markdown
```

Generate JSON instead:

```powershell
cargo run -p mps-cli -- generate examples/shoulder_freedom_60.json --database $db --format json
```

## Request Shape

Example:

```json
{
  "students": 4,
  "movement_experience": "ShoulderFreedom",
  "level": "BeginnerIntermediate",
  "equipment": ["Reformer", "Chair"],
  "duration_minutes": 60,
  "observations": ["thoracic stiffness", "limited overhead reach"],
  "group_safety": {
    "contraindications": ["wrist_pain"],
    "risk_policy": "Conservative"
  }
}
```

Supported durations are `45`, `60`, and `75` minutes. Primary equipment can be
`Reformer`, `Chair`, or both. `Mat` and `Standing` are movement contexts used by
the engine for arrival, transfer, and retest work; they are not valid primary
apparatus in requests.

## Equipment Selection Rule

The generator enforces selected primary apparatus:

- Reformer exercises are used only when `Reformer` is requested.
- Chair exercises are used only when `Chair` is requested.
- Mat and Standing exercises are allowed only in ARRIVE, TRANSFER, and RESET &
  RETEST phases.

This keeps apparatus-specific programming honest while preserving the movement
context phases needed for assessment and retest.

## Web Prototype

The static web app lives in `web/`.

For a quick local check, open `web/index.html` in a browser. The deployed static
prototype is configured by `vercel.json`.

## Useful Files

- `AGENTS.md`: agent-facing architecture and workflow notes
- `HANDOFF.md`: latest session handoff
- `CONTEXT.md`: domain glossary
- `data/seed/mps_seed.sql`: MVP seed database content
- `examples/`: sample class requests
- `docs/FLASHCARDS.md`: fast project learning cards
- `docs/IMPROVEMENT_PLAN.md`: next improvement plan
