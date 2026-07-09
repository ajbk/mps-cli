use std::collections::HashSet;

use crate::{
    build_strategy, phase_allocations_for_duration, score_exercise, AssessmentPlan, BenchmarkPlan,
    ClassLevel, ClassPlan, ClassRequest, Equipment, ExerciseRecord, ExerciseRole,
    ExerciseTeachingUnit, JourneyPhasePlan, MovementJourneyPhase, MpsError, MpsRepository,
    MpsResult, RetestPlan, SafetyExerciseNote, SafetySeverity, SafetySummary,
};

pub fn generate_class_plan<R: MpsRepository>(
    request: &ClassRequest,
    repository: &R,
) -> MpsResult<ClassPlan> {
    request.validate()?;

    let allocations = phase_allocations_for_duration(request.duration_minutes)?;
    let base_strategy = repository.base_strategy(request.movement_experience)?;
    let modifiers = repository.observation_modifiers(&request.observations)?;
    let strategy = build_strategy(base_strategy, modifiers);
    let benchmarks = repository.benchmarks(request.movement_experience)?;
    let exercises = repository.exercises()?;
    let safety_tags = normalized_tags(&request.group_safety.contraindications);

    let mut warnings = Vec::new();
    if benchmarks.is_empty() {
        warnings.push("No benchmark records were found for this movement experience.".to_string());
    }

    let mut used_exercise_ids = HashSet::new();
    let mut journey = Vec::with_capacity(allocations.len());
    let mut modified_exercises = Vec::new();

    for allocation in &allocations {
        let target_role = target_role_for_phase(allocation.phase);
        let phase_candidates = ranked_candidates(
            &exercises,
            request,
            allocation.phase,
            target_role,
            &strategy,
            &safety_tags,
            &used_exercise_ids,
        );

        let allow_reuse_fallback = phase_candidates.is_empty();
        let mut ranked = if allow_reuse_fallback {
            ranked_candidates(
                &exercises,
                request,
                allocation.phase,
                target_role,
                &strategy,
                &safety_tags,
                &HashSet::new(),
            )
        } else {
            phase_candidates
        };

        let max_count = max_exercise_count(allocation.phase);
        let min_count = min_exercise_count(allocation.phase);
        let mut remaining = allocation.minutes;
        let mut selected = Vec::new();

        for (exercise, score) in ranked.drain(..) {
            if selected.len() >= max_count || remaining == 0 {
                break;
            }

            let mut regression = exercise.regression.clone();
            let mut safety_notes = caution_notes(exercise, &safety_tags);
            if requires_regression(exercise, &safety_tags) {
                let note = "Regression required by group safety constraint.".to_string();
                safety_notes.push(note.clone());
                if regression.is_none() {
                    regression = Some("Reduce range, load, tempo, or support the position.".into());
                }
                modified_exercises.push(SafetyExerciseNote {
                    exercise_id: exercise.id.clone(),
                    name: exercise.name.clone(),
                    reason: note,
                });
            }

            let duration = exercise.default_duration_minutes.min(remaining);
            remaining -= duration;
            used_exercise_ids.insert(exercise.id.clone());

            selected.push(ExerciseTeachingUnit {
                exercise_id: exercise.id.clone(),
                name: exercise.name.clone(),
                apparatus: exercise.equipment,
                role: target_role,
                duration_minutes: duration,
                movement_objectives: exercise.objectives.clone(),
                why_selected: format!(
                    "Matched {} role for {} with score {}.",
                    role_label(target_role),
                    allocation.phase.label(),
                    score
                ),
                teaching_cues: exercise.teaching_cues.clone(),
                regression,
                progression: exercise.progression.clone(),
                safety_notes,
            });
        }

        if selected.len() < min_count {
            return Err(MpsError::NoCandidates {
                phase: allocation.phase.label().to_string(),
                role: role_label(target_role).to_string(),
            });
        }

        if remaining > 0 {
            warnings.push(format!(
                "{} is under-filled by {} minute(s). Add more seed data for this role/equipment/level.",
                allocation.phase.label(),
                remaining
            ));
        }

        journey.push(JourneyPhasePlan {
            phase: allocation.phase,
            purpose: allocation.phase.purpose().to_string(),
            target_duration_minutes: allocation.minutes,
            exercises: selected,
        });
    }

    validate_journey(&journey)?;

    let benchmark = BenchmarkPlan {
        assessments: benchmarks
            .iter()
            .map(|record| AssessmentPlan {
                name: record.name.clone(),
                instruction: record.instruction.clone(),
                what_to_watch: record.watch_points.clone(),
            })
            .collect(),
    };

    let excluded_exercises = excluded_exercises(&exercises, &safety_tags);
    let safety_summary = SafetySummary {
        risk_policy: request.group_safety.risk_policy,
        applied_contraindications: request.group_safety.contraindications.clone(),
        excluded_exercises,
        modified_exercises,
        safety_notes: vec![
            "This plan is a teaching aid, not a medical diagnosis. Instructor judgement is required."
                .to_string(),
            format!("Group safety policy: {:?}", request.group_safety.risk_policy),
        ],
    };

    let class_title = format!(
        "{} - {} min {}",
        request.movement_experience.label(),
        request.duration_minutes,
        request.level.label()
    );

    let retest = RetestPlan {
        assessments: benchmark.assessments.clone(),
        expected_improvement: format!(
            "Students should feel clearer {} movement quality and be able to compare it against the opening benchmark.",
            request.movement_experience.label()
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
            "Improved {} movement quality with whole-body support.",
            request.movement_experience.label()
        ),
        warnings,
    })
}

fn ranked_candidates<'a>(
    exercises: &'a [ExerciseRecord],
    request: &ClassRequest,
    phase: MovementJourneyPhase,
    role: ExerciseRole,
    strategy: &crate::MovementStrategyPlan,
    safety_tags: &HashSet<String>,
    used_exercise_ids: &HashSet<String>,
) -> Vec<(&'a ExerciseRecord, i32)> {
    let mut candidates: Vec<_> = exercises
        .iter()
        .filter(|exercise| !used_exercise_ids.contains(&exercise.id))
        .filter(|exercise| exercise.roles.contains(&role))
        .filter(|exercise| {
            exercise_allowed_for_phase(exercise.equipment, &request.equipment, phase)
        })
        .filter(|exercise| level_allowed(exercise, request.level, phase))
        .filter(|exercise| !hard_excluded(exercise, safety_tags))
        .map(|exercise| {
            let mut score = score_exercise(
                exercise,
                role,
                request.level,
                request.movement_experience,
                strategy.primary_focus,
                strategy.secondary_focus,
                &strategy.preferred_exercise_objectives,
            );
            if request.equipment.contains(&exercise.equipment) {
                score += 5;
            }
            score -= safety_penalty(exercise, safety_tags);
            (exercise, score)
        })
        .collect();

    candidates.sort_by(|(a_ex, a_score), (b_ex, b_score)| {
        b_score
            .cmp(a_score)
            .then_with(|| a_ex.difficulty.cmp(&b_ex.difficulty))
            .then_with(|| a_ex.id.cmp(&b_ex.id))
    });
    candidates
}

fn exercise_allowed_for_phase(
    equipment: Equipment,
    selected_equipment: &[Equipment],
    phase: MovementJourneyPhase,
) -> bool {
    if equipment.is_primary_apparatus() {
        return selected_equipment.contains(&equipment);
    }

    matches!(
        phase,
        MovementJourneyPhase::Arrive
            | MovementJourneyPhase::Prepare
            | MovementJourneyPhase::Transfer
            | MovementJourneyPhase::ResetRetest
    )
}

fn level_allowed(
    exercise: &ExerciseRecord,
    request_level: ClassLevel,
    phase: MovementJourneyPhase,
) -> bool {
    let level = request_level.numeric();
    if exercise.min_level.numeric() > level {
        return false;
    }

    if level <= exercise.max_level.numeric() {
        return true;
    }

    matches!(phase, MovementJourneyPhase::Challenge)
        && exercise.difficulty <= level + 1
        && exercise.regression.is_some()
}

fn target_role_for_phase(phase: MovementJourneyPhase) -> ExerciseRole {
    match phase {
        MovementJourneyPhase::Arrive => ExerciseRole::Assess,
        MovementJourneyPhase::Prepare => ExerciseRole::Prepare,
        MovementJourneyPhase::Build => ExerciseRole::Prime,
        MovementJourneyPhase::Integrate => ExerciseRole::Integrate,
        MovementJourneyPhase::Challenge => ExerciseRole::Challenge,
        MovementJourneyPhase::Transfer => ExerciseRole::Transfer,
        MovementJourneyPhase::ResetRetest => ExerciseRole::Restore,
    }
}

fn max_exercise_count(phase: MovementJourneyPhase) -> usize {
    match phase {
        MovementJourneyPhase::Arrive => 2,
        MovementJourneyPhase::Prepare => 3,
        MovementJourneyPhase::Build => 4,
        MovementJourneyPhase::Integrate => 3,
        MovementJourneyPhase::Challenge => 2,
        MovementJourneyPhase::Transfer => 2,
        MovementJourneyPhase::ResetRetest => 2,
    }
}

fn min_exercise_count(phase: MovementJourneyPhase) -> usize {
    match phase {
        MovementJourneyPhase::Build => 2,
        _ => 1,
    }
}

fn validate_journey(journey: &[JourneyPhasePlan]) -> MpsResult<()> {
    if journey.len() != 7 {
        return Err(MpsError::Generation(format!(
            "expected 7 phases, got {}",
            journey.len()
        )));
    }

    let build = journey
        .iter()
        .find(|phase| phase.phase == MovementJourneyPhase::Build)
        .ok_or_else(|| MpsError::Generation("BUILD phase is missing".to_string()))?;
    let prime_count = build
        .exercises
        .iter()
        .filter(|exercise| exercise.role == ExerciseRole::Prime)
        .count();

    if !(2..=4).contains(&prime_count) {
        return Err(MpsError::Generation(format!(
            "BUILD must contain 2-4 Prime exercises, got {prime_count}"
        )));
    }

    Ok(())
}

fn normalized_tags(tags: &[String]) -> HashSet<String> {
    tags.iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn hard_excluded(exercise: &ExerciseRecord, safety_tags: &HashSet<String>) -> bool {
    exercise.contraindications.iter().any(|contra| {
        contra.severity == SafetySeverity::HardExclude
            && safety_tags.contains(&contra.tag.to_lowercase())
    })
}

fn requires_regression(exercise: &ExerciseRecord, safety_tags: &HashSet<String>) -> bool {
    exercise.contraindications.iter().any(|contra| {
        contra.severity == SafetySeverity::RequireRegression
            && safety_tags.contains(&contra.tag.to_lowercase())
    })
}

fn caution_notes(exercise: &ExerciseRecord, safety_tags: &HashSet<String>) -> Vec<String> {
    exercise
        .contraindications
        .iter()
        .filter(|contra| {
            contra.severity == SafetySeverity::Caution
                && safety_tags.contains(&contra.tag.to_lowercase())
        })
        .map(|contra| contra.note.clone())
        .collect()
}

fn safety_penalty(exercise: &ExerciseRecord, safety_tags: &HashSet<String>) -> i32 {
    exercise
        .contraindications
        .iter()
        .filter(|contra| safety_tags.contains(&contra.tag.to_lowercase()))
        .map(|contra| match contra.severity {
            SafetySeverity::HardExclude => 1000,
            SafetySeverity::RequireRegression => 8,
            SafetySeverity::Caution => 4,
        })
        .sum()
}

fn excluded_exercises(
    exercises: &[ExerciseRecord],
    safety_tags: &HashSet<String>,
) -> Vec<SafetyExerciseNote> {
    exercises
        .iter()
        .filter(|exercise| hard_excluded(exercise, safety_tags))
        .map(|exercise| SafetyExerciseNote {
            exercise_id: exercise.id.clone(),
            name: exercise.name.clone(),
            reason: exercise
                .contraindications
                .iter()
                .filter(|contra| {
                    contra.severity == SafetySeverity::HardExclude
                        && safety_tags.contains(&contra.tag.to_lowercase())
                })
                .map(|contra| contra.note.clone())
                .collect::<Vec<_>>()
                .join("; "),
        })
        .collect()
}

fn role_label(role: ExerciseRole) -> &'static str {
    match role {
        ExerciseRole::Assess => "Assess",
        ExerciseRole::Prepare => "Prepare",
        ExerciseRole::Prime => "Prime",
        ExerciseRole::Integrate => "Integrate",
        ExerciseRole::Challenge => "Challenge",
        ExerciseRole::Transfer => "Transfer",
        ExerciseRole::Restore => "Restore",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BaseStrategyRecord, BenchmarkRecord, ContraindicationRecord, GroupSafety,
        MovementExperience, MovementSystem, ObservationModifierRecord, RiskPolicy,
    };
    use std::collections::BTreeMap;

    struct FakeRepository;

    impl MpsRepository for FakeRepository {
        fn base_strategy(&self, _exp: MovementExperience) -> MpsResult<BaseStrategyRecord> {
            Ok(BaseStrategyRecord {
                movement_experience: MovementExperience::ShoulderFreedom,
                primary_focus: MovementSystem::Shoulder,
                secondary_focus: MovementSystem::Thoracic,
                emphasis: BTreeMap::from([
                    ("Shoulder".to_string(), 40),
                    ("Thoracic".to_string(), 20),
                    ("BreathCore".to_string(), 15),
                    ("Hip".to_string(), 10),
                    ("Legs".to_string(), 10),
                    ("Balance".to_string(), 5),
                ]),
                objectives: vec!["Increase overhead reach range".to_string()],
                explanation_template: "Shoulder Freedom base strategy.".to_string(),
            })
        }

        fn observation_modifiers(
            &self,
            _obs: &[String],
        ) -> MpsResult<Vec<ObservationModifierRecord>> {
            Ok(vec![])
        }

        fn benchmarks(&self, _exp: MovementExperience) -> MpsResult<Vec<BenchmarkRecord>> {
            Ok(vec![BenchmarkRecord {
                id: "overhead_reach".to_string(),
                name: "Overhead Reach".to_string(),
                instruction: "Reach both arms overhead and note compensation.".to_string(),
                watch_points: vec!["Rib flare".to_string()],
            }])
        }

        fn exercises(&self) -> MpsResult<Vec<ExerciseRecord>> {
            Ok(vec![
                exercise("arm_raise", Equipment::Standing, &[ExerciseRole::Assess]),
                exercise("breath_reset", Equipment::Mat, &[ExerciseRole::Prepare]),
                exercise("cat_cow", Equipment::Mat, &[ExerciseRole::Restore]),
                exercise(
                    "arms_in_straps",
                    Equipment::Reformer,
                    &[ExerciseRole::Prime, ExerciseRole::Integrate],
                ),
                exercise(
                    "pulling_straps",
                    Equipment::Reformer,
                    &[ExerciseRole::Prime, ExerciseRole::Challenge],
                ),
                exercise("chair_push_down", Equipment::Chair, &[ExerciseRole::Prime]),
                exercise("swan_chair", Equipment::Chair, &[ExerciseRole::Prime]),
                exercise(
                    "mermaid_chair",
                    Equipment::Chair,
                    &[ExerciseRole::Integrate],
                ),
                exercise(
                    "standing_push",
                    Equipment::Chair,
                    &[ExerciseRole::Challenge],
                ),
                exercise(
                    "chair_balance",
                    Equipment::Chair,
                    &[ExerciseRole::Challenge],
                ),
                exercise(
                    "standing_reach",
                    Equipment::Standing,
                    &[ExerciseRole::Transfer],
                ),
            ])
        }
    }

    fn exercise(id: &str, equipment: Equipment, roles: &[ExerciseRole]) -> ExerciseRecord {
        ExerciseRecord {
            id: id.to_string(),
            name: id.replace('_', " "),
            description: String::new(),
            equipment,
            roles: roles.to_vec(),
            movement_systems: vec![MovementSystem::Shoulder, MovementSystem::Thoracic],
            experience_tags: vec![MovementExperience::ShoulderFreedom],
            objectives: vec!["Increase overhead reach range".to_string()],
            min_level: ClassLevel::Beginner,
            max_level: ClassLevel::Advanced,
            difficulty: 2,
            default_duration_minutes: 5,
            teaching_cues: vec!["Move with breath.".to_string()],
            regression: Some("Reduce range.".to_string()),
            progression: Some("Add tempo.".to_string()),
            contraindications: vec![],
        }
    }

    fn request(equipment: Vec<Equipment>) -> ClassRequest {
        ClassRequest {
            students: 4,
            movement_experience: MovementExperience::ShoulderFreedom,
            level: ClassLevel::BeginnerIntermediate,
            equipment,
            duration_minutes: 60,
            observations: vec![],
            group_safety: GroupSafety {
                contraindications: vec![],
                risk_policy: RiskPolicy::Balanced,
            },
        }
    }

    #[test]
    fn generates_all_phases_with_build_prime_count() {
        let plan = generate_class_plan(&request(vec![Equipment::Reformer]), &FakeRepository)
            .expect("generate");

        assert_eq!(plan.journey.len(), 7);
        let build = plan
            .journey
            .iter()
            .find(|phase| phase.phase == MovementJourneyPhase::Build)
            .unwrap();
        assert!((2..=4).contains(&build.exercises.len()));
    }

    #[test]
    fn selected_primary_apparatus_is_enforced() {
        let plan =
            generate_class_plan(&request(vec![Equipment::Chair]), &FakeRepository).expect("plan");

        assert!(!plan
            .journey
            .iter()
            .flat_map(|phase| &phase.exercises)
            .any(|exercise| exercise.apparatus == Equipment::Reformer));
        assert!(plan
            .journey
            .iter()
            .flat_map(|phase| &phase.exercises)
            .any(|exercise| exercise.apparatus == Equipment::Chair));
    }

    #[test]
    fn hard_excluded_exercises_are_reported_and_not_selected() {
        struct SafetyRepo;
        impl MpsRepository for SafetyRepo {
            fn base_strategy(&self, exp: MovementExperience) -> MpsResult<BaseStrategyRecord> {
                FakeRepository.base_strategy(exp)
            }
            fn observation_modifiers(
                &self,
                obs: &[String],
            ) -> MpsResult<Vec<ObservationModifierRecord>> {
                FakeRepository.observation_modifiers(obs)
            }
            fn benchmarks(&self, exp: MovementExperience) -> MpsResult<Vec<BenchmarkRecord>> {
                FakeRepository.benchmarks(exp)
            }
            fn exercises(&self) -> MpsResult<Vec<ExerciseRecord>> {
                let mut exercises = FakeRepository.exercises()?;
                exercises
                    .iter_mut()
                    .find(|exercise| exercise.id == "standing_push")
                    .unwrap()
                    .contraindications
                    .push(ContraindicationRecord {
                        tag: "wrist_pain".to_string(),
                        severity: SafetySeverity::HardExclude,
                        note: "Avoid loaded wrist pressure.".to_string(),
                    });
                Ok(exercises)
            }
        }

        let mut req = request(vec![Equipment::Chair]);
        req.group_safety.contraindications = vec!["wrist_pain".to_string()];
        let plan = generate_class_plan(&req, &SafetyRepo).expect("plan");

        assert!(!plan
            .journey
            .iter()
            .flat_map(|phase| &phase.exercises)
            .any(|exercise| exercise.exercise_id == "standing_push"));
        assert!(plan
            .safety_summary
            .excluded_exercises
            .iter()
            .any(|exercise| exercise.exercise_id == "standing_push"));
    }
}
