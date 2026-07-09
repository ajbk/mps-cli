use crate::{
    ClassLevel, Equipment, ExerciseRole, MovementExperience, MovementJourneyPhase, MovementSystem,
    RiskPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
