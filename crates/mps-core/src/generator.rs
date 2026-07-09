use crate::{
    AssessmentPlan, BenchmarkPlan, ClassPlan, ClassRequest, ExerciseRole, ExerciseTeachingUnit,
    JourneyPhasePlan, MovementJourneyPhase, MpsRepository, MpsResult, RetestPlan,
    SafetyExerciseNote, SafetySummary,
    build_strategy, difficulty_fit_score, objective_match_score,
    phase_allocations_for_duration, role_match_score,
};

pub fn generate_class_plan<R: MpsRepository>(
    request: &ClassRequest,
    repository: &R,
) -> MpsResult<ClassPlan> {
    request.validate()?;

    let allocations = phase_allocations_for_duration(request.duration_minutes)?;
    let base = repository.base_strategy(request.movement_experience)?;
    let modifiers = repository.observation_modifiers(&request.observations)?;
    let strategy = build_strategy(base, modifiers);
    let benchmark_records = repository.benchmarks(request.movement_experience)?;
    let all_exercises = repository.exercises()?;

    // Safety: filter out exercises with HardExclude contraindications
    let safe_exercises: Vec<_> = all_exercises
        .iter()
        .filter(|ex| {
            !ex.contraindications.iter().any(|c| {
                c.severity == crate::SafetySeverity::HardExclude
                    && request
                        .group_safety
                        .contraindications
                        .iter()
                        .any(|gc| gc.eq_ignore_ascii_case(&c.tag))
            })
        })
        .collect();

    // Build journey phases
    let mut journey = Vec::new();
    let mut warnings = Vec::new();

    for allocation in &allocations {
        let target_role = match allocation.phase {
            MovementJourneyPhase::Arrive => ExerciseRole::Assess,
            MovementJourneyPhase::Prepare => ExerciseRole::Prepare,
            MovementJourneyPhase::Build => ExerciseRole::Prime,
            MovementJourneyPhase::Integrate => ExerciseRole::Integrate,
            MovementJourneyPhase::Challenge => ExerciseRole::Challenge,
            MovementJourneyPhase::Transfer => ExerciseRole::Transfer,
            MovementJourneyPhase::ResetRetest => ExerciseRole::Restore,
        };

        let purpose = match allocation.phase {
            MovementJourneyPhase::Arrive => "Assess baseline and prepare the body".to_string(),
            MovementJourneyPhase::Prepare => "Warm up and mobilize target systems".to_string(),
            MovementJourneyPhase::Build => "Build strength and movement capacity".to_string(),
            MovementJourneyPhase::Integrate => "Connect movements across systems".to_string(),
            MovementJourneyPhase::Challenge => "Push 80% success / 20% challenge".to_string(),
            MovementJourneyPhase::Transfer => "Transfer to functional movement".to_string(),
            MovementJourneyPhase::ResetRetest => "Retest and confirm improvement".to_string(),
        };

        // Score and select exercises for this phase
        let mut scored: Vec<_> = safe_exercises
            .iter()
            .filter(|ex| ex.roles.contains(&target_role))
            .filter(|ex| {
                let level_num = request.level.numeric();
                ex.min_level.numeric() <= level_num && level_num <= ex.max_level.numeric()
            })
            .map(|ex| {
                let r_score = role_match_score(&ex.roles, target_role);
                let d_score = difficulty_fit_score(ex.difficulty, request.level);
                let o_score =
                    objective_match_score(&ex.objectives, &strategy.preferred_exercise_objectives);
                let equipment_score: i32 =
                    if request.equipment.contains(&ex.equipment) { 5 } else { 0 };
                (ex, r_score + d_score + o_score + equipment_score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        // Select top exercises that fit in the time budget
        let mut phase_exercises = Vec::new();
        let mut remaining = allocation.minutes;

        for (ex, _score) in &scored {
            if remaining == 0 {
                break;
            }
            let dur = ex.default_duration_minutes.min(remaining);
            phase_exercises.push(ExerciseTeachingUnit {
                exercise_id: ex.id.clone(),
                name: ex.name.clone(),
                apparatus: ex.equipment,
                role: target_role,
                duration_minutes: dur,
                movement_objectives: ex.objectives.clone(),
                why_selected: format!("Score-based selection for {:?} phase", allocation.phase),
                teaching_cues: ex.teaching_cues.clone(),
                regression: ex.regression.clone(),
                progression: ex.progression.clone(),
                safety_notes: ex
                    .contraindications
                    .iter()
                    .filter(|c| {
                        c.severity == crate::SafetySeverity::Caution
                            && request
                                .group_safety
                                .contraindications
                                .iter()
                                .any(|gc| gc.eq_ignore_ascii_case(&c.tag))
                    })
                    .map(|c| c.note.clone())
                    .collect(),
            });
            remaining -= dur;
        }

        if phase_exercises.is_empty()
            && matches!(allocation.phase, MovementJourneyPhase::Build)
        {
            return Err(crate::MpsError::NoCandidates {
                phase: allocation.phase.label().to_string(),
                role: format!("{:?}", target_role),
            });
        }

        if phase_exercises.is_empty() {
            warnings.push(format!(
                "No exercises found for {:?} phase",
                allocation.phase
            ));
        }

        journey.push(JourneyPhasePlan {
            phase: allocation.phase,
            purpose,
            target_duration_minutes: allocation.minutes,
            exercises: phase_exercises,
        });
    }

    // Build benchmark
    let benchmark = BenchmarkPlan {
        assessments: benchmark_records
            .iter()
            .map(|br| AssessmentPlan {
                name: br.name.clone(),
                instruction: br.instruction.clone(),
                what_to_watch: br.watch_points.clone(),
            })
            .collect(),
    };

    // Build safety summary
    let excluded: Vec<_> = all_exercises
        .iter()
        .filter(|ex| {
            ex.contraindications.iter().any(|c| {
                c.severity == crate::SafetySeverity::HardExclude
                    && request
                        .group_safety
                        .contraindications
                        .iter()
                        .any(|gc| gc.eq_ignore_ascii_case(&c.tag))
            })
        })
        .map(|ex| SafetyExerciseNote {
            exercise_id: ex.id.clone(),
            name: ex.name.clone(),
            reason: ex
                .contraindications
                .iter()
                .filter(|c| c.severity == crate::SafetySeverity::HardExclude)
                .map(|c| c.note.clone())
                .collect::<Vec<_>>()
                .join("; "),
        })
        .collect();

    let safety_summary = SafetySummary {
        risk_policy: request.group_safety.risk_policy,
        applied_contraindications: request.group_safety.contraindications.clone(),
        excluded_exercises: excluded,
        modified_exercises: vec![],
        safety_notes: vec![format!(
            "Group safety policy: {:?}",
            request.group_safety.risk_policy
        )],
    };

    // Build class title
    let class_title = format!(
        "{:?} — {} min {:?}",
        request.movement_experience, request.duration_minutes, request.level
    );

    let retest = RetestPlan {
        assessments: benchmark.assessments.clone(),
        expected_improvement: format!(
            "After this {:?} class, expect improved movement quality in the target systems.",
            request.movement_experience
        ),
    };

    Ok(ClassPlan {
        class_title,
        movement_experience: request.movement_experience,
        duration_minutes: request.duration_minutes,
        level: request.level,
        students: request.students,
        equipment: request.equipment.clone(),
        movement_strategy: strategy,
        benchmark,
        journey,
        safety_summary,
        retest,
        expected_improvement: format!(
            "Improved {:?} movement quality after class.",
            request.movement_experience
        ),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::collections::BTreeMap;

    struct FakeRepository;

    impl MpsRepository for FakeRepository {
        fn base_strategy(
            &self,
            _exp: MovementExperience,
        ) -> MpsResult<BaseStrategyRecord> {
            Ok(BaseStrategyRecord {
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
                explanation_template: "Shoulder Freedom base strategy.".to_string(),
            })
        }
        fn observation_modifiers(
            &self,
            _obs: &[String],
        ) -> MpsResult<Vec<ObservationModifierRecord>> {
            Ok(vec![])
        }
        fn benchmarks(
            &self,
            _exp: MovementExperience,
        ) -> MpsResult<Vec<BenchmarkRecord>> {
            Ok(vec![BenchmarkRecord {
                id: "overhead_reach".to_string(),
                name: "Overhead Reach Test".to_string(),
                instruction: "Reach arms overhead".to_string(),
                watch_points: vec!["Limited ROM".to_string()],
            }])
        }
        fn exercises(&self) -> MpsResult<Vec<ExerciseRecord>> {
            let make_ex = |id: &str,
                           name: &str,
                           equip: Equipment,
                           roles: Vec<ExerciseRole>,
                           min_l: ClassLevel,
                           max_l: ClassLevel,
                           diff: u8|
             -> ExerciseRecord {
                ExerciseRecord {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: "".to_string(),
                    equipment: equip,
                    roles,
                    objectives: vec!["Mobility".to_string()],
                    min_level: min_l,
                    max_level: max_l,
                    difficulty: diff,
                    default_duration_minutes: 5,
                    teaching_cues: vec!["Breathe".to_string()],
                    regression: None,
                    progression: None,
                    contraindications: vec![],
                }
            };
            Ok(vec![
                make_ex(
                    "arm_raise",
                    "Arm Raise",
                    Equipment::Standing,
                    vec![ExerciseRole::Assess, ExerciseRole::Prepare],
                    ClassLevel::Beginner,
                    ClassLevel::Advanced,
                    1,
                ),
                make_ex(
                    "cat_cow",
                    "Cat-Cow",
                    Equipment::Mat,
                    vec![ExerciseRole::Prepare, ExerciseRole::Restore],
                    ClassLevel::Beginner,
                    ClassLevel::Advanced,
                    1,
                ),
                make_ex(
                    "pelvic_curl",
                    "Pelvic Curl",
                    Equipment::Reformer,
                    vec![ExerciseRole::Prepare, ExerciseRole::Prime],
                    ClassLevel::Beginner,
                    ClassLevel::IntermediateAdvanced,
                    1,
                ),
                make_ex(
                    "arms_straps",
                    "Arms in Straps",
                    Equipment::Reformer,
                    vec![ExerciseRole::Prime, ExerciseRole::Integrate],
                    ClassLevel::BeginnerIntermediate,
                    ClassLevel::Advanced,
                    3,
                ),
                make_ex(
                    "pulling_straps",
                    "Pulling Straps",
                    Equipment::Reformer,
                    vec![ExerciseRole::Prime, ExerciseRole::Challenge],
                    ClassLevel::Intermediate,
                    ClassLevel::Advanced,
                    3,
                ),
                make_ex(
                    "standing_roll",
                    "Standing Roll Down",
                    Equipment::Standing,
                    vec![ExerciseRole::Restore],
                    ClassLevel::Beginner,
                    ClassLevel::Advanced,
                    1,
                ),
            ])
        }
    }

    #[test]
    fn generates_7_phases() {
        let request = ClassRequest {
            students: 3,
            movement_experience: MovementExperience::ShoulderFreedom,
            level: ClassLevel::BeginnerIntermediate,
            equipment: vec![Equipment::Reformer, Equipment::Chair],
            duration_minutes: 60,
            observations: vec![],
            group_safety: GroupSafety {
                contraindications: vec![],
                risk_policy: RiskPolicy::Balanced,
            },
        };
        let plan = generate_class_plan(&request, &FakeRepository).unwrap();
        assert_eq!(plan.journey.len(), 7);
        assert_eq!(plan.duration_minutes, 60);
        assert_eq!(plan.movement_strategy.emphasis.values().sum::<u32>(), 100);
        assert!(plan.journey.iter().any(|p| p.phase == MovementJourneyPhase::Build));
    }
}
