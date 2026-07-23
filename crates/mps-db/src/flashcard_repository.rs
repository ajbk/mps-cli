#[cfg(test)]
mod tests {
    use super::*;
    use mps_flashcards::{
        CanonicalCatalog, FlashcardCard, FlashcardStatus, VisualBrief, LOCKED_STYLE_PROFILE,
    };
    use serde_json::json;

    fn catalog() -> CanonicalCatalog {
        CanonicalCatalog::load_json(include_str!("../../../data/reference/mps_database_v1_core.json"))
            .expect("canonical catalog should load")
    }

    fn repository() -> FlashcardRepository {
        FlashcardRepository::open_in_memory(catalog()).expect("open flashcard repository")
    }

    fn card(id: &str) -> FlashcardCard {
        FlashcardCard {
            id: id.into(),
            source_exercise_id: "source_chair_achilles_stretch_row_4".into(),
            category: "Spinal Mobility".into(),
            teaching_copy_json: json!({"cue": "Breathe"}),
            style_profile: LOCKED_STYLE_PROFILE.into(),
            character_id: "teacher-01".into(),
            current_asset_id: None,
            version: 1,
            status: FlashcardStatus::Draft,
        }
    }

    fn brief(card_id: &str) -> VisualBrief {
        VisualBrief {
            id: "brief-1".into(),
            card_id: card_id.into(),
            exercise_id: "source_chair_achilles_stretch_row_4".into(),
            style_profile: LOCKED_STYLE_PROFILE.into(),
            character_id: "teacher-01".into(),
            outfit: "off-white crop top and charcoal biker shorts".into(),
            pose_json: json!({"position": "standing"}),
            apparatus: "Chair".into(),
            palette_json: json!({"cheekAccent": "#D98F9A"}),
            must_show_json: json!(["neutral lumbar position"]),
            must_not_show_json: json!(["arrows", "text"]),
            version: 1,
        }
    }

    #[test]
    fn creates_a_card_from_a_valid_source_exercise_id() {
        let repository = repository();
        let created = repository.create_flashcard(&card("card-1")).unwrap();

        assert_eq!(created.source_exercise_id, "source_chair_achilles_stretch_row_4");
        assert_eq!(repository.get_flashcard("card-1").unwrap(), Some(created));
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 1);
    }

    #[test]
    fn lists_cards_by_status_and_category() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        let mut other = card("card-2");
        other.category = "Shoulder".into();
        repository.create_flashcard(&other).unwrap();

        let cards = repository
            .list_flashcards(FlashcardListFilter {
                status: Some(FlashcardStatus::Draft),
                category: Some("Spinal Mobility".into()),
            })
            .unwrap();

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "card-1");
    }

    #[test]
    fn stores_a_versioned_brief_and_asset() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository.save_visual_brief(&brief("card-1")).unwrap();
        repository
            .record_asset(&FlashcardAsset {
                id: "asset-1".into(),
                card_id: "card-1".into(),
                brief_id: "brief-1".into(),
                repo_path: "docs/assets/asset-1.png".into(),
                provider_job_id: Some("provider-job-1".into()),
                version: 2,
                status: FlashcardAssetStatus::NeedsReview,
            })
            .unwrap();

        assert_eq!(repository.latest_brief_version("card-1").unwrap(), Some(1));
        assert_eq!(repository.asset_version("asset-1").unwrap(), Some(2));
        assert_eq!(
            repository
                .get_flashcard("card-1")
                .unwrap()
                .unwrap()
                .current_asset_id,
            Some("asset-1".into())
        );
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 3);
    }

    #[test]
    fn records_jobs_and_explicit_audit_events() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository
            .create_job(&FlashcardJob {
                id: "job-1".into(),
                card_id: "card-1".into(),
                kind: FlashcardJobKind::Generate,
                status: FlashcardJobStatus::Queued,
                input_json: json!({"brief_id": "brief-1"}),
                output_json: None,
                error: None,
            })
            .unwrap();
        repository
            .record_audit_event(
                "card-1",
                AuditActorKind::Teacher,
                Some("teacher-01"),
                "flashcard.note_added",
                &json!({"note": "Check elbow placement"}),
            )
            .unwrap();

        assert_eq!(repository.audit_event_count("card-1").unwrap(), 3);
    }

    #[test]
    fn rejects_publish_until_an_approved_card_has_a_passing_latest_review() {
        let repository = repository();
        repository.create_flashcard(&card("card-1")).unwrap();
        repository.transition_status("card-1", FlashcardStatus::Generating).unwrap();
        repository.transition_status("card-1", FlashcardStatus::NeedsReview).unwrap();
        repository.transition_status("card-1", FlashcardStatus::Approved).unwrap();

        let error = repository
            .transition_status("card-1", FlashcardStatus::Published)
            .expect_err("a card cannot publish without a review");
        assert!(error.to_string().contains("latest review"));

        repository.save_visual_brief(&brief("card-1")).unwrap();
        repository
            .record_asset(&FlashcardAsset {
                id: "asset-1".into(),
                card_id: "card-1".into(),
                brief_id: "brief-1".into(),
                repo_path: "docs/assets/asset-1.png".into(),
                provider_job_id: None,
                version: 1,
                status: FlashcardAssetStatus::Approved,
            })
            .unwrap();
        repository
            .record_review(&FlashcardReview {
                id: "review-1".into(),
                card_id: "card-1".into(),
                asset_id: "asset-1".into(),
                passed: true,
                findings_json: json!([]),
                reviewer_kind: ReviewerKind::Automated,
                reviewer_id: None,
            })
            .unwrap();

        repository
            .record_review(&FlashcardReview {
                id: "review-2".into(),
                card_id: "card-1".into(),
                asset_id: "asset-1".into(),
                passed: false,
                findings_json: json!(["elbow angle is unclear"]),
                reviewer_kind: ReviewerKind::Automated,
                reviewer_id: None,
            })
            .unwrap();
        assert!(repository
            .transition_status("card-1", FlashcardStatus::Published)
            .expect_err("a failed latest review must block publication")
            .to_string()
            .contains("latest review"));

        repository
            .record_review(&FlashcardReview {
                id: "review-3".into(),
                card_id: "card-1".into(),
                asset_id: "asset-1".into(),
                passed: true,
                findings_json: json!([]),
                reviewer_kind: ReviewerKind::Teacher,
                reviewer_id: Some("teacher-01".into()),
            })
            .unwrap();

        repository.transition_status("card-1", FlashcardStatus::Published).unwrap();
        assert_eq!(
            repository.get_flashcard("card-1").unwrap().unwrap().status,
            FlashcardStatus::Published
        );
        assert_eq!(repository.audit_event_count("card-1").unwrap(), 10);
    }
}
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use mps_flashcards::{
    can_transition, validate_new_draft, validate_visual_brief, CanonicalCatalog, FlashcardCard,
    FlashcardStatus, VisualBrief,
};
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{FromRow, SqlitePool};
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Default)]
pub struct FlashcardListFilter {
    pub status: Option<FlashcardStatus>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashcardAssetStatus {
    Generating,
    NeedsReview,
    Rejected,
    Approved,
}

#[derive(Debug, Clone)]
pub struct FlashcardAsset {
    pub id: String,
    pub card_id: String,
    pub brief_id: String,
    pub repo_path: String,
    pub provider_job_id: Option<String>,
    pub version: i64,
    pub status: FlashcardAssetStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashcardJobKind {
    Generate,
    Review,
    Regenerate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashcardJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct FlashcardJob {
    pub id: String,
    pub card_id: String,
    pub kind: FlashcardJobKind,
    pub status: FlashcardJobStatus,
    pub input_json: Value,
    pub output_json: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewerKind {
    Automated,
    Teacher,
}

#[derive(Debug, Clone)]
pub struct FlashcardReview {
    pub id: String,
    pub card_id: String,
    pub asset_id: String,
    pub passed: bool,
    pub findings_json: Value,
    pub reviewer_kind: ReviewerKind,
    pub reviewer_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditActorKind {
    Teacher,
    Chatgpt,
    System,
}

#[derive(Debug, FromRow)]
struct CardRow {
    id: String,
    source_exercise_id: String,
    category: String,
    teaching_copy_json: String,
    style_profile: String,
    character_id: String,
    current_asset_id: Option<String>,
    version: i64,
    status: String,
}

pub struct FlashcardRepository {
    pool: SqlitePool,
    runtime: Arc<Runtime>,
    catalog: CanonicalCatalog,
}

impl FlashcardRepository {
    pub fn open(path: &str, catalog: CanonicalCatalog) -> Result<Self> {
        let runtime = Runtime::new().context("create Tokio runtime")?;
        let database_url = format!("sqlite:{path}?mode=rwc");
        let pool = runtime.block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .with_context(|| format!("connect to SQLite database at {path}"))?;
            sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok::<_, anyhow::Error>(pool)
        })?;

        Ok(Self {
            pool,
            runtime: Arc::new(runtime),
            catalog,
        })
    }

    pub fn open_in_memory(catalog: CanonicalCatalog) -> Result<Self> {
        let runtime = Runtime::new().context("create Tokio runtime")?;
        let pool = runtime.block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .context("connect to in-memory SQLite")?;
            sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok::<_, anyhow::Error>(pool)
        })?;

        Ok(Self { pool, runtime: Arc::new(runtime), catalog })
    }

    pub fn create_flashcard(&self, card: &FlashcardCard) -> Result<FlashcardCard> {
        validate_new_draft(card, &self.catalog).map_err(|error| anyhow!(error))?;
        let card = card.clone();
        let timestamp = timestamp();
        self.runtime.block_on(async {
            sqlx::query("INSERT INTO flashcard_cards (id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&card.id).bind(&card.source_exercise_id).bind(&card.category)
                .bind(serde_json::to_string(&card.teaching_copy_json)?)
                .bind(&card.style_profile).bind(&card.character_id).bind(&card.current_asset_id)
                .bind(card.version).bind(status_to_db(card.status)).bind(&timestamp).bind(&timestamp)
                .execute(&self.pool).await?;
            self.record_system_audit_async(&card.id, "flashcard.created", json!({"version": card.version})).await
        })?;
        Ok(card)
    }

    pub fn get_flashcard(&self, id: &str) -> Result<Option<FlashcardCard>> {
        let id = id.to_owned();
        self.runtime.block_on(async {
            sqlx::query_as::<_, CardRow>("SELECT id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status FROM flashcard_cards WHERE id = ?")
                .bind(id).fetch_optional(&self.pool).await?.map(card_from_row).transpose()
        })
    }

    pub fn list_flashcards(&self, filter: FlashcardListFilter) -> Result<Vec<FlashcardCard>> {
        self.runtime.block_on(async {
            let mut query = String::from("SELECT id, source_exercise_id, category, teaching_copy_json, style_profile, character_id, current_asset_id, version, status FROM flashcard_cards WHERE 1 = 1");
            if filter.status.is_some() { query.push_str(" AND status = ?"); }
            if filter.category.is_some() { query.push_str(" AND category = ?"); }
            query.push_str(" ORDER BY created_at, id");
            let mut statement = sqlx::query_as::<_, CardRow>(&query);
            if let Some(status) = filter.status { statement = statement.bind(status_to_db(status)); }
            if let Some(category) = filter.category { statement = statement.bind(category); }
            statement.fetch_all(&self.pool).await?.into_iter().map(card_from_row).collect()
        })
    }

    pub fn save_visual_brief(&self, brief: &VisualBrief) -> Result<()> {
        validate_visual_brief(brief).map_err(|error| anyhow!(error))?;
        let brief = brief.clone();
        self.runtime.block_on(async {
            sqlx::query("INSERT INTO visual_briefs (id, card_id, exercise_id, brief_json, version, created_at) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(&brief.id).bind(&brief.card_id).bind(&brief.exercise_id)
                .bind(serde_json::to_string(&brief)?).bind(brief.version).bind(timestamp())
                .execute(&self.pool).await?;
            self.record_system_audit_async(&brief.card_id, "visual_brief.saved", json!({"brief_id": brief.id, "version": brief.version})).await
        })
    }

    pub fn create_job(&self, job: &FlashcardJob) -> Result<()> {
        self.runtime.block_on(async {
            let now = timestamp();
            sqlx::query("INSERT INTO flashcard_jobs (id, card_id, kind, status, input_json, output_json, error, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&job.id).bind(&job.card_id).bind(job_kind_to_db(&job.kind)).bind(job_status_to_db(&job.status))
                .bind(serde_json::to_string(&job.input_json)?).bind(job.output_json.as_ref().map(serde_json::to_string).transpose()?)
                .bind(&job.error).bind(&now).bind(&now).execute(&self.pool).await?;
            self.record_system_audit_async(&job.card_id, "flashcard_job.created", json!({"job_id": job.id})).await
        })
    }

    pub fn record_asset(&self, asset: &FlashcardAsset) -> Result<()> {
        self.runtime.block_on(async {
            sqlx::query("INSERT INTO flashcard_assets (id, card_id, brief_id, repo_path, provider_job_id, version, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&asset.id).bind(&asset.card_id).bind(&asset.brief_id).bind(&asset.repo_path)
                .bind(&asset.provider_job_id).bind(asset.version).bind(asset_status_to_db(&asset.status)).bind(timestamp())
                .execute(&self.pool).await?;
            sqlx::query("UPDATE flashcard_cards SET current_asset_id = ?, updated_at = ? WHERE id = ?")
                .bind(&asset.id).bind(timestamp()).bind(&asset.card_id).execute(&self.pool).await?;
            self.record_system_audit_async(&asset.card_id, "flashcard_asset.recorded", json!({"asset_id": asset.id, "version": asset.version})).await
        })
    }

    pub fn record_review(&self, review: &FlashcardReview) -> Result<()> {
        self.runtime.block_on(async {
            sqlx::query("INSERT INTO flashcard_reviews (id, card_id, asset_id, passed, findings_json, reviewer_kind, reviewer_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&review.id).bind(&review.card_id).bind(&review.asset_id).bind(review.passed)
                .bind(serde_json::to_string(&review.findings_json)?).bind(reviewer_kind_to_db(&review.reviewer_kind))
                .bind(&review.reviewer_id).bind(timestamp()).execute(&self.pool).await?;
            self.record_system_audit_async(&review.card_id, "flashcard_review.recorded", json!({"review_id": review.id, "passed": review.passed})).await
        })
    }

    pub fn transition_status(&self, card_id: &str, next: FlashcardStatus) -> Result<()> {
        let card = self.get_flashcard(card_id)?.ok_or_else(|| anyhow!("flashcard not found: {card_id}"))?;
        if !can_transition(card.status, next) { return Err(anyhow!("invalid flashcard status transition")); }
        if next == FlashcardStatus::Published && !self.latest_review_passed(card_id)? {
            return Err(anyhow!("cannot publish: latest review has not passed"));
        }
        let card_id = card_id.to_owned();
        self.runtime.block_on(async {
            sqlx::query("UPDATE flashcard_cards SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status_to_db(next)).bind(timestamp()).bind(&card_id).execute(&self.pool).await?;
            self.record_system_audit_async(&card_id, "flashcard.status_transitioned", json!({"from": status_to_db(card.status), "to": status_to_db(next)})).await
        })
    }

    pub fn record_audit_event(&self, card_id: &str, actor_kind: AuditActorKind, actor_id: Option<&str>, action: &str, payload: &Value) -> Result<()> {
        let card_id = card_id.to_owned();
        let actor_id = actor_id.map(str::to_owned);
        let action = action.to_owned();
        let payload = payload.clone();
        self.runtime.block_on(async { self.record_audit_async(&card_id, actor_kind, actor_id.as_deref(), &action, payload).await })
    }

    pub fn latest_brief_version(&self, card_id: &str) -> Result<Option<i64>> { self.scalar_i64("SELECT MAX(version) FROM visual_briefs WHERE card_id = ?", card_id) }
    pub fn asset_version(&self, asset_id: &str) -> Result<Option<i64>> { self.scalar_i64("SELECT version FROM flashcard_assets WHERE id = ?", asset_id) }
    pub fn audit_event_count(&self, card_id: &str) -> Result<i64> {
        let card_id = card_id.to_owned();
        self.runtime.block_on(async { Ok(sqlx::query_scalar("SELECT COUNT(*) FROM flashcard_audit_events WHERE card_id = ?").bind(card_id).fetch_one(&self.pool).await?) })
    }

    fn scalar_i64(&self, statement: &'static str, value: &str) -> Result<Option<i64>> {
        let value = value.to_owned();
        self.runtime.block_on(async { Ok(sqlx::query_scalar(statement).bind(value).fetch_one(&self.pool).await?) })
    }

    fn latest_review_passed(&self, card_id: &str) -> Result<bool> {
        let card_id = card_id.to_owned();
        self.runtime.block_on(async { Ok(sqlx::query_scalar::<_, i64>("SELECT passed FROM flashcard_reviews WHERE card_id = ? ORDER BY created_at DESC, id DESC LIMIT 1").bind(card_id).fetch_optional(&self.pool).await?.unwrap_or(0) == 1) })
    }

    async fn record_system_audit_async(&self, card_id: &str, action: &str, payload: Value) -> Result<()> { self.record_audit_async(card_id, AuditActorKind::System, None, action, payload).await }
    async fn record_audit_async(&self, card_id: &str, actor_kind: AuditActorKind, actor_id: Option<&str>, action: &str, payload: Value) -> Result<()> {
        sqlx::query("INSERT INTO flashcard_audit_events (card_id, actor_kind, actor_id, action, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(card_id).bind(actor_kind_to_db(&actor_kind)).bind(actor_id).bind(action)
            .bind(serde_json::to_string(&payload)?).bind(timestamp()).execute(&self.pool).await?;
        Ok(())
    }
}

fn timestamp() -> String { SystemTime::now().duration_since(UNIX_EPOCH).expect("clock after epoch").as_millis().to_string() }
fn status_to_db(status: FlashcardStatus) -> &'static str { match status { FlashcardStatus::Draft => "draft", FlashcardStatus::Generating => "generating", FlashcardStatus::NeedsReview => "needs-review", FlashcardStatus::RevisionRequested => "revision-requested", FlashcardStatus::Approved => "approved", FlashcardStatus::Published => "published" } }
fn status_from_db(status: &str) -> Result<FlashcardStatus> { match status { "draft" => Ok(FlashcardStatus::Draft), "generating" => Ok(FlashcardStatus::Generating), "needs-review" => Ok(FlashcardStatus::NeedsReview), "revision-requested" => Ok(FlashcardStatus::RevisionRequested), "approved" => Ok(FlashcardStatus::Approved), "published" => Ok(FlashcardStatus::Published), _ => Err(anyhow!("unknown flashcard status: {status}")) } }
fn asset_status_to_db(status: &FlashcardAssetStatus) -> &'static str { match status { FlashcardAssetStatus::Generating => "generating", FlashcardAssetStatus::NeedsReview => "needs-review", FlashcardAssetStatus::Rejected => "rejected", FlashcardAssetStatus::Approved => "approved" } }
fn job_kind_to_db(kind: &FlashcardJobKind) -> &'static str { match kind { FlashcardJobKind::Generate => "generate", FlashcardJobKind::Review => "review", FlashcardJobKind::Regenerate => "regenerate" } }
fn job_status_to_db(status: &FlashcardJobStatus) -> &'static str { match status { FlashcardJobStatus::Queued => "queued", FlashcardJobStatus::Running => "running", FlashcardJobStatus::Succeeded => "succeeded", FlashcardJobStatus::Failed => "failed" } }
fn reviewer_kind_to_db(kind: &ReviewerKind) -> &'static str { match kind { ReviewerKind::Automated => "automated", ReviewerKind::Teacher => "teacher" } }
fn actor_kind_to_db(kind: &AuditActorKind) -> &'static str { match kind { AuditActorKind::Teacher => "teacher", AuditActorKind::Chatgpt => "chatgpt", AuditActorKind::System => "system" } }
fn card_from_row(row: CardRow) -> Result<FlashcardCard> { Ok(FlashcardCard { id: row.id, source_exercise_id: row.source_exercise_id, category: row.category, teaching_copy_json: serde_json::from_str(&row.teaching_copy_json)?, style_profile: row.style_profile, character_id: row.character_id, current_asset_id: row.current_asset_id, version: row.version, status: status_from_db(&row.status)? }) }
