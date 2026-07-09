use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementExperience {
    ShoulderFreedom,
    HappyHips,
    SpineReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementSystem {
    BreathCore,
    Spine,
    Shoulder,
    Hip,
    Legs,
    Balance,
    Thoracic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ClassLevel {
    Beginner,
    BeginnerIntermediate,
    Intermediate,
    IntermediateAdvanced,
    Advanced,
}

impl ClassLevel {
    pub fn numeric(self) -> u8 {
        match self {
            Self::Beginner => 1,
            Self::BeginnerIntermediate => 2,
            Self::Intermediate => 3,
            Self::IntermediateAdvanced => 4,
            Self::Advanced => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Equipment {
    Reformer,
    Chair,
    Mat,
    Standing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentCategory {
    Apparatus,
    MovementContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementJourneyPhase {
    Arrive,
    Prepare,
    Build,
    Integrate,
    Challenge,
    Transfer,
    ResetRetest,
}

impl MovementJourneyPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Arrive => "ARRIVE",
            Self::Prepare => "PREPARE",
            Self::Build => "BUILD",
            Self::Integrate => "INTEGRATE",
            Self::Challenge => "CHALLENGE",
            Self::Transfer => "TRANSFER",
            Self::ResetRetest => "RESET & RETEST",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExerciseRole {
    Assess,
    Prepare,
    Prime,
    Integrate,
    Challenge,
    Transfer,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskPolicy {
    Conservative,
    Balanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetySeverity {
    HardExclude,
    Caution,
    RequireRegression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChallengeType {
    Balance,
    Coordination,
    Rotation,
    Standing,
    Complexity,
    Range,
    Load,
    Tempo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_level_numeric_maps_beginner_to_advanced() {
        assert_eq!(ClassLevel::Beginner.numeric(), 1);
        assert_eq!(ClassLevel::BeginnerIntermediate.numeric(), 2);
        assert_eq!(ClassLevel::Intermediate.numeric(), 3);
        assert_eq!(ClassLevel::IntermediateAdvanced.numeric(), 4);
        assert_eq!(ClassLevel::Advanced.numeric(), 5);
    }

    #[test]
    fn movement_journey_phase_labels_are_exact() {
        assert_eq!(MovementJourneyPhase::Arrive.label(), "ARRIVE");
        assert_eq!(MovementJourneyPhase::Prepare.label(), "PREPARE");
        assert_eq!(MovementJourneyPhase::Build.label(), "BUILD");
        assert_eq!(MovementJourneyPhase::Integrate.label(), "INTEGRATE");
        assert_eq!(MovementJourneyPhase::Challenge.label(), "CHALLENGE");
        assert_eq!(MovementJourneyPhase::Transfer.label(), "TRANSFER");
        assert_eq!(MovementJourneyPhase::ResetRetest.label(), "RESET & RETEST");
    }

    #[test]
    fn serde_round_trips_core_enums() {
        let experience = MovementExperience::ShoulderFreedom;
        let encoded = serde_json::to_string(&experience).unwrap();
        assert_eq!(encoded, "\"ShoulderFreedom\"");
        let decoded: MovementExperience = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, experience);

        let risk_policy = RiskPolicy::Conservative;
        let encoded = serde_json::to_string(&risk_policy).unwrap();
        assert_eq!(encoded, "\"Conservative\"");
        let decoded: RiskPolicy = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, risk_policy);
    }
}
