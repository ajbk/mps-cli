use crate::{ClassLevel, ExerciseRecord, ExerciseRole, MovementExperience, MovementSystem};

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

pub fn experience_match_score(
    exercise_experiences: &[MovementExperience],
    target: MovementExperience,
) -> i32 {
    if exercise_experiences.contains(&target) {
        10
    } else {
        0
    }
}

pub fn movement_system_score(
    exercise_systems: &[MovementSystem],
    primary: MovementSystem,
    secondary: MovementSystem,
) -> i32 {
    let mut score = 0;
    if exercise_systems.contains(&primary) {
        score += 8;
    }
    if exercise_systems.contains(&secondary) {
        score += 4;
    }
    score
}

pub fn objective_match_score(exercise_objectives: &[String], preferred: &[String]) -> i32 {
    exercise_objectives
        .iter()
        .filter(|obj| {
            preferred.iter().any(|p| {
                p.eq_ignore_ascii_case(obj) || obj.to_lowercase().contains(&p.to_lowercase())
            })
        })
        .count() as i32
        * 5
}

pub fn score_exercise(
    exercise: &ExerciseRecord,
    role: ExerciseRole,
    level: ClassLevel,
    experience: MovementExperience,
    primary: MovementSystem,
    secondary: MovementSystem,
    preferred_objectives: &[String],
) -> i32 {
    role_match_score(&exercise.roles, role)
        + difficulty_fit_score(exercise.difficulty, level)
        + experience_match_score(&exercise.experience_tags, experience)
        + movement_system_score(&exercise.movement_systems, primary, secondary)
        + objective_match_score(&exercise.objectives, preferred_objectives)
}

pub fn role_match_score(roles: &[ExerciseRole], target_role: ExerciseRole) -> i32 {
    if roles.contains(&target_role) {
        20
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClassLevel, ExerciseRole};

    #[test]
    fn exact_level_scores_higher_than_too_easy() {
        assert!(
            difficulty_fit_score(2, ClassLevel::BeginnerIntermediate)
                > difficulty_fit_score(1, ClassLevel::BeginnerIntermediate)
        );
    }

    #[test]
    fn too_hard_scores_zero() {
        assert_eq!(difficulty_fit_score(5, ClassLevel::BeginnerIntermediate), 0);
    }

    #[test]
    fn role_match_scores_positive() {
        assert_eq!(
            role_match_score(&[ExerciseRole::Prime], ExerciseRole::Prime),
            20
        );
    }

    #[test]
    fn role_mismatch_scores_zero() {
        assert_eq!(
            role_match_score(&[ExerciseRole::Prepare], ExerciseRole::Prime),
            0
        );
    }

    #[test]
    fn objective_match_scores_per_match() {
        let exercise_obj = vec![
            "Shoulder Mobility".to_string(),
            "Thoracic Extension".to_string(),
        ];
        let preferred = vec!["Shoulder Mobility".to_string()];
        assert_eq!(objective_match_score(&exercise_obj, &preferred), 5);
    }
}
