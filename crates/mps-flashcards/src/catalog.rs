use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CanonicalCatalog {
    pub exercises: Vec<CatalogExercise>,
}

impl CanonicalCatalog {
    pub fn load_json(json: &str) -> Result<Self, CatalogError> {
        let source: CatalogSource = serde_json::from_str(json)?;
        Ok(Self {
            exercises: source.tables.exercise_master,
        })
    }

    pub fn find(&self, exercise_id: &str) -> Option<&CatalogExercise> {
        self.exercises
            .iter()
            .find(|exercise| exercise.id == exercise_id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("invalid canonical catalog JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct CatalogSource {
    tables: CatalogTables,
}

#[derive(Debug, Deserialize)]
struct CatalogTables {
    #[serde(rename = "exerciseMaster")]
    exercise_master: Vec<CatalogExercise>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CatalogExercise {
    #[serde(rename = "sourceRow")]
    pub source_row: i64,
    pub id: String,
    pub exercise: String,
    pub apparatus: String,
    #[serde(rename = "equipmentKey")]
    pub equipment_key: String,
    #[serde(rename = "familyIds")]
    pub family_ids: Vec<String>,
    #[serde(rename = "familyNames")]
    pub family_names: Vec<String>,
    pub categories: Vec<String>,
    #[serde(rename = "bodyRegions")]
    pub body_regions: Vec<String>,
    #[serde(rename = "movementGoals")]
    pub movement_goals: Vec<String>,
    #[serde(rename = "progressionStages")]
    pub progression_stages: Vec<String>,
    #[serde(rename = "sourceRelationships")]
    pub source_relationships: Vec<String>,
    #[serde(rename = "sourcePages")]
    pub source_pages: Vec<i64>,
    pub mps: serde_json::Value,
    pub readiness: serde_json::Value,
}
