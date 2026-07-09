use crate::ClassPlan;

pub fn render_markdown(plan: &ClassPlan) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", plan.class_title));
    out.push_str("## Class Summary\n\n");
    out.push_str(&format!(
        "- Movement experience: {}\n",
        plan.movement_experience.label()
    ));
    out.push_str(&format!("- Duration: {} min\n", plan.duration_minutes));
    out.push_str(&format!("- Level: {}\n", plan.level.label()));
    out.push_str(&format!("- Students: {}\n", plan.students));
    out.push_str(&format!(
        "- Equipment: {}\n\n",
        plan.equipment
            .iter()
            .map(|equipment| equipment.label())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    out.push_str("> Teaching aid only. This is not medical advice or diagnosis; instructor judgement is required.\n\n");

    out.push_str("## Movement Strategy\n\n");
    out.push_str(&format!(
        "- Primary focus: {}\n",
        plan.movement_strategy.primary_focus.label()
    ));
    out.push_str(&format!(
        "- Secondary focus: {}\n\n",
        plan.movement_strategy.secondary_focus.label()
    ));
    out.push_str("| System | Emphasis |\n|---|---:|\n");
    for (system, value) in &plan.movement_strategy.emphasis {
        out.push_str(&format!("| {} | {}% |\n", system, value));
    }
    out.push_str("\n");
    out.push_str(&plan.movement_strategy.explanation);
    out.push_str("\n\n### Key Objectives\n\n");
    for objective in &plan.movement_strategy.key_objectives {
        out.push_str(&format!("- {}\n", objective));
    }

    out.push_str("\n## Before Class Benchmark\n\n");
    for assessment in &plan.benchmark.assessments {
        out.push_str(&format!(
            "### {}\n\n{}\n\n",
            assessment.name, assessment.instruction
        ));
        out.push_str("Watch for:\n");
        for point in &assessment.what_to_watch {
            out.push_str(&format!("- {}\n", point));
        }
        out.push('\n');
    }

    out.push_str("## Class Journey\n\n");
    for phase in &plan.journey {
        out.push_str(&format!(
            "### {} - {} min\n\n{}\n\n",
            phase.phase.label(),
            phase.target_duration_minutes,
            phase.purpose
        ));
        for exercise in &phase.exercises {
            out.push_str(&format!(
                "**{}** | {} | {} min | {}\n\n",
                exercise.name,
                exercise.apparatus.label(),
                exercise.duration_minutes,
                format!("{:?}", exercise.role)
            ));
            out.push_str(&format!(
                "- Objectives: {}\n",
                exercise.movement_objectives.join(", ")
            ));
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

    out.push_str("## Safety Summary\n\n");
    out.push_str(&format!(
        "- Risk policy: {:?}\n",
        plan.safety_summary.risk_policy
    ));
    if !plan.safety_summary.applied_contraindications.is_empty() {
        out.push_str("- Applied contraindications: ");
        out.push_str(&plan.safety_summary.applied_contraindications.join(", "));
        out.push('\n');
    }
    for item in &plan.safety_summary.excluded_exercises {
        out.push_str(&format!("- Excluded {}: {}\n", item.name, item.reason));
    }
    for item in &plan.safety_summary.modified_exercises {
        out.push_str(&format!("- Modified {}: {}\n", item.name, item.reason));
    }
    for note in &plan.safety_summary.safety_notes {
        out.push_str(&format!("- {}\n", note));
    }

    out.push_str("\n## After Class Retest\n\n");
    for assessment in &plan.retest.assessments {
        out.push_str(&format!(
            "### {}\n\n{}\n\n",
            assessment.name, assessment.instruction
        ));
    }
    out.push_str(&format!(
        "**Expected improvement:** {}\n\n",
        plan.retest.expected_improvement
    ));

    if !plan.warnings.is_empty() {
        out.push_str("## Engine Warnings\n\n");
        for warning in &plan.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssessmentPlan, BenchmarkPlan, ClassLevel, ClassPlan, Equipment, ExerciseRole,
        ExerciseTeachingUnit, JourneyPhasePlan, MovementExperience, MovementJourneyPhase,
        MovementStrategyPlan, MovementSystem, RetestPlan, RiskPolicy, SafetySummary,
    };
    use std::collections::BTreeMap;

    fn sample_plan() -> ClassPlan {
        ClassPlan {
            class_title: "Shoulder Freedom - 60 min Beginner Intermediate".to_string(),
            movement_experience: MovementExperience::ShoulderFreedom,
            duration_minutes: 60,
            level: ClassLevel::BeginnerIntermediate,
            students: 8,
            equipment: vec![Equipment::Reformer],
            movement_strategy: MovementStrategyPlan {
                primary_focus: MovementSystem::Shoulder,
                secondary_focus: MovementSystem::Thoracic,
                emphasis: BTreeMap::from([
                    ("Shoulder".to_string(), 45),
                    ("Thoracic".to_string(), 25),
                    ("BreathCore".to_string(), 30),
                ]),
                key_objectives: vec!["Increase overhead reach range".to_string()],
                preferred_exercise_objectives: vec!["Increase overhead reach range".to_string()],
                explanation: "Shoulder strategy explanation.".to_string(),
            },
            benchmark: BenchmarkPlan {
                assessments: vec![AssessmentPlan {
                    name: "Overhead Reach".to_string(),
                    instruction: "Reach both arms overhead.".to_string(),
                    what_to_watch: vec!["Rib flare".to_string()],
                }],
            },
            journey: vec![JourneyPhasePlan {
                phase: MovementJourneyPhase::Arrive,
                purpose: "Assess baseline.".to_string(),
                target_duration_minutes: 5,
                exercises: vec![ExerciseTeachingUnit {
                    exercise_id: "arm_raise".to_string(),
                    name: "Arm Raise".to_string(),
                    apparatus: Equipment::Standing,
                    role: ExerciseRole::Assess,
                    duration_minutes: 3,
                    movement_objectives: vec!["Assess overhead reach range".to_string()],
                    why_selected: "Matched Assess role.".to_string(),
                    teaching_cues: vec!["Reach without rib flare.".to_string()],
                    regression: None,
                    progression: None,
                    safety_notes: vec![],
                }],
            }],
            safety_summary: SafetySummary {
                risk_policy: RiskPolicy::Conservative,
                applied_contraindications: vec![],
                excluded_exercises: vec![],
                modified_exercises: vec![],
                safety_notes: vec!["Teaching aid only.".to_string()],
            },
            retest: RetestPlan {
                assessments: vec![AssessmentPlan {
                    name: "Overhead Reach".to_string(),
                    instruction: "Repeat the reach.".to_string(),
                    what_to_watch: vec!["Smoother range".to_string()],
                }],
                expected_improvement: "Clearer overhead reach.".to_string(),
            },
            expected_improvement: "Improved shoulder movement quality.".to_string(),
            warnings: vec![],
        }
    }

    #[test]
    fn render_markdown_contains_key_sections() {
        let markdown = render_markdown(&sample_plan());
        for text in [
            "Class Summary",
            "Movement Strategy",
            "Before Class Benchmark",
            "Class Journey",
            "Safety Summary",
            "After Class Retest",
        ] {
            assert!(markdown.contains(text), "missing {text}");
        }
    }

    #[test]
    fn class_plan_json_round_trips() {
        let plan = sample_plan();
        let json = serde_json::to_string(&plan).expect("serialize");
        let decoded: ClassPlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.class_title, plan.class_title);
        assert_eq!(decoded.journey.len(), plan.journey.len());
    }
}
