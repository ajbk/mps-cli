mod catalog;

pub use catalog::{CanonicalCatalog, CatalogError, CatalogExercise};

use serde::{Deserialize, Serialize};

pub const LOCKED_STYLE_PROFILE: &str = "mono-gesture-ink-pilates-v1";

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlashcardStatus {
    Draft,
    Generating,
    NeedsReview,
    RevisionRequested,
    Approved,
    Published,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FlashcardCard {
    pub id: String,
    pub source_exercise_id: String,
    pub category: String,
    pub teaching_copy_json: serde_json::Value,
    pub style_profile: String,
    pub character_id: String,
    pub current_asset_id: Option<String>,
    pub version: i64,
    pub status: FlashcardStatus,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct VisualBrief {
    pub id: String,
    pub card_id: String,
    pub exercise_id: String,
    pub style_profile: String,
    pub character_id: String,
    pub outfit: String,
    pub pose_json: serde_json::Value,
    pub apparatus: String,
    pub palette_json: serde_json::Value,
    pub must_show_json: serde_json::Value,
    pub must_not_show_json: serde_json::Value,
    pub version: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AssetReview {
    pub asset_id: String,
    pub passed: bool,
    pub findings_json: serde_json::Value,
    pub validator_version: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FlashcardError {
    #[error("source exercise ID must not be empty")]
    EmptyExerciseId,
    #[error("source exercise not found: {0}")]
    ExerciseNotFound(String),
    #[error("new flashcard must start in draft status")]
    InvalidDraftStatus,
    #[error("unsupported style profile: {0}")]
    UnsupportedStyleProfile(String),
}

pub fn validate_new_draft(
    card: &FlashcardCard,
    catalog: &CanonicalCatalog,
) -> Result<(), FlashcardError> {
    if card.source_exercise_id.trim().is_empty() {
        return Err(FlashcardError::EmptyExerciseId);
    }
    if card.status != FlashcardStatus::Draft {
        return Err(FlashcardError::InvalidDraftStatus);
    }
    if card.style_profile != LOCKED_STYLE_PROFILE {
        return Err(FlashcardError::UnsupportedStyleProfile(
            card.style_profile.clone(),
        ));
    }
    if catalog.find(&card.source_exercise_id).is_none() {
        return Err(FlashcardError::ExerciseNotFound(
            card.source_exercise_id.clone(),
        ));
    }
    Ok(())
}

pub fn can_transition(from: FlashcardStatus, to: FlashcardStatus) -> bool {
    matches!(
        (from, to),
        (FlashcardStatus::Draft, FlashcardStatus::Generating)
            | (FlashcardStatus::Generating, FlashcardStatus::NeedsReview)
            | (FlashcardStatus::NeedsReview, FlashcardStatus::RevisionRequested)
            | (FlashcardStatus::NeedsReview, FlashcardStatus::Approved)
            | (FlashcardStatus::RevisionRequested, FlashcardStatus::Generating)
            | (FlashcardStatus::Approved, FlashcardStatus::Published)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn catalog() -> CanonicalCatalog {
        let json = include_str!("../../../data/reference/mps_database_v1_core.json");
        CanonicalCatalog::load_json(json).expect("canonical catalog should load")
    }

    fn draft(source_exercise_id: &str) -> FlashcardCard {
        FlashcardCard {
            id: "card-1".into(),
            source_exercise_id: source_exercise_id.into(),
            category: "Spinal Mobility".into(),
            teaching_copy_json: json!({"cue": "Breathe"}),
            style_profile: "mono-gesture-ink-pilates-v1".into(),
            character_id: "teacher-01".into(),
            current_asset_id: None,
            version: 1,
            status: FlashcardStatus::Draft,
        }
    }

    #[test]
    fn accepts_a_new_draft_with_an_existing_source_exercise() {
        let card = draft("source_chair_achilles_stretch_row_4");

        assert!(validate_new_draft(&card, &catalog()).is_ok());
    }

    #[test]
    fn rejects_a_new_draft_with_an_empty_source_exercise_id() {
        let card = draft("");

        assert!(matches!(
            validate_new_draft(&card, &catalog()),
            Err(FlashcardError::EmptyExerciseId)
        ));
    }

    #[test]
    fn does_not_allow_published_cards_to_return_to_draft() {
        assert!(!can_transition(
            FlashcardStatus::Published,
            FlashcardStatus::Draft
        ));
    }

    #[test]
    fn round_trips_a_visual_brief_with_locked_style_and_cheek_accent() {
        let brief = VisualBrief {
            id: "brief-1".into(),
            card_id: "card-1".into(),
            exercise_id: "source_chair_achilles_stretch_row_4".into(),
            style_profile: "mono-gesture-ink-pilates-v1".into(),
            character_id: "teacher-01".into(),
            outfit: "off-white crop top and charcoal biker shorts".into(),
            pose_json: json!({"position": "standing"}),
            apparatus: "Chair".into(),
            palette_json: json!({"cheekAccent": "#D98F9A"}),
            must_show_json: json!(["neutral lumbar position"]),
            must_not_show_json: json!(["arrows", "text"]),
            version: 1,
        };

        let encoded = serde_json::to_string(&brief).expect("brief should serialize");
        let decoded: VisualBrief =
            serde_json::from_str(&encoded).expect("brief should deserialize");

        assert_eq!(decoded.style_profile, "mono-gesture-ink-pilates-v1");
        assert_eq!(decoded.palette_json["cheekAccent"], "#D98F9A");
    }

    #[test]
    fn loads_the_canonical_export_and_finds_a_source_exercise() {
        let found = catalog()
            .find("source_chair_achilles_stretch_row_4")
            .expect("known source exercise should exist");

        assert_eq!(found.exercise, "Achilles Stretch");
        assert_eq!(found.apparatus, "Chair");
    }
}
