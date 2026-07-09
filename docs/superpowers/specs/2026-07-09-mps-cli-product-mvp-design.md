# MPS CLI Product MVP — PRD / Design Spec

## Product Overview

Memellow Programming System (MPS) คือ Teaching Intelligence System สำหรับช่วย instructor ออกแบบ Total Body Pilates Class จาก Movement Experience, Movement Strategy, exercise metadata, safety constraints และ deterministic generation rules.

MVP นี้จะเป็น Rust CLI product ที่ generate class plan ได้จริงจาก JSON input และ SQLite exercise database โดย output ได้ทั้ง structured JSON และ instructor-friendly Markdown.

## Product Vision

Every class is a Total Body Pilates Class with a clear Movement Experience, allowing students to immediately feel improvements in their movement after class.

MPS is not an exercise library. ระบบไม่ได้เริ่มจาก “วันนี้ใช้ท่าอะไรดี” แต่เริ่มจาก:

```text
Movement Experience
→ Movement Benchmark
→ Movement Strategy
→ Exercise Selection
→ Movement Journey
→ Movement Benchmark Retest
```

## Target User

Primary user:

- Pilates instructor
- Studio educator
- Instructor trainer
- Advanced practitioner designing structured class plans

MVP assumes the user can prepare a JSON request or use example templates. A future UI may make this accessible to non-technical instructors.

## Goals

1. Generate a usable Pilates class plan from structured input.
2. Preserve MPS philosophy: Total Body, one clear Movement Experience, before/after benchmark, movement strategy, and retest.
3. Use deterministic Rust logic for class quality, explainability, and testing.
4. Use SQLite as the MVP data store for exercise metadata, strategies, modifiers, benchmarks, and safety rules.
5. Output both canonical JSON and instructor-friendly Markdown.
6. Keep core generation independent from CLI, SQLite, HTTP, and LLM providers.

## Non-Goals

MVP will not include:

- Web UI
- User accounts
- Student profile persistence
- Payment / booking
- Analytics dashboard
- Full admin CMS
- Per-student modification engine
- Fully flexible duration interpolation
- LLM-driven class planning
- Medical diagnosis or medical screening

## Locked Product Decisions

| Area | Decision |
|---|---|
| MVP Scope | Rust Engine + Thin Instructor Class Generator |
| Interface | Rust Library + CLI first, API later |
| Database | SQLite first |
| Algorithm | Hybrid: structure rules + scoring-based exercise selection |
| Strategy | Base Strategy + Data-Driven Observation Modifiers |
| Output | Rich Structured JSON + Markdown Renderer |
| Movement Experiences | Shoulder Freedom, Happy Hips, Spine Reset |
| Equipment | Reformer + Chair + Mat/Standing Context |
| Duration | 45, 60, 75 minutes |
| Level | 5 practical levels |
| AI/LLM | Deterministic Core + Optional LLM Text Enhancement later |
| Safety | Full Safety Direction, MVP Safety Foundation |
| Safety Input | Group-Level MVP, Per-Student Later |
| Deliverable | CLI Product MVP + Core Engine Boundary |
| Language | Hybrid Thai/English |

## MVP User Flow

```text
Instructor prepares ClassRequest JSON
        ↓
CLI validates input
        ↓
SQLite repositories load movement strategy, modifiers, benchmarks, exercises, safety rules
        ↓
MPS core generates deterministic ClassPlan
        ↓
CLI renders JSON or Markdown
        ↓
Instructor teaches from generated plan
```

## CLI Workflow

Required commands:

```bash
mps init-db --database mps.sqlite
mps seed --database mps.sqlite --file data/seed/mps_seed.sql
mps generate examples/shoulder_freedom_60.json --database mps.sqlite --format markdown
mps generate examples/happy_hips_45.json --database mps.sqlite --format json
mps generate examples/spine_reset_75.json --database mps.sqlite --format markdown
```

Accepted format values:

```text
json
markdown
md
```

## Functional Requirements

### FR1 — Validate ClassRequest

The CLI must read a JSON request and validate:

- movement experience is supported
- level is supported
- duration is 45, 60, or 75
- equipment contains only supported primary apparatus
- group safety risk policy is valid
- observations and contraindications are parsed as tags / strings

### FR2 — Generate Movement Strategy

The system must:

1. Load base strategy for the requested Movement Experience.
2. Match observations to observation modifiers.
3. Apply emphasis adjustments.
4. Add objectives and preferred exercise objectives.
5. Normalize total-body emphasis to 100%.
6. Produce a human-readable strategy explanation.

### FR3 — Allocate Movement Journey

The system must generate all seven phases:

1. ARRIVE
2. PREPARE
3. BUILD
4. INTEGRATE
5. CHALLENGE
6. TRANSFER
7. RESET_RETEST

Duration templates:

| Phase | 45 | 60 | 75 |
|---|---:|---:|---:|
| ARRIVE | 4 | 5 | 6 |
| PREPARE | 7 | 10 | 12 |
| BUILD | 15 | 20 | 26 |
| INTEGRATE | 7 | 10 | 13 |
| CHALLENGE | 6 | 8 | 10 |
| TRANSFER | 3 | 4 | 5 |
| RESET_RETEST | 3 | 3 | 3 |

### FR4 — Select Benchmarks

Every class must include before benchmark and retest. Benchmark selection must match Movement Experience and strategy objectives.

### FR5 — Select Exercises

Exercise selection must be hybrid:

- rules enforce structure and safety
- scoring chooses best candidates per phase

BUILD must contain 2–4 Prime exercises.

### FR6 — Enforce Equipment Scope

Primary apparatus:

- Reformer
- Chair

Movement contexts:

- Mat
- Standing

Mat/Standing are allowed for ARRIVE, PREPARE, TRANSFER, RESET_RETEST, and selected fallback cases. Unrequested apparatus must be excluded.

### FR7 — Enforce Level and 80/20 Rule

Supported levels:

- Beginner
- BeginnerIntermediate
- Intermediate
- IntermediateAdvanced
- Advanced

Most selected exercises should be at or below class level. Slightly above-level exercises are allowed only in CHALLENGE phase and only with regression support.

### FR8 — Apply Group Safety

MVP input supports group-level safety constraints:

```json
"group_safety": {
  "contraindications": ["wrist_pain"],
  "risk_policy": "Conservative"
}
```

The engine must:

- hard exclude matching hard contraindications
- downgrade caution candidates
- require regression where configured
- add safety notes to ClassPlan
- report excluded or modified exercises

### FR9 — Render ClassPlan

The canonical output is `ClassPlan` JSON. Markdown rendering must be generated from the same struct.

Markdown must include:

- title
- summary
- movement strategy
- emphasis table
- benchmark
- seven-phase journey
- exercise teaching units
- safety notes
- retest
- expected improvement

## ClassRequest Input Contract

Example:

```json
{
  "students": 4,
  "movement_experience": "ShoulderFreedom",
  "level": "BeginnerIntermediate",
  "equipment": ["Reformer", "Chair"],
  "duration_minutes": 60,
  "observations": [
    "thoracic stiffness",
    "limited overhead reach",
    "office syndrome"
  ],
  "group_safety": {
    "contraindications": ["wrist_pain"],
    "risk_policy": "Conservative"
  }
}
```

## ClassPlan Output Contract

Top-level fields:

```text
class_title
movement_experience
duration_minutes
level
students
equipment
movement_strategy
benchmark
journey
safety_summary
retest
expected_improvement
warnings
```

Exercise teaching unit fields:

```text
exercise_id
name
apparatus
role
duration_minutes
movement_objectives
why_selected
teaching_cues
regression
progression
safety_notes
```

## Domain Model

Core enums / concepts:

```text
MovementExperience
MovementSystem
ClassLevel
Equipment
EquipmentCategory
MovementJourneyPhase
ExerciseRole
RiskPolicy
SafetySeverity
ChallengeType
```

## Movement Strategy Model

Base strategy data:

```text
movement_experience
primary_focus
secondary_focus
emphasis_distribution
base_objectives
recommended_benchmarks
explanation_template
```

Observation modifier data:

```text
keywords
emphasis_adjustments
added_objectives
preferred_exercise_objectives
explanation_fragment
```

## Generation Algorithm

```text
validate request
load base strategy
match observation modifiers
apply strategy adjustments
normalize emphasis
allocate phase durations
select benchmarks
for each phase:
  load candidate exercises by role/equipment/level/safety
  score candidates
  select exercises to satisfy phase constraints
validate full class rules
render ClassPlan
```

Scoring factors:

```text
role_match_score
movement_experience_match_score
movement_objective_match_score
movement_system_emphasis_score
equipment_match_score
difficulty_fit_score
observation_relevance_score
balanced_body_order_fit_score
safety_penalty
contraindication_exclusion
```

## SQLite Database Design

Initial tables:

```sql
CREATE TABLE movement_experiences (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE base_movement_strategies (
  movement_experience_id TEXT PRIMARY KEY,
  primary_focus TEXT NOT NULL,
  secondary_focus TEXT NOT NULL,
  explanation_template TEXT NOT NULL,
  FOREIGN KEY (movement_experience_id) REFERENCES movement_experiences(id)
);

CREATE TABLE base_strategy_emphasis (
  movement_experience_id TEXT NOT NULL,
  movement_system TEXT NOT NULL,
  percentage INTEGER NOT NULL,
  PRIMARY KEY (movement_experience_id, movement_system),
  FOREIGN KEY (movement_experience_id) REFERENCES movement_experiences(id)
);

CREATE TABLE base_strategy_objectives (
  movement_experience_id TEXT NOT NULL,
  objective TEXT NOT NULL,
  PRIMARY KEY (movement_experience_id, objective),
  FOREIGN KEY (movement_experience_id) REFERENCES movement_experiences(id)
);

CREATE TABLE observation_modifiers (
  id TEXT PRIMARY KEY,
  explanation_fragment TEXT NOT NULL
);

CREATE TABLE observation_modifier_keywords (
  modifier_id TEXT NOT NULL,
  keyword TEXT NOT NULL,
  PRIMARY KEY (modifier_id, keyword),
  FOREIGN KEY (modifier_id) REFERENCES observation_modifiers(id)
);

CREATE TABLE observation_emphasis_adjustments (
  modifier_id TEXT NOT NULL,
  movement_system TEXT NOT NULL,
  adjustment INTEGER NOT NULL,
  PRIMARY KEY (modifier_id, movement_system),
  FOREIGN KEY (modifier_id) REFERENCES observation_modifiers(id)
);

CREATE TABLE observation_added_objectives (
  modifier_id TEXT NOT NULL,
  objective TEXT NOT NULL,
  PRIMARY KEY (modifier_id, objective),
  FOREIGN KEY (modifier_id) REFERENCES observation_modifiers(id)
);

CREATE TABLE equipment (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  category TEXT NOT NULL CHECK (category IN ('apparatus', 'movement_context'))
);

CREATE TABLE exercises (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  min_level TEXT NOT NULL,
  max_level TEXT NOT NULL,
  difficulty INTEGER NOT NULL CHECK (difficulty BETWEEN 1 AND 5),
  balanced_body_order INTEGER,
  default_duration_minutes INTEGER NOT NULL
);

CREATE TABLE exercise_equipment (
  exercise_id TEXT NOT NULL,
  equipment_id TEXT NOT NULL,
  PRIMARY KEY (exercise_id, equipment_id),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id),
  FOREIGN KEY (equipment_id) REFERENCES equipment(id)
);

CREATE TABLE exercise_roles (
  exercise_id TEXT NOT NULL,
  role TEXT NOT NULL,
  PRIMARY KEY (exercise_id, role),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE exercise_objectives (
  exercise_id TEXT NOT NULL,
  objective TEXT NOT NULL,
  PRIMARY KEY (exercise_id, objective),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE exercise_experience_tags (
  exercise_id TEXT NOT NULL,
  movement_experience_id TEXT NOT NULL,
  PRIMARY KEY (exercise_id, movement_experience_id),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id),
  FOREIGN KEY (movement_experience_id) REFERENCES movement_experiences(id)
);

CREATE TABLE exercise_contribution_scores (
  exercise_id TEXT NOT NULL,
  objective TEXT NOT NULL,
  score INTEGER NOT NULL CHECK (score BETWEEN 1 AND 5),
  PRIMARY KEY (exercise_id, objective),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE exercise_cues (
  id TEXT PRIMARY KEY,
  exercise_id TEXT NOT NULL,
  cue_type TEXT NOT NULL,
  cue_text TEXT NOT NULL,
  related_objective TEXT,
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE exercise_regressions (
  id TEXT PRIMARY KEY,
  exercise_id TEXT NOT NULL,
  description TEXT NOT NULL,
  reason TEXT NOT NULL,
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE exercise_progressions (
  id TEXT PRIMARY KEY,
  exercise_id TEXT NOT NULL,
  description TEXT NOT NULL,
  added_challenge_type TEXT NOT NULL,
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE exercise_contraindications (
  exercise_id TEXT NOT NULL,
  contraindication_tag TEXT NOT NULL,
  severity TEXT NOT NULL CHECK (severity IN ('hard_exclude', 'caution', 'require_regression')),
  note TEXT NOT NULL,
  PRIMARY KEY (exercise_id, contraindication_tag),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

CREATE TABLE benchmarks (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  instruction TEXT NOT NULL,
  movement_experience_id TEXT NOT NULL,
  FOREIGN KEY (movement_experience_id) REFERENCES movement_experiences(id)
);

CREATE TABLE benchmark_watch_points (
  benchmark_id TEXT NOT NULL,
  watch_point TEXT NOT NULL,
  PRIMARY KEY (benchmark_id, watch_point),
  FOREIGN KEY (benchmark_id) REFERENCES benchmarks(id)
);
```

## Safety Model

MVP safety is group-level. Per-student safety is future architecture.

MVP risk policies:

```text
Conservative
Balanced
```

The engine must include a disclaimer in Markdown:

> This plan is a teaching aid, not a medical diagnosis. Instructor judgement is required.

## Rust Workspace Architecture

```text
mps/
  Cargo.toml
  crates/
    mps-core/
    mps-db/
    mps-cli/
  data/
    seed/
  examples/
```

### mps-core

Owns:

- domain structs
- validation
- strategy generation
- scoring
- class generation
- JSON-serializable ClassPlan
- Markdown renderer if kept dependency-light

Must not depend on:

- SQLite
- CLI
- HTTP
- LLM

### mps-db

Owns:

- SQLite connection
- migrations
- seed execution
- repository implementations

### mps-cli

Owns:

- command parsing
- file IO
- database path
- output format selection
- stdout / output file

## Seed Data Requirements

MVP seed must include:

- 3 Movement Experiences
- 3 base strategies
- 8–12 observation modifiers
- 20–30 exercises
- 6–10 benchmarks
- 3–5 cues per major exercise
- regressions/progressions for challenge-relevant exercises
- safety tags for common constraints

## Acceptance Criteria

### AC1 — CLI Demo Works

```bash
mps generate examples/shoulder_freedom_60.json --database mps.sqlite --format markdown
```

Outputs an instructor-readable class plan.

### AC2 — JSON Output Works

```bash
mps generate examples/happy_hips_45.json --database mps.sqlite --format json
```

Outputs valid JSON matching the ClassPlan contract.

### AC3 — Unsupported Duration Fails

Input duration `50` returns a validation error naming supported durations.

### AC4 — Safety Exclusion Works

If group safety includes `wrist_pain`, wrist-loading hard-excluded exercises are not selected and appear in safety summary when excluded.

### AC5 — Seven Phases Always Present

Every generated ClassPlan includes all seven Movement Journey phases.

### AC6 — BUILD Contains Prime Exercises

BUILD includes 2–4 exercises with role `Prime`.

### AC7 — Strategy Emphasis Totals 100

Final movement strategy emphasis sums to exactly 100.

### AC8 — Markdown Contains Retest

Markdown output includes before benchmark and after-class retest.

## Future Extensions

- Axum HTTP API
- Web UI
- Postgres repository
- pgvector semantic exercise search
- Per-student safety profiles
- Instructor override logs
- Saved class plan history
- Optional LLM text enhancer
- Admin exercise editor

## Risks / Open Questions

1. Exercise metadata quality will determine class quality.
2. Safety tags require domain expert review.
3. SQLite seed data must be rich enough to avoid repetitive plans.
4. Challenge rules need careful tuning so challenge does not mean unsafe difficulty.
5. Markdown output should remain concise enough for instructors to use in real classes.
