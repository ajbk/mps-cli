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
    out.push_str(&format!(
        "**Primary Focus:** {:?}  \n",
        plan.movement_strategy.primary_focus
    ));
    out.push_str(&format!(
        "**Secondary Focus:** {:?}\n\n",
        plan.movement_strategy.secondary_focus
    ));
    out.push_str("| System | Emphasis |\n|---|---:|\n");
    for (system, value) in &plan.movement_strategy.emphasis {
        out.push_str(&format!("| {} | {}% |\n", system, value));
    }
    out.push_str("\n**Why this strategy:**  \n");
    out.push_str(&plan.movement_strategy.explanation);
    out.push_str("\n\n---\n\n## Before Class Benchmark\n\n");

    for assessment in &plan.benchmark.assessments {
        out.push_str(&format!(
            "### {}\n\n{}\n\n",
            assessment.name, assessment.instruction
        ));
        out.push_str("**Watch for:**\n");
        for point in &assessment.what_to_watch {
            out.push_str(&format!("- {}\n", point));
        }
        out.push('\n');
    }

    out.push_str("---\n\n## Class Journey\n\n");
    for phase in &plan.journey {
        out.push_str(&format!(
            "### {} — {} min\n\n",
            phase.phase.label(),
            phase.target_duration_minutes
        ));
        out.push_str(&format!("{}\n\n", phase.purpose));
        for exercise in &phase.exercises {
            out.push_str(&format!(
                "**{}** | {:?} | {} min | Role: {:?}\n\n",
                exercise.name, exercise.apparatus, exercise.duration_minutes, exercise.role
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

    out.push_str("---\n\n## Safety Notes\n\n");
    out.push_str(&format!(
        "**Risk Policy:** {:?}\n\n",
        plan.safety_summary.risk_policy
    ));
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
        out.push_str(&format!(
            "### {}\n\n{}\n\n",
            assessment.name, assessment.instruction
        ));
    }
    out.push_str(&format!(
        "**Expected improvement:**  \n{}\n",
        plan.expected_improvement
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        class_plan::*, ClassLevel, Equipment, ExerciseRole, MovementExperience,
        MovementJourneyPhase, MovementSystem, RiskPolicy,
    };
    use std::collections::BTreeMap;

    fn sample_plan() -> ClassPlan {
        ClassPlan {
            class_title: "Test Total Body Class".to_string(),
            movement_experience: MovementExperience::ShoulderFreedom,
            duration_minutes: 55,
            level: ClassLevel::Intermediate,
            students: 8,
            equipment: vec![Equipment::Reformer, Equipment::Mat],
            movement_strategy: MovementStrategyPlan {
                primary_focus: MovementSystem::Spine,
                secondary_focus: MovementSystem::Shoulder,
                emphasis: BTreeMap::from([
                    ("Spine".to_string(), 40),
                    ("Shoulder".to_string(), 30),
                    ("BreathCore".to_string(), 30),
                ]),
                key_objectives: vec!["Improve thoracic rotation".to_string()],
                preferred_exercise_objectives: vec!["Spinal articulation".to_string()],
                explanation: "Shoulder freedom experience favors spinal and shoulder work."
                    .to_string(),
            },
            benchmark: BenchmarkPlan {
                assessments: vec![AssessmentPlan {
                    name: "Seated Rotation".to_string(),
                    instruction: "Sit tall, rotate right, note range.".to_string(),
                    what_to_watch: vec!["Ribcage shifting".to_string(), "Hip hiking".to_string()],
                }],
            },
            journey: vec![JourneyPhasePlan {
                phase: MovementJourneyPhase::Arrive,
                purpose: "Centre and connect to breath.".to_string(),
                target_duration_minutes: 5,
                exercises: vec![ExerciseTeachingUnit {
                    exercise_id: "arr-001".to_string(),
                    name: "Breath Reset".to_string(),
                    apparatus: Equipment::Mat,
                    role: ExerciseRole::Assess,
                    duration_minutes: 3,
                    movement_objectives: vec!["Diaphragmatic breath".to_string()],
                    why_selected: "Establishes baseline breathing pattern.".to_string(),
                    teaching_cues: vec!["Inhale through nose".to_string()],
                    regression: None,
                    progression: Some("Add lateral rib expansion".to_string()),
                    safety_notes: vec![],
                }],
            }],
            safety_summary: SafetySummary {
                risk_policy: RiskPolicy::Balanced,
                applied_contraindications: vec![],
                excluded_exercises: vec![],
                modified_exercises: vec![],
                safety_notes: vec!["Monitor shoulder range for impingement signs.".to_string()],
            },
            retest: RetestPlan {
                assessments: vec![AssessmentPlan {
                    name: "Seated Rotation".to_string(),
                    instruction: "Repeat seated rotation, compare to baseline.".to_string(),
                    what_to_watch: vec!["Improved range".to_string()],
                }],
                expected_improvement: "5-10 degrees more rotation".to_string(),
            },
            expected_improvement: "Improved thoracic rotation and shoulder mobility".to_string(),
            warnings: vec![],
        }
    }

    #[test]
    fn render_markdown_contains_all_key_headings() {
        let plan = sample_plan();
        let md = render_markdown(&plan);
        for heading in [
            "Movement Strategy",
            "Before Class Benchmark",
            "Class Journey",
            "Safety Notes",
            "After Class Retest",
        ] {
            assert!(md.contains(heading), "Missing heading: {heading}");
        }
    }

    #[test]
    fn render_markdown_contains_class_title() {
        let plan = sample_plan();
        let md = render_markdown(&plan);
        assert!(md.contains("# Test Total Body Class"));
    }

    #[test]
    fn class_plan_serde_round_trip() {
        let plan = sample_plan();
        let json = serde_json::to_string(&plan).expect("serialize");
        let decoded: ClassPlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.class_title, plan.class_title);
        assert_eq!(decoded.duration_minutes, plan.duration_minutes);
        assert_eq!(decoded.level, plan.level);
        assert_eq!(decoded.students, plan.students);
        assert_eq!(decoded.journey.len(), plan.journey.len());
        assert_eq!(
            decoded.safety_summary.risk_policy,
            plan.safety_summary.risk_policy
        );
    }
}
