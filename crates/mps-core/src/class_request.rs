use serde::{Deserialize, Serialize};

use crate::{ClassLevel, Equipment, MovementExperience, MpsError, MpsResult, RiskPolicy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassRequest {
    pub students: u32,
    pub movement_experience: MovementExperience,
    pub level: ClassLevel,
    pub equipment: Vec<Equipment>,
    pub duration_minutes: u32,
    pub observations: Vec<String>,
    pub group_safety: GroupSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSafety {
    pub contraindications: Vec<String>,
    pub risk_policy: RiskPolicy,
}

impl ClassRequest {
    pub fn validate(&self) -> MpsResult<()> {
        if !matches!(self.duration_minutes, 45 | 60 | 75) {
            return Err(MpsError::UnsupportedDuration {
                requested: self.duration_minutes,
            });
        }

        if self.equipment.is_empty() {
            return Err(MpsError::UnsupportedEquipment(
                "at least one primary apparatus is required".to_string(),
            ));
        }

        for item in &self.equipment {
            match item {
                Equipment::Reformer | Equipment::Chair => {}
                Equipment::Mat | Equipment::Standing => {
                    return Err(MpsError::UnsupportedEquipment(format!(
                        "{:?} is a movement context, not primary apparatus",
                        item
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> ClassRequest {
        ClassRequest {
            students: 8,
            movement_experience: MovementExperience::SpineReset,
            level: ClassLevel::Intermediate,
            equipment: vec![Equipment::Reformer],
            duration_minutes: 60,
            observations: vec!["prefers slower transitions".to_string()],
            group_safety: GroupSafety {
                contraindications: vec!["wrist sensitivity".to_string()],
                risk_policy: RiskPolicy::Balanced,
            },
        }
    }

    #[test]
    fn valid_request_is_accepted() {
        assert!(valid_request().validate().is_ok());
    }

    #[test]
    fn unsupported_duration_is_rejected() {
        let mut request = valid_request();
        request.duration_minutes = 50;

        let err = request.validate().unwrap_err();

        match err {
            MpsError::UnsupportedDuration { requested } => assert_eq!(requested, 50),
            other => panic!("expected UnsupportedDuration, got {other:?}"),
        }
    }

    #[test]
    fn movement_context_as_primary_equipment_is_rejected() {
        let mut request = valid_request();
        request.equipment = vec![Equipment::Mat];

        let err = request.validate().unwrap_err();

        match err {
            MpsError::UnsupportedEquipment(message) => {
                assert_eq!(
                    message,
                    "Mat is a movement context, not primary apparatus".to_string()
                );
            }
            other => panic!("expected UnsupportedEquipment, got {other:?}"),
        }
    }

    #[test]
    fn empty_equipment_is_rejected() {
        let mut request = valid_request();
        request.equipment = vec![];

        let err = request.validate().unwrap_err();

        match err {
            MpsError::UnsupportedEquipment(message) => {
                assert_eq!(message, "at least one primary apparatus is required");
            }
            other => panic!("expected UnsupportedEquipment, got {other:?}"),
        }
    }

    #[test]
    fn json_serde_round_trips_valid_class_request() {
        let request = valid_request();

        let json = serde_json::to_string(&request).unwrap();
        let decoded: ClassRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.students, request.students);
        assert_eq!(decoded.movement_experience, request.movement_experience);
        assert_eq!(decoded.level, request.level);
        assert_eq!(decoded.equipment, request.equipment);
        assert_eq!(decoded.duration_minutes, request.duration_minutes);
        assert_eq!(decoded.observations, request.observations);
        assert_eq!(
            decoded.group_safety.contraindications,
            request.group_safety.contraindications
        );
        assert_eq!(
            decoded.group_safety.risk_policy,
            request.group_safety.risk_policy
        );
        assert!(decoded.validate().is_ok());
    }
}
