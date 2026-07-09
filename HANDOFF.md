# MPS CLI MVP — Session Handoff

> เอกสารนี้สร้างจาก conversation session วันที่ 9 กรกฎาคม 2026
> เพื่อให้ agent ใหม่สามารถรับช่วงต่อได้ทันที

## สรุปสถานะปัจจุบัน

**MPS (Memellow Programming System)** เป็น Pilates class plan generator ที่สร้าง class plan อัตโนมัติจาก input JSON — deterministic, no LLM in the loop.

### ✅ สิ่งที่เสร็จแล้ว (MVP Complete)

| Component | Status | Details |
|---|---|---|
| Rust Workspace | ✅ | 3 crates: mps-core, mps-db, mps-cli |
| Domain Types | ✅ | 1,556 lines — enums, structs, validation |
| Generator Engine | ✅ | 7-phase scoring algorithm, 24 exercises |
| SQLite Repository | ✅ | sqlx 0.7, migrations, seed data (24 exercises, 12 modifiers, 6 benchmarks) |
| CLI Binary | ✅ | `mps init-db`, `mps seed`, `mps generate` |
| Web Frontend | ✅ | Dark-themed SPA, client-side JS engine |
| GitHub | ✅ | https://github.com/ajbk/mps-cli (branch: mps-cli-mvp) |
| Vercel | ✅ | https://mps-cli-mvp.vercel.app (account: akoong) |

### 📊 Test Results

- **38/38 tests pass** (25 mps-core + 13 mps-db)
- CLI smoke test: `mps generate` produces valid markdown + JSON
- Web: all equipment chips clickable, equipment filter working

### 📁 Project Structure

```
D:\AI Project\MPS\.worktrees\mps-cli-mvp\
├── AGENTS.md                    # AI context file (just created)
├── Cargo.toml                   # Workspace: 3 crates
├── vercel.json                  # Vercel deploy config
├── crates/
│   ├── mps-core/src/            # 11 modules, 1,556 lines
│   │   ├── domain.rs            # Enums: MovementExperience, ClassLevel, Equipment, etc.
│   │   ├── class_request.rs     # Input struct + validation
│   │   ├── class_plan.rs        # Output structs
│   │   ├── repository.rs        # MpsRepository trait (sync)
│   │   ├── generator.rs         # Main engine: generate_class_plan<R>()
│   │   ├── strategy.rs          # build_strategy() — base + observation modifiers
│   │   ├── scoring.rs           # difficulty_fit, role_match, objective_match
│   │   ├── phase_allocation.rs  # 45/60/75 min templates
│   │   ├── markdown.rs          # render_markdown()
│   │   └── error.rs             # MpsError enum
│   ├── mps-db/src/              # 847 lines
│   │   ├── sqlite_repository.rs # SqliteRepository (sqlx + tokio block_on)
│   │   └── migrations/          # 0001_initial_schema.sql (19 tables)
│   └── mps-cli/src/
│       └── main.rs              # Clap CLI (88 lines)
├── data/seed/mps_seed.sql       # Seed data (24 exercises)
├── examples/*.json              # 3 example requests
├── web/                         # Vercel static frontend
│   ├── index.html               # Dark-themed SPA
│   ├── mps-engine.js            # JS port of Rust generation logic
│   └── seed-data.js             # Auto-extracted from SQLite
└── output/                      # Generated class plans
```

### 🔧 Environment

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR=/c/Users/Admin/AppData/Local/Temp/mps-target  # Required!
```

- **Rust:** 1.96.1, Edition 2021
- **Build Tools:** Visual Studio Build Tools 2022 (MSVC)
- **Windows file lock:** SQLite files can't be deleted immediately — use unique filenames

### 🚀 Quick Commands

```bash
# Build
cargo build --release -p mps-cli

# Test
cargo test  # 38/38 pass

# CLI usage
mps init-db --database mps.sqlite
mps seed --database mps.sqlite --file data/seed/mps_seed.sql
mps generate examples/shoulder_freedom_60.json --database mps.sqlite --format markdown

# Deploy
vercel --yes --prod
```

### 🔐 Credentials

- **GitHub:** account `ajbk` — token in `~/.git-credentials`
- **Vercel:** account `akoong` — already authenticated via CLI

### ⚠️ Known Issues

1. **CARGO_TARGET_DIR** — ต้อง export ก่อน cargo เสมอ (project path มีช่องว่าง)
2. **Windows file lock** — SQLite ลบไม่ได้ทันที ต้องใช้ชื่อ unique
3. **Sync MpsRepository** — ใช้ `tokio::runtime::Runtime::block_on()` (ไม่ ideal แต่ทำงานได้)
4. **Equipment filter** — มีแค่ใน web JS engine, Rust CLI ยังไม่มี

### 📋 TODO ที่เหลือ

- [ ] README.md quickstart guide
- [ ] Integration test with real SQLite DB
- [ ] Equipment filter in Rust CLI (currently only in web)
- [ ] GitHub Actions auto-deploy to Vercel on push
- [ ] Pre-generate all 45 combinations as static JSON

### 🎯 Suggested Skills

ถ้า agent ใหม่จะทำงานต่อ แนะนำโหลด skills เหล่านี้:

- `github-pr-workflow` — ถ้าจะทำ PR
- `github-repo-management` — ถ้าจะจัดการ repo
- `test-driven-development` — ถ้าจะเพิ่ม tests
- `plan` — ถ้าจะวางแผน feature ใหม่

### 📌 Git History (ล่าสุด)

```
19e07a9 docs: add AGENTS.md for AI context
46e977d fix: filter exercises by selected equipment
38ab5f1 fix: sync equipment chip visual with checkbox state
06e9a2b fix: make equipment chips properly clickable
49607b9 fix: improve equipment chip visual feedback
e16aac3 chore: add vercel config
f397d46 feat: add web frontend for Vercel deployment
8349d65 feat: add example class request files
321bf8f feat: add MPS CLI commands
f818c0d feat: generate deterministic class plans
84411d7 feat: implement SQLite repository
...
a124683 chore: scaffold MPS Rust workspace
```
