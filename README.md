# MPS CLI

Memellow Programming System (MPS) is a deterministic Pilates class plan
generator for Total Body Pilates instructors.

Given a structured request, it creates a complete class plan with:

- movement strategy
- before-class benchmarks
- seven journey phases
- scored exercise selection
- safety exclusions and modifications
- after-class retest guidance

No LLM is used in the generation loop.

## Workspace

```text
crates/mps-core  deterministic domain engine
crates/mps-db    SQLite repository and migrations
crates/mps-cli   command-line interface
web/             static browser app
data/seed/       canonical SQLite seed data
examples/        sample class requests
```

## Requirements

- Rust toolchain
- Node.js for web syntax checks
- Vercel CLI for deployment

On Windows, set `CARGO_TARGET_DIR` outside the project directory:

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\mps-target"
```

## Quickstart

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\mps-target"
cargo test -q

$db = Join-Path $env:TEMP "mps-$([guid]::NewGuid()).sqlite"
cargo run -p mps-cli -- init-db --database $db
cargo run -p mps-cli -- seed --database $db --file data/seed/mps_seed.sql
cargo run -p mps-cli -- generate examples/shoulder_freedom_60.json --database $db --format markdown
```

Write output to a file:

```powershell
cargo run -p mps-cli -- generate examples/happy_hips_45.json --database $db --format json --output "$env:TEMP\happy_hips.json"
```

## Request Shape

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

Supported values:

- Movement experiences: `ShoulderFreedom`, `HappyHips`, `SpineReset`
- Durations: `45`, `60`, `75`
- Primary apparatus: `Reformer`, `Chair`
- Risk policies: `Conservative`, `Balanced`

`Mat` and `Standing` are movement contexts. They are selected by the engine for
assessment, preparation, transfer, and retest work; they are not valid primary
apparatus in requests.

## Web App

The static app in `web/` is an instructor console powered by a browser-side port
of the deterministic engine.

Local smoke checks:

```powershell
node --check web\seed-data.js
node --check web\mps-engine.js
node --check web\app.js
```

Deploy:

```powershell
vercel --yes --prod
```

Production:

```text
https://mps-cli-mvp.vercel.app
```

## Quality Gates

- `cargo test -q`
- JS syntax checks for `web/seed-data.js`, `web/mps-engine.js`, `web/app.js`
- Fresh SQLite CLI smoke: init, seed, generate markdown, generate JSON
- Web smoke: 7 phases render, JSON view works, presets work, Chair-only does
  not select Reformer
