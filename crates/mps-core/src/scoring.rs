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
    if roles.contains(&target_role) {
        20
    } else {
        0
    }
}

pub fn objective_match_score(exercise_objectives: &[String], preferred: &[String]) -> i32 {
    exercise_objectives
        .iter()
        .filter(|obj| preferred.iter().any(|p| p.eq_ignore_ascii_case(obj)))
        .count() as i32
        * 5
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
