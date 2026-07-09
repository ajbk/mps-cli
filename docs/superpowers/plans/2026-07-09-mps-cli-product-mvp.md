# MPS CLI Product MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI MVP for Memellow Programming System (MPS) that generates deterministic Total Body Pilates class plans from JSON requests and SQLite exercise data, with JSON and Markdown output.

**Architecture:** The system is a Rust workspace with clear crate boundaries. `mps-core` owns deterministic domain models, validation, strategy generation, scoring, class generation, and rendering contracts. `mps-db` owns SQLite migrations, seed data execution, and repository implementations. `mps-cli` owns command parsing, file IO, database path handling, and output format selection.

**Tech Stack:** Rust 2021, Cargo workspace, `serde`, `serde_json`, `thiserror`, `clap`, `sqlx` with SQLite, `tokio`, `insta` optional for snapshots, Markdown string rendering.

## Global Constraints

- MVP is CLI-first and API-later.
- Core generation must be deterministic, explainable, and fully testable.
- `mps-core` must not depend on SQLite, CLI, HTTP, or LLM providers.
- SQLite is the MVP database.
- Supported Movement Experiences: `ShoulderFreedom`, `HappyHips`, `SpineReset`.
- Supported primary apparatus: `Reformer`, `Chair`.
- Supported movement contexts: `Mat`, `Standing`.
- Supported durations: `45`, `60`, `75` minutes only.
- Supported levels: `Beginner`, `BeginnerIntermediate`, `Intermediate`, `IntermediateAdvanced`, `Advanced`.
- Every generated class must include all seven phases: `ARRIVE`, `PREPARE`, `BUILD`, `INTEGRATE`, `CHALLENGE`, `TRANSFER`, `RESET_RETEST`.
- BUILD must contain 2–4 Prime exercises.
- Final movement strategy emphasis must sum to exactly 100.
- MVP safety input is group-level; per-student safety profiles are future scope.
- LLM integration is out of implementation scope for this MVP. Future LLM enhancement may polish text only and must not alter canonical `ClassPlan` decisions.

---

## File Structure

Create this workspace:

```text
mps/
  Cargo.toml
  crates/
    mps-core/
      Cargo.toml
      src/
        lib.rs
        error.rs
        class_request.rs
        class_plan.rs
        domain.rs
        phase_allocation.rs
        repository.rs
        strategy.rs
        scoring.rs
        generator.rs
        markdown.rs
    mps-db/
      Cargo.toml
      migrations/
        0001_initial_schema.sql
      src/
        lib.rs
        sqlite_repository.rs
        seed.rs
    mps-cli/
      Cargo.toml
      src/
        main.rs
  data/
    seed/
      mps_seed.sql
  examples/
    shoulder_freedom_60.json
    happy_hips_45.json
    spine_reset_75.json
  tests/
    acceptance_cli.rs
```

Responsibilities:

- `mps-core/src/domain.rs`: shared enums and value types.
- `mps-core/src/class_request.rs`: input contract and validation.
- `mps-core/src/class_plan.rs`: canonical output structs.
- `mps-core/src/phase_allocation.rs`: deterministic duration templates.
- `mps-core/src/repository.rs`: repository traits consumed by core.
- `mps-core/src/strategy.rs`: base strategy + observation modifier application.
- `mps-core/src/scoring.rs`: candidate scoring and exclusion logic.
- `mps-core/src/generator.rs`: orchestration of full ClassPlan generation.
- `mps-core/src/markdown.rs`: Markdown renderer from `ClassPlan`.
- `mps-db/src/sqlite_repository.rs`: SQLite implementation of repository traits.
- `mps-db/src/seed.rs`: migration and seed helpers.
- `mps-cli/src/main.rs`: CLI commands.

---

### Task 1: Scaffold Rust Workspace

**Files:**
- Create: `Cargo.toml`
- Create: `crates/mps-core/Cargo.toml`
- Create: `crates/mps-core/src/lib.rs`
- Create: `crates/mps-db/Cargo.toml`
- Create: `crates/mps-db/src/lib.rs`
- Create: `crates/mps-cli/Cargo.toml`
- Create: `crates/mps-cli/src/main.rs`

**Interfaces:**
- Produces: a compiling Cargo workspace with three crates.
- Consumes: none.

- [ ] **Step 1: Create workspace manifest**

Write `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/mps-core",
  "crates/mps-db",
  "crates/mps-cli",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "UNLICENSED"

[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "migrate"] }
```

- [ ] **Step 2: Create `mps-core` manifest**

Write `crates/mps-core/Cargo.toml`:

```toml
[package]
name = "mps-core"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
```

- [ ] **Step 3: Create `mps-db` manifest**

Write `crates/mps-db/Cargo.toml`:

```toml
[package]
name = "mps-db"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
mps-core = { path = "../mps-core" }
anyhow.workspace = true
sqlx.workspace = true
tokio.workspace = true
```

- [ ] **Step 4: Create `mps-cli` manifest**

Write `crates/mps-cli/Cargo.toml`:

```toml
[package]
name = "mps-cli"
edition.workspace = true
version.workspace = true
license.workspace = true

[[bin]]
name = "mps"
path = "src/main.rs"

[dependencies]
mps-core = { path = "../mps-core" }
mps-db = { path = "../mps-db" }
anyhow.workspace = true
clap.workspace = true
serde_json.workspace = true
tokio.workspace = true
```

- [ ] **Step 5: Add minimal source files**

Write `crates/mps-core/src/lib.rs`:

```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

Write `crates/mps-db/src/lib.rs`:

```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

Write `crates/mps-cli/src/main.rs`:

```rust
fn main() {
    println!("mps {}", mps_core::version());
}
```

- [ ] **Step 6: Run build**

Run:

```bash
cargo check
```

Expected: workspace compiles.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates
 git commit -m "chore: scaffold MPS Rust workspace"
```

---

### Task 2: Define Core Domain Types and Errors

**Files:**
- Create: `crates/mps-core/src/error.rs`
- Create: `crates/mps-core/src/domain.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: enums used across request validation, strategy generation, scoring, and output.
- Consumes: workspace from Task 1.

- [ ] **Step 1: Write error type**

Create `crates/mps-core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MpsError {
    #[error("Unsupported movement experience: {0}")]
    UnsupportedMovementExperience(String),

    #[error("Unsupported level: {0}")]
    UnsupportedLevel(String),

    #[error("Unsupported duration: {requested}. MVP supports 45, 60, and 75 minutes.")]
    UnsupportedDuration { requested: u32 },

    #[error("Unsupported equipment: {0}")]
    UnsupportedEquipment(String),

    #[error("Unsupported risk policy: {0}")]
    UnsupportedRiskPolicy(String),

    #[error("No candidates available for phase {phase} and role {role}")]
    NoCandidates { phase: String, role: String },

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Generation failed: {0}")]
    Generation(String),
}

pub type MpsResult<T> = Result<T, MpsError>;
```

- [ ] **Step 2: Write domain enums**

Create `crates/mps-core/src/domain.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementExperience {
    ShoulderFreedom,
    HappyHips,
    SpineReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementSystem {
    BreathCore,
    Spine,
    Shoulder,
    Hip,
    Legs,
    Balance,
    Thoracic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ClassLevel {
    Beginner,
    BeginnerIntermediate,
    Intermediate,
    IntermediateAdvanced,
    Advanced,
}

impl ClassLevel {
    pub fn numeric(self) -> u8 {
        match self {
            Self::Beginner => 1,
            Self::BeginnerIntermediate => 2,
            Self::Intermediate => 3,
            Self::IntermediateAdvanced => 4,
            Self::Advanced => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Equipment {
    Reformer,
    Chair,
    Mat,
    Standing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentCategory {
    Apparatus,
    MovementContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementJourneyPhase {
    Arrive,
    Prepare,
    Build,
    Integrate,
    Challenge,
    Transfer,
    ResetRetest,
}

impl MovementJourneyPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Arrive => "ARRIVE",
            Self::Prepare => "PREPARE",
            Self::Build => "BUILD",
            Self::Integrate => "INTEGRATE",
            Self::Challenge => "CHALLENGE",
            Self::Transfer => "TRANSFER",
            Self::ResetRetest => "RESET & RETEST",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExerciseRole {
    Assess,
    Prepare,
    Prime,
    Integrate,
    Challenge,
    Transfer,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskPolicy {
    Conservative,
    Balanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetySeverity {
    HardExclude,
    Caution,
    RequireRegression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChallengeType {
    Balance,
    Coordination,
    Rotation,
    Standing,
    Complexity,
    Range,
    Load,
    Tempo,
}
```

- [ ] **Step 3: Export modules**

Update `crates/mps-core/src/lib.rs`:

```rust
pub mod domain;
pub mod error;

pub use domain::*;
pub use error::{MpsError, MpsResult};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 4: Run tests/build**

Run:

```bash
cargo check -p mps-core
```

Expected: `mps-core` compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: add MPS core domain types"
```

---

### Task 3: Implement ClassRequest Validation

**Files:**
- Create: `crates/mps-core/src/class_request.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `ClassRequest`, `GroupSafety`, `ClassRequest::validate()`.
- Consumes: domain enums and `MpsError` from Task 2.

- [ ] **Step 1: Write failing unit tests**

Create `crates/mps-core/src/class_request.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Equipment, MovementExperience, ClassLevel, RiskPolicy};

    fn valid_request() -> ClassRequest {
        ClassRequest {
            students: 4,
            movement_experience: MovementExperience::ShoulderFreedom,
            level: ClassLevel::BeginnerIntermediate,
            equipment: vec![Equipment::Reformer, Equipment::Chair],
            duration_minutes: 60,
            observations: vec!["thoracic stiffness".to_string()],
            group_safety: GroupSafety {
                contraindications: vec!["wrist_pain".to_string()],
                risk_policy: RiskPolicy::Conservative,
            },
        }
    }

    #[test]
    fn accepts_valid_request() {
        assert!(valid_request().validate().is_ok());
    }

    #[test]
    fn rejects_unsupported_duration() {
        let mut request = valid_request();
        request.duration_minutes = 50;
        assert!(request.validate().is_err());
    }

    #[test]
    fn rejects_movement_context_as_primary_equipment() {
        let mut request = valid_request();
        request.equipment = vec![Equipment::Standing];
        assert!(request.validate().is_err());
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p mps-core class_request -- --nocapture
```

Expected: compile failure because `ClassRequest` is not defined.

- [ ] **Step 3: Implement request structs and validation**

Replace `crates/mps-core/src/class_request.rs` with:

```rust
use serde::{Deserialize, Serialize};

use crate::{ClassLevel, Equipment, MovementExperience, MpsError, MpsResult, RiskPolicy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassRequest {
    pub students: u32,
    pub movement_experience: MovementExperience,
    pub level: ClassLevel,
    pub equipment: Vec<Equipment>,
    pub duration_minutes: u32,
    pub observations: Vec<String>,
    pub group_safety: GroupSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSafety {
    pub contraindications: Vec<String>,
    pub risk_policy: RiskPolicy,
}

impl ClassRequest {
    pub fn validate(&self) -> MpsResult<()> {
        match self.duration_minutes {
            45 | 60 | 75 => {}
            requested => return Err(MpsError::UnsupportedDuration { requested }),
        }

        if self.equipment.is_empty() {
            return Err(MpsError::UnsupportedEquipment("at least one primary apparatus is required".to_string()));
        }

        for item in &self.equipment {
            match item {
                Equipment::Reformer | Equipment::Chair => {}
                Equipment::Mat | Equipment::Standing => {
                    return Err(MpsError::UnsupportedEquipment(format!(
                        "{:?} is a movement context, not primary apparatus",
                        item
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Equipment, MovementExperience, ClassLevel, RiskPolicy};

    fn valid_request() -> ClassRequest {
        ClassRequest {
            students: 4,
            movement_experience: MovementExperience::ShoulderFreedom,
            level: ClassLevel::BeginnerIntermediate,
            equipment: vec![Equipment::Reformer, Equipment::Chair],
            duration_minutes: 60,
            observations: vec!["thoracic stiffness".to_string()],
            group_safety: GroupSafety {
                contraindications: vec!["wrist_pain".to_string()],
                risk_policy: RiskPolicy::Conservative,
            },
        }
    }

    #[test]
    fn accepts_valid_request() {
        assert!(valid_request().validate().is_ok());
    }

    #[test]
    fn rejects_unsupported_duration() {
        let mut request = valid_request();
        request.duration_minutes = 50;
        assert!(request.validate().is_err());
    }

    #[test]
    fn rejects_movement_context_as_primary_equipment() {
        let mut request = valid_request();
        request.equipment = vec![Equipment::Standing];
        assert!(request.validate().is_err());
    }
}
```

- [ ] **Step 4: Export module**

Update `crates/mps-core/src/lib.rs`:

```rust
pub mod class_request;
pub mod domain;
pub mod error;

pub use class_request::{ClassRequest, GroupSafety};
pub use domain::*;
pub use error::{MpsError, MpsResult};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p mps-core class_request
```

Expected: all request validation tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: validate class request input"
```

---

### Task 4: Implement Phase Allocation

**Files:**
- Create: `crates/mps-core/src/phase_allocation.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `PhaseAllocation`, `allocate_phases(duration_minutes)`.
- Consumes: `MpsError`.

- [ ] **Step 1: Write phase allocation tests**

Create `crates/mps-core/src/phase_allocation.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_60_minutes() {
        let allocation = allocate_phases(60).unwrap();
        assert_eq!(allocation.total(), 60);
        assert_eq!(allocation.build, 20);
        assert_eq!(allocation.reset_retest, 3);
    }

    #[test]
    fn rejects_unsupported_duration() {
        assert!(allocate_phases(50).is_err());
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p mps-core phase_allocation
```

Expected: compile failure because `allocate_phases` is not defined.

- [ ] **Step 3: Implement allocation**

Replace file with:

```rust
use serde::{Deserialize, Serialize};

use crate::{MpsError, MpsResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseAllocation {
    pub arrive: u32,
    pub prepare: u32,
    pub build: u32,
    pub integrate: u32,
    pub challenge: u32,
    pub transfer: u32,
    pub reset_retest: u32,
}

impl PhaseAllocation {
    pub fn total(self) -> u32 {
        self.arrive
            + self.prepare
            + self.build
            + self.integrate
            + self.challenge
            + self.transfer
            + self.reset_retest
    }
}

pub fn allocate_phases(duration_minutes: u32) -> MpsResult<PhaseAllocation> {
    match duration_minutes {
        45 => Ok(PhaseAllocation {
            arrive: 4,
            prepare: 7,
            build: 15,
            integrate: 7,
            challenge: 6,
            transfer: 3,
            reset_retest: 3,
        }),
        60 => Ok(PhaseAllocation {
            arrive: 5,
            prepare: 10,
            build: 20,
            integrate: 10,
            challenge: 8,
            transfer: 4,
            reset_retest: 3,
        }),
        75 => Ok(PhaseAllocation {
            arrive: 6,
            prepare: 12,
            build: 26,
            integrate: 13,
            challenge: 10,
            transfer: 5,
            reset_retest: 3,
        }),
        requested => Err(MpsError::UnsupportedDuration { requested }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_60_minutes() {
        let allocation = allocate_phases(60).unwrap();
        assert_eq!(allocation.total(), 60);
        assert_eq!(allocation.build, 20);
        assert_eq!(allocation.reset_retest, 3);
    }

    #[test]
    fn rejects_unsupported_duration() {
        assert!(allocate_phases(50).is_err());
    }
}
```

- [ ] **Step 4: Export module**

Update `crates/mps-core/src/lib.rs` to include:

```rust
pub mod phase_allocation;
pub use phase_allocation::{allocate_phases, PhaseAllocation};
```

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p mps-core phase_allocation
```

Expected: tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: add deterministic phase allocation"
```

---

### Task 5: Define ClassPlan Output Structs and Markdown Renderer

**Files:**
- Create: `crates/mps-core/src/class_plan.rs`
- Create: `crates/mps-core/src/markdown.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `ClassPlan`, `JourneyPhasePlan`, `ExerciseTeachingUnit`, `render_markdown(&ClassPlan)`.
- Consumes: domain enums.

- [ ] **Step 1: Write ClassPlan structs**

Create `crates/mps-core/src/class_plan.rs`:

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ClassLevel, Equipment, ExerciseRole, MovementExperience, MovementJourneyPhase, MovementSystem, RiskPolicy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassPlan {
    pub class_title: String,
    pub movement_experience: MovementExperience,
    pub duration_minutes: u32,
    pub level: ClassLevel,
    pub students: u32,
    pub equipment: Vec<Equipment>,
    pub movement_strategy: MovementStrategyPlan,
    pub benchmark: BenchmarkPlan,
    pub journey: Vec<JourneyPhasePlan>,
    pub safety_summary: SafetySummary,
    pub retest: RetestPlan,
    pub expected_improvement: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementStrategyPlan {
    pub primary_focus: MovementSystem,
    pub secondary_focus: MovementSystem,
    pub emphasis: BTreeMap<String, u32>,
    pub key_objectives: Vec<String>,
    pub preferred_exercise_objectives: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkPlan {
    pub assessments: Vec<AssessmentPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentPlan {
    pub name: String,
    pub instruction: String,
    pub what_to_watch: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyPhasePlan {
    pub phase: MovementJourneyPhase,
    pub purpose: String,
    pub target_duration_minutes: u32,
    pub exercises: Vec<ExerciseTeachingUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseTeachingUnit {
    pub exercise_id: String,
    pub name: String,
    pub apparatus: Equipment,
    pub role: ExerciseRole,
    pub duration_minutes: u32,
    pub movement_objectives: Vec<String>,
    pub why_selected: String,
    pub teaching_cues: Vec<String>,
    pub regression: Option<String>,
    pub progression: Option<String>,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySummary {
    pub risk_policy: RiskPolicy,
    pub applied_contraindications: Vec<String>,
    pub excluded_exercises: Vec<SafetyExerciseNote>,
    pub modified_exercises: Vec<SafetyExerciseNote>,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyExerciseNote {
    pub exercise_id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetestPlan {
    pub assessments: Vec<AssessmentPlan>,
    pub expected_improvement: String,
}
```

- [ ] **Step 2: Write Markdown renderer**

Create `crates/mps-core/src/markdown.rs`:

```rust
use crate::ClassPlan;

pub fn render_markdown(plan: &ClassPlan) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n", plan.class_title));
    out.push_str("## Total Body Pilates Class\n\n");
    out.push_str(&format!("**Duration:** {} min  \n", plan.duration_minutes));
    out.push_str(&format!("**Level:** {:?}  \n", plan.level));
    out.push_str(&format!("**Students:** {}  \n", plan.students));
    out.push_str(&format!("**Equipment:** {:?}\n\n", plan.equipment));

    out.push_str("---\n\n## Movement Strategy\n\n");
    out.push_str(&format!("**Primary Focus:** {:?}  \n", plan.movement_strategy.primary_focus));
    out.push_str(&format!("**Secondary Focus:** {:?}\n\n", plan.movement_strategy.secondary_focus));
    out.push_str("| System | Emphasis |\n|---|---:|\n");
    for (system, value) in &plan.movement_strategy.emphasis {
        out.push_str(&format!("| {} | {}% |\n", system, value));
    }
    out.push_str("\n**Why this strategy:**  \n");
    out.push_str(&plan.movement_strategy.explanation);
    out.push_str("\n\n---\n\n## Before Class Benchmark\n\n");

    for assessment in &plan.benchmark.assessments {
        out.push_str(&format!("### {}\n\n{}\n\n", assessment.name, assessment.instruction));
        out.push_str("**Watch for:**\n");
        for point in &assessment.what_to_watch {
            out.push_str(&format!("- {}\n", point));
        }
        out.push('\n');
    }

    out.push_str("---\n\n## Class Journey\n\n");
    for phase in &plan.journey {
        out.push_str(&format!("### {} — {} min\n\n", phase.phase.label(), phase.target_duration_minutes));
        out.push_str(&format!("{}\n\n", phase.purpose));
        for exercise in &phase.exercises {
            out.push_str(&format!(
                "**{}** | {:?} | {} min | Role: {:?}\n\n",
                exercise.name, exercise.apparatus, exercise.duration_minutes, exercise.role
            ));
            out.push_str(&format!("- Objectives: {}\n", exercise.movement_objectives.join(", ")));
            out.push_str(&format!("- Why selected: {}\n", exercise.why_selected));
            out.push_str("- Cues:\n");
            for cue in &exercise.teaching_cues {
                out.push_str(&format!("  - {}\n", cue));
            }
            if let Some(regression) = &exercise.regression {
                out.push_str(&format!("- Regression: {}\n", regression));
            }
            if let Some(progression) = &exercise.progression {
                out.push_str(&format!("- Progression: {}\n", progression));
            }
            for note in &exercise.safety_notes {
                out.push_str(&format!("- Safety: {}\n", note));
            }
            out.push('\n');
        }
    }

    out.push_str("---\n\n## Safety Notes\n\n");
    out.push_str(&format!("**Risk Policy:** {:?}\n\n", plan.safety_summary.risk_policy));
    if !plan.safety_summary.applied_contraindications.is_empty() {
        out.push_str("**Applied group contraindications:**\n");
        for item in &plan.safety_summary.applied_contraindications {
            out.push_str(&format!("- {}\n", item));
        }
        out.push('\n');
    }
    for note in &plan.safety_summary.safety_notes {
        out.push_str(&format!("- {}\n", note));
    }

    out.push_str("\n---\n\n## After Class Retest\n\n");
    for assessment in &plan.retest.assessments {
        out.push_str(&format!("### {}\n\n{}\n\n", assessment.name, assessment.instruction));
    }
    out.push_str(&format!("**Expected improvement:**  \n{}\n", plan.expected_improvement));

    out
}
```

- [ ] **Step 3: Export modules**

Update `crates/mps-core/src/lib.rs` to include:

```rust
pub mod class_plan;
pub mod markdown;

pub use class_plan::*;
pub use markdown::render_markdown;
```

- [ ] **Step 4: Run build**

Run:

```bash
cargo check -p mps-core
```

Expected: compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: define class plan output and markdown renderer"
```

---

### Task 6: Add Repository Traits and Data Transfer Types

**Files:**
- Create: `crates/mps-core/src/repository.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `MpsRepository` trait and DB-independent records.
- Consumes: domain types.

- [ ] **Step 1: Write repository interfaces**

Create `crates/mps-core/src/repository.rs`:

```rust
use std::collections::BTreeMap;

use crate::{ClassLevel, Equipment, ExerciseRole, MovementExperience, MovementSystem, SafetySeverity};

#[derive(Debug, Clone)]
pub struct BaseStrategyRecord {
    pub movement_experience: MovementExperience,
    pub primary_focus: MovementSystem,
    pub secondary_focus: MovementSystem,
    pub emphasis: BTreeMap<String, i32>,
    pub objectives: Vec<String>,
    pub explanation_template: String,
}

#[derive(Debug, Clone)]
pub struct ObservationModifierRecord {
    pub id: String,
    pub emphasis_adjustments: BTreeMap<String, i32>,
    pub added_objectives: Vec<String>,
    pub preferred_exercise_objectives: Vec<String>,
    pub explanation_fragment: String,
}

#[derive(Debug, Clone)]
pub struct ExerciseRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub equipment: Equipment,
    pub roles: Vec<ExerciseRole>,
    pub objectives: Vec<String>,
    pub min_level: ClassLevel,
    pub max_level: ClassLevel,
    pub difficulty: u8,
    pub default_duration_minutes: u32,
    pub teaching_cues: Vec<String>,
    pub regression: Option<String>,
    pub progression: Option<String>,
    pub contraindications: Vec<ContraindicationRecord>,
}

#[derive(Debug, Clone)]
pub struct ContraindicationRecord {
    pub tag: String,
    pub severity: SafetySeverity,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct BenchmarkRecord {
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub watch_points: Vec<String>,
}

pub trait MpsRepository {
    fn base_strategy(&self, experience: MovementExperience) -> crate::MpsResult<BaseStrategyRecord>;
    fn observation_modifiers(&self, observations: &[String]) -> crate::MpsResult<Vec<ObservationModifierRecord>>;
    fn benchmarks(&self, experience: MovementExperience) -> crate::MpsResult<Vec<BenchmarkRecord>>;
    fn exercises(&self) -> crate::MpsResult<Vec<ExerciseRecord>>;
}
```

- [ ] **Step 2: Export module**

Update `crates/mps-core/src/lib.rs` to include:

```rust
pub mod repository;
pub use repository::*;
```

- [ ] **Step 3: Run build**

Run:

```bash
cargo check -p mps-core
```

Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: define repository boundary for core engine"
```

---

### Task 7: Implement Strategy Generation

**Files:**
- Create: `crates/mps-core/src/strategy.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `build_strategy(base, modifiers)`.
- Consumes: `BaseStrategyRecord`, `ObservationModifierRecord`, `MovementStrategyPlan`.

- [ ] **Step 1: Write strategy tests**

Create `crates/mps-core/src/strategy.rs` with tests that verify emphasis normalization:

```rust
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use super::*;
    use crate::{BaseStrategyRecord, MovementExperience, MovementSystem, ObservationModifierRecord};

    #[test]
    fn applies_modifiers_and_normalizes_to_100() {
        let base = BaseStrategyRecord {
            movement_experience: MovementExperience::ShoulderFreedom,
            primary_focus: MovementSystem::Shoulder,
            secondary_focus: MovementSystem::Thoracic,
            emphasis: BTreeMap::from([
                ("Shoulder".to_string(), 40),
                ("Thoracic".to_string(), 20),
                ("Core".to_string(), 15),
                ("Hip".to_string(), 10),
                ("Legs".to_string(), 10),
                ("Balance".to_string(), 5),
            ]),
            objectives: vec!["Improve shoulder mobility".to_string()],
            explanation_template: "Base explanation.".to_string(),
        };
        let modifier = ObservationModifierRecord {
            id: "thoracic_stiffness".to_string(),
            emphasis_adjustments: BTreeMap::from([
                ("Thoracic".to_string(), 5),
                ("Shoulder".to_string(), -3),
            ]),
            added_objectives: vec!["Restore thoracic extension".to_string()],
            preferred_exercise_objectives: vec!["Thoracic Mobility".to_string()],
            explanation_fragment: "Thoracic stiffness increases thoracic emphasis.".to_string(),
        };

        let strategy = build_strategy(base, vec![modifier]);
        let total: u32 = strategy.emphasis.values().sum();
        assert_eq!(total, 100);
        assert!(strategy.key_objectives.contains(&"Restore thoracic extension".to_string()));
    }
}
```

- [ ] **Step 2: Run failing test**

Run:

```bash
cargo test -p mps-core strategy
```

Expected: compile failure because `build_strategy` is missing.

- [ ] **Step 3: Implement strategy generation**

Add implementation above tests:

```rust
use std::collections::BTreeMap;

use crate::{BaseStrategyRecord, MovementStrategyPlan, ObservationModifierRecord};

pub fn build_strategy(
    base: BaseStrategyRecord,
    modifiers: Vec<ObservationModifierRecord>,
) -> MovementStrategyPlan {
    let mut emphasis = base.emphasis.clone();
    let mut objectives = base.objectives.clone();
    let mut preferred = Vec::new();
    let mut explanation_parts = vec![base.explanation_template.clone()];

    for modifier in modifiers {
        for (system, adjustment) in modifier.emphasis_adjustments {
            *emphasis.entry(system).or_insert(0) += adjustment;
        }
        for objective in modifier.added_objectives {
            if !objectives.contains(&objective) {
                objectives.push(objective);
            }
        }
        for objective in modifier.preferred_exercise_objectives {
            if !preferred.contains(&objective) {
                preferred.push(objective);
            }
        }
        explanation_parts.push(modifier.explanation_fragment);
    }

    let normalized = normalize_to_100(emphasis);

    MovementStrategyPlan {
        primary_focus: base.primary_focus,
        secondary_focus: base.secondary_focus,
        emphasis: normalized,
        key_objectives: objectives,
        preferred_exercise_objectives: preferred,
        explanation: explanation_parts.join(" "),
    }
}

fn normalize_to_100(input: BTreeMap<String, i32>) -> BTreeMap<String, u32> {
    let clipped: Vec<(String, i32)> = input
        .into_iter()
        .map(|(key, value)| (key, value.max(0)))
        .collect();
    let sum: i32 = clipped.iter().map(|(_, value)| *value).sum();

    if sum <= 0 {
        return BTreeMap::new();
    }

    let mut normalized = BTreeMap::new();
    let mut running_total = 0u32;
    let len = clipped.len();

    for (idx, (key, value)) in clipped.into_iter().enumerate() {
        let pct = if idx + 1 == len {
            100 - running_total
        } else {
            ((value as f64 / sum as f64) * 100.0).round() as u32
        };
        running_total += pct;
        normalized.insert(key, pct);
    }

    normalized
}
```

- [ ] **Step 4: Export module**

Update `crates/mps-core/src/lib.rs` to include:

```rust
pub mod strategy;
pub use strategy::build_strategy;
```

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p mps-core strategy
```

Expected: tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: generate movement strategy from observation modifiers"
```

---

### Task 8: Implement Scoring and Safety Filtering

**Files:**
- Create: `crates/mps-core/src/scoring.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `score_candidate`, `filter_safe_candidates`.
- Consumes: `ExerciseRecord`, `ClassRequest`, `MovementStrategyPlan`.

- [ ] **Step 1: Write scoring tests**

Create `crates/mps-core/src/scoring.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClassLevel, ExerciseRole};

    #[test]
    fn exact_level_scores_higher_than_too_easy() {
        assert!(difficulty_fit_score(2, ClassLevel::BeginnerIntermediate) > difficulty_fit_score(1, ClassLevel::BeginnerIntermediate));
    }

    #[test]
    fn too_hard_scores_zero() {
        assert_eq!(difficulty_fit_score(5, ClassLevel::BeginnerIntermediate), 0);
    }

    #[test]
    fn role_match_scores_positive() {
        assert_eq!(role_match_score(&[ExerciseRole::Prime], ExerciseRole::Prime), 20);
    }
}
```

- [ ] **Step 2: Run failing test**

Run:

```bash
cargo test -p mps-core scoring
```

Expected: compile failure because functions are missing.

- [ ] **Step 3: Implement scoring helpers**

Replace file with:

```rust
use crate::{ClassLevel, ExerciseRole};

pub fn difficulty_fit_score(exercise_difficulty: u8, class_level: ClassLevel) -> i32 {
    let diff = exercise_difficulty as i8;
    let level = class_level.numeric() as i8;

    match diff - level {
        ..=-1 => 3,
        0 => 5,
        1 => 2,
        _ => 0,
    }
}

pub fn role_match_score(roles: &[ExerciseRole], target_role: ExerciseRole) -> i32 {
    if roles.contains(&target_role) { 20 } else { 0 }
}

pub fn objective_match_score(exercise_objectives: &[String], preferred: &[String]) -> i32 {
    exercise_objectives
        .iter()
        .filter(|objective| preferred.iter().any(|wanted| wanted.eq_ignore_ascii_case(objective)))
        .count() as i32
        * 5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClassLevel, ExerciseRole};

    #[test]
    fn exact_level_scores_higher_than_too_easy() {
        assert!(difficulty_fit_score(2, ClassLevel::BeginnerIntermediate) > difficulty_fit_score(1, ClassLevel::BeginnerIntermediate));
    }

    #[test]
    fn too_hard_scores_zero() {
        assert_eq!(difficulty_fit_score(5, ClassLevel::BeginnerIntermediate), 0);
    }

    #[test]
    fn role_match_scores_positive() {
        assert_eq!(role_match_score(&[ExerciseRole::Prime], ExerciseRole::Prime), 20);
    }
}
```

- [ ] **Step 4: Export module**

Update `crates/mps-core/src/lib.rs` to include:

```rust
pub mod scoring;
pub use scoring::*;
```

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p mps-core scoring
```

Expected: tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: add exercise scoring helpers"
```

---

### Task 9: Add SQLite Schema and Seed Data

**Files:**
- Create: `crates/mps-db/migrations/0001_initial_schema.sql`
- Create: `data/seed/mps_seed.sql`

**Interfaces:**
- Produces: SQLite schema and seed data for all MVP experiences.
- Consumes: PRD schema.

- [ ] **Step 1: Write migration file**

Create `crates/mps-db/migrations/0001_initial_schema.sql` using the schema from the PRD. Include all tables listed in the PRD database section.

- [ ] **Step 2: Write seed data**

Create `data/seed/mps_seed.sql` with at least:

```sql
INSERT INTO movement_experiences (id, name) VALUES
('ShoulderFreedom', 'Shoulder Freedom'),
('HappyHips', 'Happy Hips'),
('SpineReset', 'Spine Reset');

INSERT INTO equipment (id, name, category) VALUES
('Reformer', 'Reformer', 'apparatus'),
('Chair', 'Chair', 'apparatus'),
('Mat', 'Mat', 'movement_context'),
('Standing', 'Standing', 'movement_context');
```

Then add base strategies, observation modifiers, benchmarks, and 20–30 exercises. The first minimum pass must include these exercises:

```text
Footwork
Pelvic Curl
Feet in Straps
Arms in Straps
Pulling Straps
Short Spine Prep
Elephant
Long Stretch Prep
Eve's Lunge
Mermaid
Seated Push Down
Standing Push Down
Swan on Chair
Mermaid on Chair
Step Up Prep
Standing Leg Pump
Standing Roll Down
Arm Raise
Standing Rotation
Squat + Reach
Single Leg Balance
Hip Hinge
Standing Reach
Cat-Cow
```

- [ ] **Step 3: Validate SQL manually with sqlite**

Run:

```bash
sqlite3 mps.sqlite < crates/mps-db/migrations/0001_initial_schema.sql
sqlite3 mps.sqlite < data/seed/mps_seed.sql
sqlite3 mps.sqlite 'SELECT COUNT(*) FROM exercises;'
```

Expected: count is at least 20.

- [ ] **Step 4: Commit**

```bash
git add crates/mps-db/migrations data/seed
 git commit -m "feat: add SQLite schema and MVP seed data"
```

---

### Task 10: Implement SQLite Repository

**Files:**
- Create: `crates/mps-db/src/sqlite_repository.rs`
- Modify: `crates/mps-db/src/lib.rs`

**Interfaces:**
- Produces: `SqliteMpsRepository` implementing `MpsRepository`.
- Consumes: SQLite schema and `mps-core` repository trait.

- [ ] **Step 1: Implement repository skeleton**

Create `crates/mps-db/src/sqlite_repository.rs`:

```rust
use mps_core::*;
use sqlx::SqlitePool;

pub struct SqliteMpsRepository {
    pool: SqlitePool,
}

impl SqliteMpsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl MpsRepository for SqliteMpsRepository {
    fn base_strategy(&self, _experience: MovementExperience) -> MpsResult<BaseStrategyRecord> {
        Err(MpsError::Repository("base_strategy not implemented".to_string()))
    }

    fn observation_modifiers(&self, _observations: &[String]) -> MpsResult<Vec<ObservationModifierRecord>> {
        Err(MpsError::Repository("observation_modifiers not implemented".to_string()))
    }

    fn benchmarks(&self, _experience: MovementExperience) -> MpsResult<Vec<BenchmarkRecord>> {
        Err(MpsError::Repository("benchmarks not implemented".to_string()))
    }

    fn exercises(&self) -> MpsResult<Vec<ExerciseRecord>> {
        Err(MpsError::Repository("exercises not implemented".to_string()))
    }
}
```

- [ ] **Step 2: Export repository**

Update `crates/mps-db/src/lib.rs`:

```rust
pub mod sqlite_repository;

pub use sqlite_repository::SqliteMpsRepository;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 3: Replace skeleton methods with SQL queries**

Implement each method using `sqlx::query` and `futures`-free blocking boundaries by making repository async if necessary. If async is preferred, update the trait to use `async_trait` or make CLI orchestration call async DB helpers and convert into in-memory repository. Keep `mps-core` independent from `sqlx`.

Recommended simpler MVP approach: `mps-db` loads all DB data into an `InMemoryMpsRepository` from `mps-core` records, then passes that to the generator. Add this if direct sync trait implementation becomes awkward.

- [ ] **Step 4: Run build**

Run:

```bash
cargo check -p mps-db
```

Expected: compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/mps-db
 git commit -m "feat: implement SQLite repository boundary"
```

---

### Task 11: Implement Class Generator

**Files:**
- Create: `crates/mps-core/src/generator.rs`
- Modify: `crates/mps-core/src/lib.rs`

**Interfaces:**
- Produces: `generate_class_plan(request, repository)`.
- Consumes: request validation, repository traits, strategy, scoring, phase allocation, ClassPlan structs.

- [ ] **Step 1: Write generator acceptance-style unit test with fake repository**

Create a test in `generator.rs` that uses an in-memory fake repository and asserts:

```rust
assert_eq!(plan.journey.len(), 7);
assert_eq!(plan.duration_minutes, 60);
assert_eq!(plan.movement_strategy.emphasis.values().sum::<u32>(), 100);
assert!(plan.journey.iter().any(|phase| phase.phase == MovementJourneyPhase::Build));
```

- [ ] **Step 2: Implement generator orchestration**

Implement:

```rust
pub fn generate_class_plan<R: MpsRepository>(
    request: &ClassRequest,
    repository: &R,
) -> MpsResult<ClassPlan>
```

Generation order:

1. `request.validate()`
2. `allocate_phases(request.duration_minutes)`
3. `repository.base_strategy(request.movement_experience)`
4. `repository.observation_modifiers(&request.observations)`
5. `build_strategy(base, modifiers)`
6. `repository.benchmarks(request.movement_experience)`
7. `repository.exercises()`
8. select phase exercises by role and score
9. build `SafetySummary`
10. build `ClassPlan`

- [ ] **Step 3: Ensure seven phases always present**

Generator must always create all seven phases even if a phase has warnings due to limited candidates. If required exercises are missing for BUILD, return `MpsError::NoCandidates`.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test -p mps-core generator
```

Expected: generator tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/mps-core
 git commit -m "feat: generate deterministic class plans"
```

---

### Task 12: Implement CLI Commands

**Files:**
- Modify: `crates/mps-cli/src/main.rs`

**Interfaces:**
- Produces: `mps init-db`, `mps seed`, `mps generate`.
- Consumes: `mps-db`, `mps-core`.

- [ ] **Step 1: Implement clap command structure**

Use:

```rust
#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    InitDb { #[arg(long)] database: String },
    Seed { #[arg(long)] database: String, #[arg(long)] file: String },
    Generate { input: String, #[arg(long)] database: String, #[arg(long, default_value = "markdown")] format: String },
}
```

- [ ] **Step 2: Implement `init-db`**

Read migration SQL and execute it against SQLite database path.

- [ ] **Step 3: Implement `seed`**

Read seed SQL file and execute it.

- [ ] **Step 4: Implement `generate`**

Read input JSON into `ClassRequest`, load repository from SQLite, call `generate_class_plan`, and print either pretty JSON or Markdown.

- [ ] **Step 5: Run CLI manually**

Run:

```bash
cargo run -p mps-cli --bin mps -- init-db --database mps.sqlite
cargo run -p mps-cli --bin mps -- seed --database mps.sqlite --file data/seed/mps_seed.sql
```

Expected: commands succeed.

- [ ] **Step 6: Commit**

```bash
git add crates/mps-cli
 git commit -m "feat: add MPS CLI commands"
```

---

### Task 13: Add Example Requests

**Files:**
- Create: `examples/shoulder_freedom_60.json`
- Create: `examples/happy_hips_45.json`
- Create: `examples/spine_reset_75.json`

**Interfaces:**
- Produces: CLI demo inputs.
- Consumes: `ClassRequest` contract.

- [ ] **Step 1: Add Shoulder Freedom example**

Write `examples/shoulder_freedom_60.json`:

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

- [ ] **Step 2: Add Happy Hips example**

Write `examples/happy_hips_45.json`:

```json
{
  "students": 3,
  "movement_experience": "HappyHips",
  "level": "BeginnerIntermediate",
  "equipment": ["Reformer", "Chair"],
  "duration_minutes": 45,
  "observations": [
    "hip tightness",
    "limited squat depth"
  ],
  "group_safety": {
    "contraindications": ["knee_pain"],
    "risk_policy": "Conservative"
  }
}
```

- [ ] **Step 3: Add Spine Reset example**

Write `examples/spine_reset_75.json`:

```json
{
  "students": 5,
  "movement_experience": "SpineReset",
  "level": "Intermediate",
  "equipment": ["Reformer", "Chair"],
  "duration_minutes": 75,
  "observations": [
    "low back tension",
    "limited rotation",
    "weak core connection"
  ],
  "group_safety": {
    "contraindications": ["acute_low_back_pain"],
    "risk_policy": "Conservative"
  }
}
```

- [ ] **Step 4: Validate JSON files**

Run:

```bash
python -m json.tool examples/shoulder_freedom_60.json >/dev/null
python -m json.tool examples/happy_hips_45.json >/dev/null
python -m json.tool examples/spine_reset_75.json >/dev/null
```

Expected: all JSON files parse.

- [ ] **Step 5: Commit**

```bash
git add examples
 git commit -m "test: add MVP class request examples"
```

---

### Task 14: Add Acceptance Tests and Demo Verification

**Files:**
- Create: `tests/acceptance_cli.rs`
- Modify: root `Cargo.toml` if needed for dev dependencies.

**Interfaces:**
- Produces: verification that CLI demo works.
- Consumes: CLI commands and examples.

- [ ] **Step 1: Add CLI acceptance test strategy**

If direct Rust integration tests are too heavy initially, create a shell-based smoke test script instead:

```text
scripts/smoke-test.sh
```

The smoke test must run:

```bash
cargo run -p mps-cli --bin mps -- init-db --database /tmp/mps-test.sqlite
cargo run -p mps-cli --bin mps -- seed --database /tmp/mps-test.sqlite --file data/seed/mps_seed.sql
cargo run -p mps-cli --bin mps -- generate examples/shoulder_freedom_60.json --database /tmp/mps-test.sqlite --format markdown
cargo run -p mps-cli --bin mps -- generate examples/happy_hips_45.json --database /tmp/mps-test.sqlite --format json
```

- [ ] **Step 2: Assert Markdown contains required sections**

Smoke test must check output contains:

```text
Movement Strategy
Before Class Benchmark
Class Journey
Safety Notes
After Class Retest
```

- [ ] **Step 3: Assert JSON parses**

Pipe JSON output into:

```bash
python -m json.tool
```

- [ ] **Step 4: Run smoke test**

Run:

```bash
bash scripts/smoke-test.sh
```

Expected: script exits 0.

- [ ] **Step 5: Commit**

```bash
git add tests scripts Cargo.toml
 git commit -m "test: add CLI acceptance smoke test"
```

---

### Task 15: Final Verification and Documentation

**Files:**
- Create or modify: `README.md`

**Interfaces:**
- Produces: user-facing instructions and verified MVP demo commands.
- Consumes: all previous tasks.

- [ ] **Step 1: Write README quickstart**

Add:

```markdown
# MPS CLI MVP

## Quickstart

```bash
cargo run -p mps-cli --bin mps -- init-db --database mps.sqlite
cargo run -p mps-cli --bin mps -- seed --database mps.sqlite --file data/seed/mps_seed.sql
cargo run -p mps-cli --bin mps -- generate examples/shoulder_freedom_60.json --database mps.sqlite --format markdown
```
```

- [ ] **Step 2: Run full test suite**

Run:

```bash
cargo test
bash scripts/smoke-test.sh
```

Expected: all tests and smoke tests pass.

- [ ] **Step 3: Capture demo output**

Run:

```bash
cargo run -p mps-cli --bin mps -- generate examples/shoulder_freedom_60.json --database mps.sqlite --format markdown > /tmp/mps-shoulder-demo.md
```

Expected: `/tmp/mps-shoulder-demo.md` contains an instructor-readable class plan.

- [ ] **Step 4: Commit final docs**

```bash
git add README.md
 git commit -m "docs: add MPS CLI MVP quickstart"
```

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-09-mps-cli-product-mvp.md`.

Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
