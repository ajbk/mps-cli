use std::collections::BTreeMap;

use crate::{
    ClassLevel, Equipment, ExerciseRole, MovementExperience, MovementSystem, SafetySeverity,
};

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
    fn base_strategy(&self, experience: MovementExperience)
        -> crate::MpsResult<BaseStrategyRecord>;
    fn observation_modifiers(
        &self,
        observations: &[String],
    ) -> crate::MpsResult<Vec<ObservationModifierRecord>>;
    fn benchmarks(&self, experience: MovementExperience) -> crate::MpsResult<Vec<BenchmarkRecord>>;
    fn exercises(&self) -> crate::MpsResult<Vec<ExerciseRecord>>;
}
