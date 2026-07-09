# AGENTS.md — MPS (Memellow Programming System)

> Context file for AI agents working on this project.

## Project Overview

**MPS** is a deterministic Pilates class plan generator for **Total Body Pilates** instructors. Given a class request (movement experience, duration, level, equipment, observations), it produces a complete class plan with 7 journey phases, scored exercise selection, benchmarks, and safety notes.

**Key principle:** No LLM in the generation loop. All decisions are deterministic scoring algorithms over a seed database.

## Architecture

```
Rust Workspace (3 crates)
├── mps-core      — Domain types, scoring, strategy, generator, markdown renderer
├── mps-db        — SQLite repository (sqlx + migrations + seed data)
└── mps-cli       — CLI binary (`mps` command)

Web Frontend (static)
├── web/index.html      — Dark-themed SPA with form UI
├── web/mps-engine.js   — JS port of Rust generation logic
└── web/seed-data.js    — Auto-extracted from SQLite seed database
```

## Crate Details

### mps-core (1,556 lines)
Pure domain logic, no I/O. All types are `Serialize`/`Deserialize`.

| Module | Lines | Purpose |
|---|---|---|
| `domain.rs` | 155 | Enums: `MovementExperience`, `ClassLevel`, `Equipment`, `MovementSystem`, `ExerciseRole`, `RiskPolicy` |
| `class_request.rs` | 145 | `ClassRequest` struct + validation |
| `class_plan.rs` | 90 | Output structs: `ClassPlan`, `JourneyPhasePlan`, `ExerciseTeachingUnit`, `SafetySummary` |
| `repository.rs` | 68 | `MpsRepository` trait (sync) + record types (`BaseStrategyRecord`, `ExerciseRecord`, etc.) |
| `phase_allocation.rs` | 208 | Phase duration templates for 45/60/75 min classes |
| `strategy.rs` | 164 | `build_strategy()` — merges base strategy with observation modifiers |
| `scoring.rs` | 73 | `difficulty_fit_score()`, `role_match_score()`, `objective_match_score()` |
| `generator.rs` | 378 | `generate_class_plan<R: MpsRepository>()` — the main engine |
| `markdown.rs` | 220 | `render_markdown(&ClassPlan) -> String` |
| `error.rs` | 30 | `MpsError` enum with `thiserror` |

### mps-db (847 lines)
SQLite persistence via sqlx 0.7 with async runtime.

| File | Purpose |
|---|---|
| `sqlite_repository.rs` | `SqliteRepository` — implements `MpsRepository`, wraps `SqlitePool` with `tokio::runtime::Runtime::block_on()` |
| `migrations/0001_initial_schema.sql` | 19 tables schema |

### mps-cli (88 lines)
Clap-based CLI with 3 subcommands: `init-db`, `seed`, `generate`.

## Seed Data

Located at `data/seed/mps_seed.sql`. Contains:
- 3 movement experiences: ShoulderFreedom, HappyHips, SpineReset
- 4 equipment types: Reformer, Chair, Mat, Standing
- 6 movement systems: Shoulder, Thoracic, Hip, Legs, Spine, BreathCore
- 3 base strategies (one per experience)
- 12 observation modifiers (keyword-matched)
- 24 exercises across Reformer/Chair/Mat/Standing
- 6 benchmarks (2 per experience)

## Generation Algorithm

1. **Validate** request
2. **Allocate phases** — 7 phases with fixed duration templates
3. **Load strategy** — base strategy + observation modifier adjustments
4. **Filter exercises** — by safety contraindications (HardExclude)
5. **Filter by equipment** — only exercises matching selected equipment
6. **For each phase:**
   - Map phase → target role (ARRIVE→Assess, BUILD→Prime, etc.)
   - Filter by role + level range
   - Score: role_match(20) + difficulty_fit(0-5) + objective_match(0-N×5)
   - Select top exercises until time budget filled
7. **Build output** — ClassPlan with journey, benchmarks, safety, retest

## Environment (Windows)

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR=/c/Users/Admin/AppData/Local/Temp/mps-target  # Required — project path has spaces
```

**Rust:** 1.96.1, Edition 2021
**Build tools:** Visual Studio Build Tools 2022 (MSVC linker)
**sqlite3:** Available in PATH

## Build & Test

```bash
cargo test                    # All tests (38 total)
cargo test -p mps-core        # Core tests (25)
cargo test -p mps-db          # DB tests (13)
cargo build --release -p mps-cli  # Release binary
```

## Deployment

| Platform | URL | Notes |
|---|---|---|
| GitHub | https://github.com/ajbk/mps-cli | Branch: `mps-cli-mvp` |
| Vercel | https://mps-cli-mvp.vercel.app | Static deploy from `web/` dir |
| Vercel Account | `akoong` | via `vercel --yes --prod` |

**Vercel config:** `vercel.json` routes `/` → `/web/` static files.

## File Conventions

- Rust: `snake_case` modules, `PascalCase` types, `camelCase` fields (serde)
- Seed SQL: `data/seed/mps_seed.sql`
- Examples: `examples/<experience>_<duration>.json`
- Output: `output/*.md` or `output/*.json`

## Known Issues

- **Windows file lock:** SQLite `.sqlite` files can't be `rm -f` immediately after use. Use unique filenames.
- **CARGO_TARGET_DIR:** Must be exported before any `cargo` command due to spaces in project path.
- **sqlx::migrate!():** Resolves migrations at compile time from `CARGO_MANIFEST_DIR` of the `mps-db` crate.
- **Sync MpsRepository:** Uses `tokio::runtime::Runtime::block_on()` inside each method (not ideal but works).

## Future Work

- [ ] README with quickstart guide
- [ ] Integration test with real SQLite DB
- [ ] Web UI: pre-generate all 45 combinations (3×3×5) as static JSON
- [ ] Vercel: add GitHub Actions auto-deploy on push
- [ ] Equipment filter in Rust CLI (currently only in web JS engine)
