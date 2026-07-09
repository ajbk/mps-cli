use crate::{BaseStrategyRecord, MovementStrategyPlan, ObservationModifierRecord};
use std::collections::BTreeMap;

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
    let clipped: Vec<(String, u32)> = input
        .into_iter()
        .map(|(key, value)| (key, value.max(0) as u32))
        .collect();
    let sum: u32 = clipped.iter().map(|(_, value)| *value).sum();

    if sum == 0 {
        return BTreeMap::new();
    }

    let mut rows: Vec<(String, u32, f64)> = clipped
        .into_iter()
        .map(|(key, value)| {
            let exact = (value as f64 / sum as f64) * 100.0;
            (key, exact.floor() as u32, exact.fract())
        })
        .collect();

    let mut remainder = 100u32 - rows.iter().map(|(_, pct, _)| *pct).sum::<u32>();
    rows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    for (_, pct, _) in &mut rows {
        if remainder == 0 {
            break;
        }
        *pct += 1;
        remainder -= 1;
    }

    let mut normalized = BTreeMap::new();
    for (key, pct, _) in rows {
        normalized.insert(key, pct);
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BaseStrategyRecord, MovementExperience, MovementSystem, ObservationModifierRecord,
    };
    use std::collections::BTreeMap;

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
        assert!(strategy
            .key_objectives
            .contains(&"Restore thoracic extension".to_string()));
    }

    #[test]
    fn no_modifiers_returns_base_strategy_normalized() {
        let base = BaseStrategyRecord {
            movement_experience: MovementExperience::HappyHips,
            primary_focus: MovementSystem::Hip,
            secondary_focus: MovementSystem::Legs,
            emphasis: BTreeMap::from([
                ("Hip".to_string(), 40),
                ("Legs".to_string(), 25),
                ("Core".to_string(), 20),
                ("Spine".to_string(), 10),
                ("Balance".to_string(), 5),
            ]),
            objectives: vec!["Improve hip mobility".to_string()],
            explanation_template: "Base hip explanation.".to_string(),
        };

        let strategy = build_strategy(base, vec![]);
        let total: u32 = strategy.emphasis.values().sum();
        assert_eq!(total, 100);
        assert_eq!(strategy.explanation, "Base hip explanation.");
    }

    #[test]
    fn negative_emphasis_clips_to_zero() {
        let base = BaseStrategyRecord {
            movement_experience: MovementExperience::SpineReset,
            primary_focus: MovementSystem::Spine,
            secondary_focus: MovementSystem::BreathCore,
            emphasis: BTreeMap::from([
                ("Spine".to_string(), 50),
                ("Core".to_string(), 30),
                ("Shoulder".to_string(), 10),
                ("Hip".to_string(), 10),
            ]),
            objectives: vec![],
            explanation_template: "Spine reset.".to_string(),
        };
        let modifier = ObservationModifierRecord {
            id: "shoulder_exclusion".to_string(),
            emphasis_adjustments: BTreeMap::from([("Shoulder".to_string(), -20)]),
            added_objectives: vec![],
            preferred_exercise_objectives: vec![],
            explanation_fragment: "Reduced shoulder.".to_string(),
        };

        let strategy = build_strategy(base, vec![modifier]);
        let total: u32 = strategy.emphasis.values().sum();
        assert_eq!(total, 100);
    }
}
