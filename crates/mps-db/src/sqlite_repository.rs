use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use mps_core::{
    BaseStrategyRecord, BenchmarkRecord, ClassLevel, ContraindicationRecord, Equipment,
    ExerciseRecord, ExerciseRole, MovementExperience, MovementSystem, MpsError, MpsRepository,
    MpsResult, ObservationModifierRecord, SafetySeverity,
};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, SqlitePool};
use tokio::runtime::Runtime;

// ── helpers: enum ↔ DB string ──────────────────────────────────────

fn experience_to_db(exp: MovementExperience) -> &'static str {
    match exp {
        MovementExperience::ShoulderFreedom => "shoulder_freedom",
        MovementExperience::HappyHips => "happy_hips",
        MovementExperience::SpineReset => "spine_reset",
    }
}

fn experience_from_db(s: &str) -> MpsResult<MovementExperience> {
    match s {
        "shoulder_freedom" => Ok(MovementExperience::ShoulderFreedom),
        "happy_hips" => Ok(MovementExperience::HappyHips),
        "spine_reset" => Ok(MovementExperience::SpineReset),
        other => Err(MpsError::UnsupportedMovementExperience(other.to_string())),
    }
}

fn system_from_db(s: &str) -> MpsResult<MovementSystem> {
    match s {
        "Shoulder" => Ok(MovementSystem::Shoulder),
        "Thoracic" => Ok(MovementSystem::Thoracic),
        "Hip" => Ok(MovementSystem::Hip),
        "Legs" => Ok(MovementSystem::Legs),
        "Spine" => Ok(MovementSystem::Spine),
        "BreathCore" => Ok(MovementSystem::BreathCore),
        "Balance" => Ok(MovementSystem::Balance),
        other => Err(MpsError::Generation(format!(
            "Unknown movement system: {other}"
        ))),
    }
}

fn equipment_from_db(s: &str) -> MpsResult<Equipment> {
    match s {
        "reformer" => Ok(Equipment::Reformer),
        "chair" => Ok(Equipment::Chair),
        "mat" => Ok(Equipment::Mat),
        "standing" => Ok(Equipment::Standing),
        other => Err(MpsError::UnsupportedEquipment(other.to_string())),
    }
}

fn class_level_from_db(s: &str) -> MpsResult<ClassLevel> {
    match s {
        "Beginner" => Ok(ClassLevel::Beginner),
        "BeginnerIntermediate" => Ok(ClassLevel::BeginnerIntermediate),
        "Intermediate" => Ok(ClassLevel::Intermediate),
        "IntermediateAdvanced" => Ok(ClassLevel::IntermediateAdvanced),
        "Advanced" => Ok(ClassLevel::Advanced),
        other => Err(MpsError::UnsupportedLevel(other.to_string())),
    }
}

fn role_from_db(s: &str) -> MpsResult<ExerciseRole> {
    match s {
        "Assess" => Ok(ExerciseRole::Assess),
        "Prepare" => Ok(ExerciseRole::Prepare),
        "Prime" => Ok(ExerciseRole::Prime),
        "Integrate" => Ok(ExerciseRole::Integrate),
        "Challenge" => Ok(ExerciseRole::Challenge),
        "Transfer" => Ok(ExerciseRole::Transfer),
        "Restore" => Ok(ExerciseRole::Restore),
        other => Err(MpsError::Generation(format!("Unknown exercise role: {other}"))),
    }
}

fn severity_from_db(s: &str) -> MpsResult<SafetySeverity> {
    match s {
        "HardExclude" => Ok(SafetySeverity::HardExclude),
        "Caution" => Ok(SafetySeverity::Caution),
        "RequireRegression" => Ok(SafetySeverity::RequireRegression),
        other => Err(MpsError::Generation(format!(
            "Unknown safety severity: {other}"
        ))),
    }
}

// ── row structs (for sqlx::FromRow) ────────────────────────────────

#[derive(FromRow)]
struct StrategyRow {
    id: i64,
    movement_experience: String,
    primary_focus: String,
    secondary_focus: String,
    explanation_template: String,
}

#[derive(FromRow)]
struct EmphasisRow {
    strategy_id: i64,
    system_name: String,
    emphasis_value: i32,
}

#[derive(FromRow)]
struct ObjectiveRow {
    strategy_id: i64,
    objective: String,
}

#[derive(FromRow)]
struct ModifierRow {
    id: String,
    explanation_fragment: String,
}

#[derive(FromRow)]
struct KeywordRow {
    modifier_id: String,
    keyword: String,
}

#[derive(FromRow)]
struct ModEmphasisRow {
    modifier_id: String,
    system_name: String,
    adjustment: i32,
}

#[derive(FromRow)]
struct ModObjectiveRow {
    modifier_id: String,
    objective: String,
}

#[derive(FromRow)]
struct ExerciseRow {
    id: String,
    name: String,
    description: String,
    equipment: String,
    min_level: String,
    max_level: String,
    difficulty: i32,
    default_duration_minutes: i32,
    regression: Option<String>,
    progression: Option<String>,
}

#[derive(FromRow)]
struct ExerciseRoleRow {
    exercise_id: String,
    role: String,
}

#[derive(FromRow)]
struct ExerciseObjectiveRow {
    exercise_id: String,
    objective: String,
}

#[derive(FromRow)]
struct ExerciseCueRow {
    exercise_id: String,
    cue: String,
    sort_order: i32,
}

#[derive(FromRow)]
struct ExerciseContraindicationRow {
    exercise_id: String,
    tag: String,
    severity: String,
    note: String,
}

#[derive(FromRow)]
struct BenchmarkRow {
    id: String,
    movement_experience: String,
    name: String,
    instruction: String,
}

#[derive(FromRow)]
struct BenchmarkWatchPointRow {
    benchmark_id: String,
    watch_point: String,
}

// ── repository ─────────────────────────────────────────────────────

pub struct SqliteRepository {
    pool: SqlitePool,
    runtime: Arc<Runtime>,
}

impl SqliteRepository {
    /// Open (or create) a SQLite database at `path` and run migrations.
    pub fn open(path: &str) -> MpsResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| MpsError::Repository(format!("Failed to create tokio runtime: {e}")))?;

        let pool = runtime.block_on(async {
            let url = format!("sqlite:{}?mode=rwc", path);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&url)
                .await
                .map_err(|e| MpsError::Repository(format!("Failed to connect: {e}")))?;

            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Migration failed: {e}")))?;

            Ok::<_, MpsError>(pool)
        })?;

        Ok(Self {
            pool,
            runtime: Arc::new(runtime),
        })
    }

    /// Open an in-memory database (useful for testing).
    pub fn open_in_memory() -> MpsResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| MpsError::Repository(format!("Failed to create tokio runtime: {e}")))?;

        let pool = runtime.block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .map_err(|e| MpsError::Repository(format!("Failed to connect: {e}")))?;

            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Migration failed: {e}")))?;

            Ok::<_, MpsError>(pool)
        })?;

        Ok(Self {
            pool,
            runtime: Arc::new(runtime),
        })
    }

    /// Load seed data from an SQL file (for testing).
    pub fn load_seed(&self, sql: &str) -> MpsResult<()> {
        let pool = self.pool.clone();
        let sql = sql.to_string();
        self.runtime.block_on(async {
            sqlx::raw_sql(&sql)
                .execute(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Seed failed: {e}")))?;
            Ok::<_, MpsError>(())
        })
    }
}

impl MpsRepository for SqliteRepository {
    fn base_strategy(
        &self,
        experience: MovementExperience,
    ) -> MpsResult<BaseStrategyRecord> {
        let db_exp = experience_to_db(experience).to_string();
        let pool = self.pool.clone();

        self.runtime.block_on(async move {
            // 1. Find the strategy row
            let strategy: StrategyRow = sqlx::query_as(
                "SELECT id, movement_experience, primary_focus, secondary_focus, explanation_template
                 FROM base_strategies WHERE movement_experience = ?1",
            )
            .bind(&db_exp)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                MpsError::Repository(format!(
                    "No base strategy for {db_exp}: {e}"
                ))
            })?;

            // 2. Emphasis
            let emphasis_rows: Vec<EmphasisRow> = sqlx::query_as(
                "SELECT strategy_id, system_name, emphasis_value
                 FROM base_strategy_emphasis WHERE strategy_id = ?1",
            )
            .bind(strategy.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Emphasis query failed: {e}")))?;

            let mut emphasis = BTreeMap::new();
            for row in &emphasis_rows {
                let system = system_from_db(&row.system_name)?;
                let name = format!("{:?}", system);
                emphasis.insert(name, row.emphasis_value);
            }

            // 3. Objectives
            let objective_rows: Vec<ObjectiveRow> = sqlx::query_as(
                "SELECT strategy_id, objective
                 FROM base_strategy_objectives WHERE strategy_id = ?1",
            )
            .bind(strategy.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Objectives query failed: {e}")))?;

            let objectives = objective_rows.into_iter().map(|r| r.objective).collect();

            Ok(BaseStrategyRecord {
                movement_experience: experience_from_db(&strategy.movement_experience)?,
                primary_focus: system_from_db(&strategy.primary_focus)?,
                secondary_focus: system_from_db(&strategy.secondary_focus)?,
                emphasis,
                objectives,
                explanation_template: strategy.explanation_template,
            })
        })
    }

    fn observation_modifiers(
        &self,
        observations: &[String],
    ) -> MpsResult<Vec<ObservationModifierRecord>> {
        if observations.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.pool.clone();
        let obs: Vec<String> = observations.to_vec();

        self.runtime.block_on(async move {
            // 1. Load all keywords
            let all_keywords: Vec<KeywordRow> = sqlx::query_as(
                "SELECT modifier_id, keyword FROM observation_modifier_keywords",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Keywords query failed: {e}")))?;

            // 2. Build keyword → modifier_id mapping and find matching modifier IDs
            let mut matched_ids: HashSet<String> = HashSet::new();
            for obs_str in &obs {
                let obs_lower = obs_str.to_lowercase();
                for kw_row in &all_keywords {
                    if obs_lower.contains(&kw_row.keyword.to_lowercase()) {
                        matched_ids.insert(kw_row.modifier_id.clone());
                    }
                }
            }

            if matched_ids.is_empty() {
                return Ok(Vec::new());
            }

            // 3. Load matching modifier rows
            let placeholders: String = matched_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");

            let sql = format!(
                "SELECT id, explanation_fragment FROM observation_modifiers WHERE id IN ({placeholders})"
            );

            // Can't use query_as with dynamic number of binds easily — use raw query
            let mut query = sqlx::query_as::<_, ModifierRow>(&sql);
            for id in &matched_ids {
                query = query.bind(id);
            }
            let modifier_rows: Vec<ModifierRow> = query
                .fetch_all(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Modifiers query failed: {e}")))?;

            // 4. Bulk-load related data for matched modifiers
            // Emphasis adjustments
            let emph_sql = format!(
                "SELECT modifier_id, system_name, adjustment \
                 FROM observation_emphasis_adjustments WHERE modifier_id IN ({placeholders})"
            );
            let mut emph_query = sqlx::query_as::<_, ModEmphasisRow>(&emph_sql);
            for id in &matched_ids {
                emph_query = emph_query.bind(id);
            }
            let emph_rows: Vec<ModEmphasisRow> = emph_query
                .fetch_all(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Emphasis adj query failed: {e}")))?;

            // Added objectives
            let add_obj_sql = format!(
                "SELECT modifier_id, objective \
                 FROM observation_added_objectives WHERE modifier_id IN ({placeholders})"
            );
            let mut add_obj_query = sqlx::query_as::<_, ModObjectiveRow>(&add_obj_sql);
            for id in &matched_ids {
                add_obj_query = add_obj_query.bind(id);
            }
            let add_obj_rows: Vec<ModObjectiveRow> = add_obj_query
                .fetch_all(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Added obj query failed: {e}")))?;

            // Preferred objectives
            let pref_obj_sql = format!(
                "SELECT modifier_id, objective \
                 FROM observation_preferred_objectives WHERE modifier_id IN ({placeholders})"
            );
            let mut pref_obj_query = sqlx::query_as::<_, ModObjectiveRow>(&pref_obj_sql);
            for id in &matched_ids {
                pref_obj_query = pref_obj_query.bind(id);
            }
            let pref_obj_rows: Vec<ModObjectiveRow> = pref_obj_query
                .fetch_all(&pool)
                .await
                .map_err(|e| {
                    MpsError::Repository(format!("Preferred obj query failed: {e}"))
                })?;

            // 5. Group by modifier_id
            let mut emphasis_map: HashMap<String, BTreeMap<String, i32>> = HashMap::new();
            for r in &emph_rows {
                let entry = emphasis_map
                    .entry(r.modifier_id.clone())
                    .or_default();
                entry.insert(r.system_name.clone(), r.adjustment);
            }

            let mut added_obj_map: HashMap<String, Vec<String>> = HashMap::new();
            for r in &add_obj_rows {
                added_obj_map
                    .entry(r.modifier_id.clone())
                    .or_default()
                    .push(r.objective.clone());
            }

            let mut pref_obj_map: HashMap<String, Vec<String>> = HashMap::new();
            for r in &pref_obj_rows {
                pref_obj_map
                    .entry(r.modifier_id.clone())
                    .or_default()
                    .push(r.objective.clone());
            }

            // 6. Build result
            let mut results = Vec::new();
            for m in &modifier_rows {
                results.push(ObservationModifierRecord {
                    id: m.id.clone(),
                    emphasis_adjustments: emphasis_map
                        .remove(&m.id)
                        .unwrap_or_default(),
                    added_objectives: added_obj_map
                        .remove(&m.id)
                        .unwrap_or_default(),
                    preferred_exercise_objectives: pref_obj_map
                        .remove(&m.id)
                        .unwrap_or_default(),
                    explanation_fragment: m.explanation_fragment.clone(),
                });
            }

            Ok(results)
        })
    }

    fn benchmarks(&self, experience: MovementExperience) -> MpsResult<Vec<BenchmarkRecord>> {
        let db_exp = experience_to_db(experience).to_string();
        let pool = self.pool.clone();

        self.runtime.block_on(async move {
            let bench_rows: Vec<BenchmarkRow> = sqlx::query_as(
                "SELECT id, movement_experience, name, instruction \
                 FROM benchmarks WHERE movement_experience = ?1",
            )
            .bind(&db_exp)
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Benchmarks query failed: {e}")))?;

            if bench_rows.is_empty() {
                return Ok(Vec::new());
            }

            // Fetch all watch points for these benchmarks
            let ids: Vec<&str> = bench_rows.iter().map(|b| b.id.as_str()).collect();
            let placeholders: String = ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");

            let wp_sql = format!(
                "SELECT benchmark_id, watch_point \
                 FROM benchmark_watch_points WHERE benchmark_id IN ({placeholders})"
            );
            let mut wp_query = sqlx::query_as::<_, BenchmarkWatchPointRow>(&wp_sql);
            for id in &ids {
                wp_query = wp_query.bind(*id);
            }
            let wp_rows: Vec<BenchmarkWatchPointRow> = wp_query
                .fetch_all(&pool)
                .await
                .map_err(|e| MpsError::Repository(format!("Watch points query failed: {e}")))?;

            let mut wp_map: HashMap<String, Vec<String>> = HashMap::new();
            for wp in &wp_rows {
                wp_map
                    .entry(wp.benchmark_id.clone())
                    .or_default()
                    .push(wp.watch_point.clone());
            }

            let results = bench_rows
                .into_iter()
                .map(|b| BenchmarkRecord {
                    id: b.id.clone(),
                    name: b.name,
                    instruction: b.instruction,
                    watch_points: wp_map.remove(&b.id).unwrap_or_default(),
                })
                .collect();

            Ok(results)
        })
    }

    fn exercises(&self) -> MpsResult<Vec<ExerciseRecord>> {
        let pool = self.pool.clone();

        self.runtime.block_on(async move {
            // 1. All exercises
            let ex_rows: Vec<ExerciseRow> = sqlx::query_as(
                "SELECT id, name, description, equipment, min_level, max_level, \
                 difficulty, default_duration_minutes, regression, progression \
                 FROM exercises",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Exercises query failed: {e}")))?;

            // 2. Bulk-load related data
            let role_rows: Vec<ExerciseRoleRow> =
                sqlx::query_as("SELECT exercise_id, role FROM exercise_roles")
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| MpsError::Repository(format!("Roles query failed: {e}")))?;

            let obj_rows: Vec<ExerciseObjectiveRow> =
                sqlx::query_as("SELECT exercise_id, objective FROM exercise_objectives")
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| MpsError::Repository(format!("Obj query failed: {e}")))?;

            let cue_rows: Vec<ExerciseCueRow> = sqlx::query_as(
                "SELECT exercise_id, cue, sort_order FROM exercise_teaching_cues",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Cues query failed: {e}")))?;

            let contra_rows: Vec<ExerciseContraindicationRow> = sqlx::query_as(
                "SELECT exercise_id, tag, severity, note FROM exercise_contraindications",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| MpsError::Repository(format!("Contra query failed: {e}")))?;

            // 3. Group by exercise_id
            let mut roles_map: HashMap<String, Vec<String>> = HashMap::new();
            for r in &role_rows {
                roles_map
                    .entry(r.exercise_id.clone())
                    .or_default()
                    .push(r.role.clone());
            }

            let mut obj_map: HashMap<String, Vec<String>> = HashMap::new();
            for o in &obj_rows {
                obj_map
                    .entry(o.exercise_id.clone())
                    .or_default()
                    .push(o.objective.clone());
            }

            let mut cue_map: HashMap<String, Vec<(i32, String)>> = HashMap::new();
            for c in &cue_rows {
                cue_map
                    .entry(c.exercise_id.clone())
                    .or_default()
                    .push((c.sort_order, c.cue.clone()));
            }
            // Sort cues by sort_order
            for cues in cue_map.values_mut() {
                cues.sort_by_key(|(ord, _)| *ord);
            }

            let mut contra_map: HashMap<String, Vec<ExerciseContraindicationRow>> = HashMap::new();
            for c in &contra_rows {
                contra_map
                    .entry(c.exercise_id.clone())
                    .or_default()
                    .push(ExerciseContraindicationRow {
                        exercise_id: c.exercise_id.clone(),
                        tag: c.tag.clone(),
                        severity: c.severity.clone(),
                        note: c.note.clone(),
                    });
            }

            // 4. Build result
            let mut results = Vec::new();
            for ex in &ex_rows {
                let roles: Vec<ExerciseRole> = roles_map
                    .remove(&ex.id)
                    .unwrap_or_default()
                    .iter()
                    .map(|s| role_from_db(s))
                    .collect::<MpsResult<Vec<_>>>()?;

                let objectives = obj_map.remove(&ex.id).unwrap_or_default();

                let teaching_cues: Vec<String> = cue_map
                    .remove(&ex.id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(_, cue)| cue)
                    .collect();

                let contraindications: Vec<ContraindicationRecord> = contra_map
                    .remove(&ex.id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| {
                        Ok(ContraindicationRecord {
                            tag: c.tag,
                            severity: severity_from_db(&c.severity)?,
                            note: c.note,
                        })
                    })
                    .collect::<MpsResult<Vec<_>>>()?;

                results.push(ExerciseRecord {
                    id: ex.id.clone(),
                    name: ex.name.clone(),
                    description: ex.description.clone(),
                    equipment: equipment_from_db(&ex.equipment)?,
                    roles,
                    objectives,
                    min_level: class_level_from_db(&ex.min_level)?,
                    max_level: class_level_from_db(&ex.max_level)?,
                    difficulty: ex.difficulty as u8,
                    default_duration_minutes: ex.default_duration_minutes as u32,
                    teaching_cues,
                    regression: ex.regression.clone(),
                    progression: ex.progression.clone(),
                    contraindications,
                });
            }

            Ok(results)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &str = include_str!("../../../data/seed/mps_seed.sql");

    fn repo_with_seed() -> SqliteRepository {
        let repo = SqliteRepository::open_in_memory().expect("open in-memory db");
        repo.load_seed(SEED).expect("seed data");
        repo
    }

    #[test]
    fn base_strategy_shoulder_freedom() {
        let repo = repo_with_seed();
        let bs = repo
            .base_strategy(MovementExperience::ShoulderFreedom)
            .unwrap();
        assert_eq!(bs.movement_experience, MovementExperience::ShoulderFreedom);
        assert_eq!(bs.primary_focus, MovementSystem::Shoulder);
        assert_eq!(bs.secondary_focus, MovementSystem::Thoracic);
        assert_eq!(bs.objectives.len(), 3);
        assert!(!bs.explanation_template.is_empty());
        assert_eq!(bs.emphasis.get("Shoulder"), Some(&40));
    }

    #[test]
    fn base_strategy_happy_hips() {
        let repo = repo_with_seed();
        let bs = repo
            .base_strategy(MovementExperience::HappyHips)
            .unwrap();
        assert_eq!(bs.primary_focus, MovementSystem::Hip);
        assert_eq!(bs.secondary_focus, MovementSystem::Legs);
    }

    #[test]
    fn base_strategy_spine_reset() {
        let repo = repo_with_seed();
        let bs = repo
            .base_strategy(MovementExperience::SpineReset)
            .unwrap();
        assert_eq!(bs.primary_focus, MovementSystem::Spine);
        assert_eq!(bs.secondary_focus, MovementSystem::BreathCore);
    }

    #[test]
    fn observation_modifiers_match_keywords() {
        let repo = repo_with_seed();

        // "stiff thoracic" should match thoracic_stiffness (keywords: thoracic, stiff, rounded)
        let mods = repo
            .observation_modifiers(&["stiff thoracic".into()])
            .unwrap();
        assert!(mods.iter().any(|m| m.id == "thoracic_stiffness"));
    }

    #[test]
    fn observation_modifiers_match_multiple() {
        let repo = repo_with_seed();

        // "knee pain and hip tightness" should match knee_pain and hip_tightness
        let mods = repo
            .observation_modifiers(&["knee pain and hip tightness".into()])
            .unwrap();
        let ids: Vec<&str> = mods.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"knee_pain"), "should match knee_pain, got {ids:?}");
        assert!(ids.contains(&"hip_tightness"), "should match hip_tightness, got {ids:?}");
    }

    #[test]
    fn observation_modifiers_populates_emphasis() {
        let repo = repo_with_seed();
        let mods = repo
            .observation_modifiers(&["stiff thoracic".into()])
            .unwrap();
        let ts = mods.iter().find(|m| m.id == "thoracic_stiffness").unwrap();
        assert!(ts.emphasis_adjustments.contains_key("Thoracic"));
        assert!(ts.emphasis_adjustments.contains_key("Shoulder"));
        assert!(!ts.explanation_fragment.is_empty());
    }

    #[test]
    fn observation_modifiers_empty_input_returns_empty() {
        let repo = repo_with_seed();
        let mods = repo.observation_modifiers(&[]).unwrap();
        assert!(mods.is_empty());
    }

    #[test]
    fn observation_modifiers_no_match_returns_empty() {
        let repo = repo_with_seed();
        let mods = repo
            .observation_modifiers(&["zzzzzz_nonexistent_zzzzzz".into()])
            .unwrap();
        assert!(mods.is_empty());
    }

    #[test]
    fn benchmarks_shoulder_freedom() {
        let repo = repo_with_seed();
        let benches = repo
            .benchmarks(MovementExperience::ShoulderFreedom)
            .unwrap();
        assert_eq!(benches.len(), 2);
        assert!(benches.iter().any(|b| b.id == "overhead_reach_test"));
        let ort = benches.iter().find(|b| b.id == "overhead_reach_test").unwrap();
        assert_eq!(ort.watch_points.len(), 3);
    }

    #[test]
    fn benchmarks_happy_hips() {
        let repo = repo_with_seed();
        let benches = repo
            .benchmarks(MovementExperience::HappyHips)
            .unwrap();
        assert_eq!(benches.len(), 2);
        assert!(benches.iter().any(|b| b.id == "deep_squat_test"));
    }

    #[test]
    fn exercises_returns_all_24() {
        let repo = repo_with_seed();
        let exs = repo.exercises().unwrap();
        assert_eq!(exs.len(), 24, "expected 24 exercises from seed data");
    }

    #[test]
    fn exercises_footwork_has_expected_data() {
        let repo = repo_with_seed();
        let exs = repo.exercises().unwrap();
        let fw = exs.iter().find(|e| e.id == "footwork").unwrap();
        assert_eq!(fw.name, "Footwork");
        assert_eq!(fw.equipment, Equipment::Reformer);
        assert_eq!(fw.min_level, ClassLevel::Beginner);
        assert_eq!(fw.max_level, ClassLevel::IntermediateAdvanced);
        assert_eq!(fw.difficulty, 2);
        assert!(fw.roles.contains(&ExerciseRole::Prepare));
        assert!(fw.roles.contains(&ExerciseRole::Prime));
        assert_eq!(fw.objectives.len(), 3);
        assert_eq!(fw.teaching_cues.len(), 3);
        assert_eq!(fw.contraindications.len(), 1);
        assert_eq!(fw.contraindications[0].severity, SafetySeverity::Caution);
    }

    #[test]
    fn exercises_elephant_has_hard_exclude_contra() {
        let repo = repo_with_seed();
        let exs = repo.exercises().unwrap();
        let elephant = exs.iter().find(|e| e.id == "elephant").unwrap();
        let hb = elephant
            .contraindications
            .iter()
            .find(|c| c.tag == "high_blood_pressure")
            .unwrap();
        assert_eq!(hb.severity, SafetySeverity::HardExclude);
    }
}
