use std::collections::BTreeSet;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context};
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{async_trait, Json, Router};
use mps_db::{
    AuditActorKind, FlashcardAsset, FlashcardAssetStatus, FlashcardJob, FlashcardJobKind,
    FlashcardJobStatus, FlashcardListFilter, FlashcardPatch, FlashcardRepository,
    FlashcardReview, FlashcardWorkerCompletion, ReviewerKind,
};
use mps_flashcards::{
    CanonicalCatalog, CatalogExercise, FlashcardCard, FlashcardStatus, VisualBrief,
    LOCKED_STYLE_PROFILE,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const LOCKED_CHARACTER_ID: &str = "teacher-01";
pub const LOCKED_OUTFIT: &str = "off-white thin-strap cropped Pilates camisole and dark charcoal high-waisted mid-thigh biker shorts";
pub const DUSTY_ROSE_CHEEK_ACCENT: &str = "#D98F9A";
pub const DEFAULT_CATALOG_EXPORT_PATH: &str =
    "data/reference/mps_database_v1_core.json";

const CANONICAL_WORKBOOK_FILE: &str = "MPS_Database_v1_Core.xlsx";
const CANONICAL_WORKBOOK_PATH: &str = "data/reference/MPS_Database_v1_Core.xlsx";

pub const READ_CATALOG: &str = "read_catalog";
pub const WRITE_DRAFT: &str = "write_draft";
pub const GENERATE_ASSET: &str = "generate_asset";
pub const SUBMIT_REVIEW: &str = "submit_review";
pub const AUTOMATED_REVIEW: &str = "automated_review";
pub const VISUAL_WORKER: &str = "visual_worker";
pub const PUBLISH: &str = "publish";

const WORKER_QA_CHECKS: &[&str] = &[
    "pose",
    "anatomy",
    "identity",
    "glasses_hair",
    "outfit",
    "apparatus",
    "linework",
    "cheek_accent",
    "forbidden_overlays",
];

static ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub studio_id: String,
    pub teacher_id: String,
    pub permissions: BTreeSet<String>,
    pub actor_kind: AuditActorKind,
}

impl AuthContext {
    pub fn new<I, P>(
        studio_id: impl Into<String>,
        teacher_id: impl Into<String>,
        permissions: I,
    ) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        Self {
            studio_id: studio_id.into(),
            teacher_id: teacher_id.into(),
            permissions: permissions.into_iter().map(Into::into).collect(),
            actor_kind: AuditActorKind::Teacher,
        }
    }

    pub fn with_actor_kind(mut self, actor_kind: AuditActorKind) -> Self {
        self.actor_kind = actor_kind;
        self
    }

    fn require(&self, permission: &str) -> Result<(), ApiError> {
        if self.permissions.contains(permission) {
            Ok(())
        } else {
            Err(ApiError::forbidden(format!(
                "missing required permission: {permission}"
            )))
        }
    }

    fn require_any(&self, permissions: &[&str]) -> Result<(), ApiError> {
        if permissions
            .iter()
            .any(|permission| self.permissions.contains(*permission))
        {
            Ok(())
        } else {
            Err(ApiError::forbidden(format!(
                "missing one of required permissions: {}",
                permissions.join(", ")
            )))
        }
    }

    fn validate(&self) -> Result<(), ApiError> {
        if self.studio_id.trim().is_empty()
            || self.teacher_id.trim().is_empty()
            || self.permissions.is_empty()
        {
            Err(ApiError::unauthorized(
                "authenticated teacher context is incomplete",
            ))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or_else(|| {
                ApiError::unauthorized("authenticated teacher context is required")
            })?;
        auth.validate()?;
        Ok(auth)
    }
}

#[derive(Clone)]
pub struct AppState {
    repository: Arc<FlashcardRepository>,
    catalog: Arc<CanonicalCatalog>,
    visual_contract: Arc<Value>,
    character_reference: Arc<Value>,
    publication_references_approved: bool,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub database_path: String,
    pub catalog_path: String,
    pub visual_styles_path: String,
    pub visual_manifest_path: String,
}

impl AppState {
    pub fn load(config: &ServerConfig) -> anyhow::Result<Self> {
        let catalog_json = fs::read_to_string(&config.catalog_path)
            .with_context(|| format!("read canonical catalog {}", config.catalog_path))?;
        validate_catalog_provenance(&catalog_json)?;
        let visual_styles_json = fs::read_to_string(&config.visual_styles_path)
            .with_context(|| format!("read visual styles {}", config.visual_styles_path))?;
        let visual_manifest_json = fs::read_to_string(&config.visual_manifest_path)
            .with_context(|| format!("read visual manifest {}", config.visual_manifest_path))?;
        let catalog = CanonicalCatalog::load_json(&catalog_json)?;
        let repository = FlashcardRepository::open(&config.database_path, catalog.clone())?;
        Self::from_documents(
            repository,
            catalog,
            &visual_styles_json,
            &visual_manifest_json,
        )
    }

    pub fn from_documents(
        repository: FlashcardRepository,
        catalog: CanonicalCatalog,
        visual_styles_json: &str,
        visual_manifest_json: &str,
    ) -> anyhow::Result<Self> {
        let (visual_contract, style_approved) =
            load_public_visual_contract(visual_styles_json)?;
        let (character_reference, character_approved) =
            load_public_character_reference(visual_manifest_json)?;
        Ok(Self {
            repository: Arc::new(repository),
            catalog: Arc::new(catalog),
            visual_contract: Arc::new(visual_contract),
            character_reference: Arc::new(character_reference),
            publication_references_approved: style_approved && character_approved,
        })
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/flashcards", get(list_flashcards).post(create_flashcard))
        .route(
            "/api/catalog/exercises/:exercise_id/context",
            get(get_exercise_context),
        )
        .route("/api/visual-contract", get(get_visual_contract))
        .route("/api/character-reference", get(get_character_reference))
        .route(
            "/api/flashcards/:id",
            get(get_flashcard).patch(update_flashcard),
        )
        .route(
            "/api/flashcards/:id/visual-briefs",
            post(create_visual_brief),
        )
        .route("/api/flashcards/:id/jobs", post(create_job))
        .route(
            "/api/flashcards/:id/jobs/:job_id",
            get(get_flashcard_job),
        )
        .route(
            "/api/internal/flashcards/:id/jobs/:job_id/claim",
            post(claim_visual_job),
        )
        .route(
            "/api/internal/flashcards/:id/jobs/:job_id/complete",
            post(complete_visual_job),
        )
        .route("/api/flashcards/:id/reviews", post(create_review))
        .route(
            "/api/flashcards/:id/submit-review",
            post(submit_review),
        )
        .route("/api/flashcards/:id/approve", post(approve_flashcard))
        .route("/api/flashcards/:id/publish", post(publish_flashcard))
        .with_state(state)
}

#[derive(Debug, Deserialize, Default)]
struct ListFlashcardsQuery {
    query: Option<String>,
    category: Option<String>,
    level: Option<String>,
    status: Option<FlashcardStatus>,
}

#[derive(Debug, Serialize)]
struct FlashcardResponse {
    id: String,
    source_exercise_id: String,
    name: String,
    apparatus: String,
    level: Option<String>,
    category: String,
    teaching_copy_json: Value,
    style_profile: String,
    character_id: String,
    current_asset_id: Option<String>,
    version: i64,
    status: FlashcardStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateFlashcardRequest {
    id: Option<String>,
    source_exercise_id: String,
    category: Option<String>,
    teaching_copy_json: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateFlashcardRequest {
    category: Option<String>,
    teaching_copy_json: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ExerciseContextResponse {
    exercise: ExerciseContextCard,
    apparatus: ApparatusContext,
    families: Vec<FamilyContext>,
    categories: Vec<NamedContext>,
    body_regions: Vec<NamedContext>,
    movement_taxonomy: Vec<NamedContext>,
    progressions: Vec<NamedContext>,
}

#[derive(Debug, Serialize)]
struct ExerciseContextCard {
    id: String,
    name: String,
    apparatus: String,
    equipment_key: String,
    level: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApparatusContext {
    name: String,
    equipment_key: String,
}

#[derive(Debug, Serialize)]
struct FamilyContext {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct NamedContext {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateVisualBriefRequest {
    id: Option<String>,
    exercise_id: Option<String>,
    style_profile: Option<String>,
    character_id: Option<String>,
    outfit: Option<String>,
    pose_json: Value,
    apparatus: Option<String>,
    palette_json: Option<Value>,
    must_show_json: Option<Value>,
    must_not_show_json: Option<Value>,
}

#[derive(Debug, Serialize)]
struct VisualBriefResponse {
    id: String,
    card_id: String,
    exercise_id: String,
    style_profile: String,
    character_id: String,
    outfit: String,
    pose_json: Value,
    apparatus: String,
    palette_json: Value,
    must_show_json: Value,
    must_not_show_json: Value,
    version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CreateJobKind {
    Generate,
    Review,
    Regenerate,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateJobRequest {
    id: Option<String>,
    kind: CreateJobKind,
    brief_id: Option<String>,
    revision_notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct JobResponse {
    id: String,
    card_id: String,
    kind: &'static str,
    status: &'static str,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateReviewRequest {
    id: Option<String>,
    asset_id: String,
    passed: bool,
    findings_json: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum WorkerCompletionStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerAssetRequest {
    id: String,
    brief_id: String,
    repo_path: String,
    provider_job_id: Option<String>,
    version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerReviewRequest {
    id: String,
    asset_id: String,
    passed: bool,
    findings_json: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerCompletionRequest {
    status: WorkerCompletionStatus,
    asset: Option<WorkerAssetRequest>,
    review: Option<WorkerReviewRequest>,
    error_code: Option<String>,
}

impl WorkerCompletionRequest {
    fn into_completion(self, card_id: &str) -> Result<FlashcardWorkerCompletion, ApiError> {
        let completion = match self.status {
            WorkerCompletionStatus::Succeeded => {
                let asset = self
                    .asset
                    .ok_or_else(|| ApiError::bad_request("successful worker completion requires an asset"))?;
                let review = self
                    .review
                    .ok_or_else(|| ApiError::bad_request("successful worker completion requires a review"))?;
                if review.asset_id != asset.id {
                    return Err(ApiError::bad_request("worker review payload is invalid"));
                }
                let findings_json = sanitize_worker_findings(&review.findings_json)?;
                FlashcardWorkerCompletion {
                    status: FlashcardJobStatus::Succeeded,
                    asset: Some(FlashcardAsset {
                        id: asset.id,
                        card_id: card_id.to_owned(),
                        brief_id: asset.brief_id,
                        repo_path: asset.repo_path,
                        provider_job_id: asset.provider_job_id,
                        version: asset.version,
                        status: if review.passed {
                            FlashcardAssetStatus::NeedsReview
                        } else {
                            FlashcardAssetStatus::Rejected
                        },
                    }),
                    review: Some(FlashcardReview {
                        id: review.id,
                        card_id: card_id.to_owned(),
                        asset_id: review.asset_id,
                        passed: review.passed,
                        findings_json,
                        reviewer_kind: ReviewerKind::Automated,
                        reviewer_id: None,
                    }),
                    error_code: None,
                }
            }
            WorkerCompletionStatus::Failed => {
                if self.asset.is_some() || self.review.is_some() {
                    return Err(ApiError::bad_request("failed worker completion cannot include an asset or review"));
                }
                FlashcardWorkerCompletion {
                    status: FlashcardJobStatus::Failed,
                    asset: None,
                    review: None,
                    error_code: Some(sanitize_worker_error_code(self.error_code.as_deref())),
                }
            }
        };
        Ok(completion)
    }
}

fn sanitize_worker_error_code(value: Option<&str>) -> String {
    match value {
        Some("visual_contract_invalid") => "visual_contract_invalid".to_owned(),
        Some("vision_review_failed") => "vision_review_failed".to_owned(),
        Some("image_generation_failed") => "image_generation_failed".to_owned(),
        _ => "visual_worker_failed".to_owned(),
    }
}

fn sanitize_worker_findings(value: &Value) -> Result<Value, ApiError> {
    let findings = value
        .as_array()
        .ok_or_else(|| ApiError::bad_request("worker review payload is invalid"))?;
    if findings.len() > 128 {
        return Err(ApiError::bad_request("worker review payload is invalid"));
    }
    let sanitized = findings
        .iter()
        .map(|finding| {
            let code = match finding.get("code").and_then(Value::as_str) {
                Some(code)
                    if matches!(
                        code,
                        "unsafe_asset_path"
                            | "unsupported_image_type"
                            | "invalid_asset_metadata"
                            | "worker_cannot_approve"
                            | "vision_reviewer_unavailable"
                            | "visual_check_failed"
                            | "vision_review_failed"
                            | "vision_finding"
                    ) => code,
                _ => "visual_finding",
            };
            let severity = if finding.get("severity").and_then(Value::as_str) == Some("warning") {
                "warning"
            } else {
                "error"
            };
            let check = if code == "visual_check_failed" {
                finding
                    .get("check")
                    .and_then(Value::as_str)
                    .filter(|check| WORKER_QA_CHECKS.contains(check))
            } else {
                None
            };
            match check {
                Some(check) => json!({"code": code, "severity": severity, "check": check}),
                None => json!({"code": code, "severity": severity}),
            }
        })
        .collect::<Vec<_>>();
    let sanitized = Value::Array(sanitized);
    if serde_json::to_vec(&sanitized)
        .map(|bytes| bytes.len() > 64 * 1024)
        .unwrap_or(true)
    {
        return Err(ApiError::bad_request("worker review payload is invalid"));
    }
    Ok(sanitized)
}

#[derive(Debug, Serialize)]
struct MutationResponse {
    card: FlashcardResponse,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ApproveFlashcardRequest {
    review_id: Option<String>,
    findings_json: Option<Value>,
}

async fn list_flashcards(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<ListFlashcardsQuery>,
) -> Result<Json<Vec<FlashcardResponse>>, ApiError> {
    auth.require(READ_CATALOG)?;
    let studio_id = auth.studio_id;
    let filter = FlashcardListFilter {
        status: query.status,
        category: query.category,
    };
    let cards = run_repository(state.repository.clone(), move |repository| {
        repository.list_flashcards_for_studio(&studio_id, filter)
    })
    .await?;
    let query_text = query.query.unwrap_or_default().trim().to_lowercase();
    let level = query.level.unwrap_or_default().trim().to_lowercase();
    let responses = cards
        .into_iter()
        .filter_map(|card| {
            let exercise = state.catalog.find(&card.source_exercise_id)?;
            let response = flashcard_response(card, exercise);
            let searchable = format!(
                "{} {} {} {} {} {} {}",
                response.id,
                response.source_exercise_id,
                response.name,
                response.apparatus,
                response.level.as_deref().unwrap_or_default(),
                response.category,
                response.teaching_copy_json
            )
            .to_lowercase();
            let matches_query = query_text.is_empty() || searchable.contains(&query_text);
            let matches_level = level.is_empty()
                || level == "all"
                || response
                    .level
                    .as_deref()
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(&level));
            (matches_query && matches_level).then_some(response)
        })
        .collect();
    Ok(Json(responses))
}

async fn get_exercise_context(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(exercise_id): Path<String>,
) -> Result<Json<ExerciseContextResponse>, ApiError> {
    auth.require(READ_CATALOG)?;
    let exercise = state
        .catalog
        .find(&exercise_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("source exercise not found"))?;
    let families = exercise
        .family_names
        .iter()
        .enumerate()
        .map(|(index, name)| FamilyContext {
            id: exercise.family_ids.get(index).cloned().unwrap_or_default(),
            name: name.clone(),
        })
        .collect();
    let exercise_context = ExerciseContextCard {
        id: exercise.id.clone(),
        name: exercise.exercise.clone(),
        apparatus: exercise.apparatus.clone(),
        equipment_key: exercise.equipment_key.clone(),
        level: exercise_level(&exercise),
    };
    Ok(Json(ExerciseContextResponse {
        apparatus: ApparatusContext {
            name: exercise.apparatus.clone(),
            equipment_key: exercise.equipment_key.clone(),
        },
        families,
        categories: named_contexts(&exercise.categories),
        body_regions: named_contexts(&exercise.body_regions),
        movement_taxonomy: named_contexts(&exercise.movement_goals),
        progressions: named_contexts(&exercise.progression_stages),
        exercise: exercise_context,
    }))
}

async fn get_visual_contract(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Value>, ApiError> {
    auth.require(READ_CATALOG)?;
    Ok(Json((*state.visual_contract).clone()))
}

async fn get_character_reference(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Value>, ApiError> {
    auth.require(READ_CATALOG)?;
    Ok(Json((*state.character_reference).clone()))
}

async fn get_flashcard(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<FlashcardResponse>, ApiError> {
    auth.require(READ_CATALOG)?;
    let studio_id = auth.studio_id;
    let requested_id = id.clone();
    let card = run_repository(state.repository.clone(), move |repository| {
        repository.get_flashcard_for_studio(&studio_id, &requested_id)
    })
    .await?
    .ok_or_else(|| ApiError::not_found("flashcard not found"))?;
    let exercise = state
        .catalog
        .find(&card.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok(Json(flashcard_response(card, exercise)))
}

async fn create_flashcard(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(request): Json<CreateFlashcardRequest>,
) -> Result<(StatusCode, Json<FlashcardResponse>), ApiError> {
    auth.require(WRITE_DRAFT)?;
    let exercise = state
        .catalog
        .find(&request.source_exercise_id)
        .cloned()
        .ok_or_else(|| ApiError::bad_request("unknown source exercise ID"))?;
    let card = FlashcardCard {
        id: request.id.unwrap_or_else(|| next_id("card")),
        source_exercise_id: exercise.id.clone(),
        category: request
            .category
            .or_else(|| exercise.categories.first().cloned())
            .unwrap_or_else(|| "Uncategorized".to_owned()),
        teaching_copy_json: request.teaching_copy_json.unwrap_or_else(|| json!({})),
        style_profile: LOCKED_STYLE_PROFILE.to_owned(),
        character_id: LOCKED_CHARACTER_ID.to_owned(),
        current_asset_id: None,
        version: 1,
        status: FlashcardStatus::Draft,
    };
    let studio_id = auth.studio_id;
    let teacher_id = auth.teacher_id;
    let actor_kind = auth.actor_kind;
    let created = run_repository(state.repository.clone(), move |repository| {
        repository.create_flashcard_for_actor(&studio_id, &teacher_id, actor_kind, &card)
    })
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(flashcard_response(created, &exercise)),
    ))
}

async fn update_flashcard(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(request): Json<UpdateFlashcardRequest>,
) -> Result<Json<FlashcardResponse>, ApiError> {
    auth.require(WRITE_DRAFT)?;
    if request.category.is_none() && request.teaching_copy_json.is_none() {
        return Err(ApiError::bad_request(
            "PATCH requires category or teaching_copy_json",
        ));
    }
    let patch = FlashcardPatch {
        category: request.category,
        teaching_copy_json: request.teaching_copy_json,
    };
    let studio_id = auth.studio_id;
    let teacher_id = auth.teacher_id;
    let actor_kind = auth.actor_kind;
    let updated = run_repository(state.repository.clone(), move |repository| {
        repository.update_flashcard_for_actor(&studio_id, &teacher_id, actor_kind, &id, patch)
    })
    .await?;
    let exercise = state
        .catalog
        .find(&updated.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok(Json(flashcard_response(updated, exercise)))
}

async fn create_visual_brief(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(request): Json<CreateVisualBriefRequest>,
) -> Result<(StatusCode, Json<VisualBriefResponse>), ApiError> {
    auth.require(WRITE_DRAFT)?;
    let card = get_scoped_card(&state, &auth, &id).await?;
    let exercise = state
        .catalog
        .find(&card.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    validate_visual_brief_request(&card, exercise, &request)?;
    let card_id = card.id.clone();
    let studio_id = auth.studio_id.clone();
    let latest_version = run_repository(state.repository.clone(), move |repository| {
        repository.latest_brief_version_for_studio(&studio_id, &card_id)
    })
    .await?;
    let brief = VisualBrief {
        id: request.id.unwrap_or_else(|| next_id("brief")),
        card_id: card.id,
        exercise_id: card.source_exercise_id,
        style_profile: LOCKED_STYLE_PROFILE.to_owned(),
        character_id: LOCKED_CHARACTER_ID.to_owned(),
        outfit: LOCKED_OUTFIT.to_owned(),
        pose_json: request.pose_json,
        apparatus: request
            .apparatus
            .unwrap_or_else(|| exercise.apparatus.clone()),
        palette_json: locked_palette(&state.visual_contract),
        must_show_json: request.must_show_json.unwrap_or_else(|| json!([])),
        must_not_show_json: request.must_not_show_json.unwrap_or_else(|| json!([])),
        version: latest_version.unwrap_or(0) + 1,
    };
    let studio_id = auth.studio_id;
    let teacher_id = auth.teacher_id;
    let actor_kind = auth.actor_kind;
    let saved_brief = brief.clone();
    run_repository(state.repository.clone(), move |repository| {
        repository.save_visual_brief_for_actor(&studio_id, &teacher_id, actor_kind, &saved_brief)
    })
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(visual_brief_response(brief)),
    ))
}

async fn create_job(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(request): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let card = get_scoped_card(&state, &auth, &id).await?;
    let kind = match request.kind {
        CreateJobKind::Generate => FlashcardJobKind::Generate,
        CreateJobKind::Review => FlashcardJobKind::Review,
        CreateJobKind::Regenerate => FlashcardJobKind::Regenerate,
    };
    match &kind {
        FlashcardJobKind::Review => auth.require(SUBMIT_REVIEW)?,
        FlashcardJobKind::Generate | FlashcardJobKind::Regenerate => {
            auth.require(GENERATE_ASSET)?
        }
    }
    if matches!(
        &kind,
        FlashcardJobKind::Generate | FlashcardJobKind::Regenerate
    ) && request.brief_id.is_none()
    {
        return Err(ApiError::bad_request(
            "generation jobs require a visual brief ID",
        ));
    }
    let job = FlashcardJob {
        id: request.id.unwrap_or_else(|| next_id("job")),
        card_id: card.id,
        kind,
        status: FlashcardJobStatus::Queued,
        input_json: json!({
            "brief_id": request.brief_id,
            "revision_notes": request.revision_notes,
            "card_version": card.version,
        }),
        output_json: None,
        error: None,
    };
    let studio_id = auth.studio_id;
    let teacher_id = auth.teacher_id;
    let actor_kind = auth.actor_kind;
    let saved_job = job.clone();
    run_repository(state.repository.clone(), move |repository| {
        repository.create_job_for_actor(&studio_id, &teacher_id, actor_kind, &saved_job)
    })
    .await?;
    Ok((StatusCode::ACCEPTED, Json(job_response(job))))
}

async fn get_flashcard_job(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<JobResponse>, ApiError> {
    auth.require_any(&[GENERATE_ASSET, SUBMIT_REVIEW])?;
    let studio_id = auth.studio_id;
    let job = run_repository(state.repository.clone(), move |repository| {
        repository.get_job_for_studio(&studio_id, &id, &job_id)
    })
    .await?
    .ok_or_else(|| ApiError::not_found("flashcard job not found"))?;
    Ok(Json(job_response(job)))
}

async fn claim_visual_job(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((card_id, job_id)): Path<(String, String)>,
) -> Result<Json<JobResponse>, ApiError> {
    auth.require(VISUAL_WORKER)?;
    if auth.actor_kind != AuditActorKind::System {
        return Err(ApiError::forbidden("visual worker service identity is required"));
    }
    let studio_id = auth.studio_id;
    let worker_id = auth.teacher_id;
    let claimed = run_repository(state.repository.clone(), move |repository| {
        repository.claim_job_for_worker(&studio_id, &worker_id, &card_id, &job_id)
    })
    .await?;
    Ok(Json(job_response(claimed)))
}

async fn complete_visual_job(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((card_id, job_id)): Path<(String, String)>,
    Json(request): Json<WorkerCompletionRequest>,
) -> Result<Json<JobResponse>, ApiError> {
    auth.require(VISUAL_WORKER)?;
    if auth.actor_kind != AuditActorKind::System {
        return Err(ApiError::forbidden("visual worker service identity is required"));
    }
    let completion = request.into_completion(&card_id)?;
    let studio_id = auth.studio_id;
    let worker_id = auth.teacher_id;
    let completed = run_repository(state.repository.clone(), move |repository| {
        repository.complete_job_for_worker(
            &studio_id,
            &worker_id,
            &card_id,
            &job_id,
            &completion,
        )
    })
    .await?;
    Ok(Json(job_response(completed)))
}

async fn create_review(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(request): Json<CreateReviewRequest>,
) -> Result<(StatusCode, Json<MutationResponse>), ApiError> {
    auth.require(AUTOMATED_REVIEW)?;
    let card = get_scoped_card(&state, &auth, &id).await?;
    let review = FlashcardReview {
        id: request.id.unwrap_or_else(|| next_id("review")),
        card_id: card.id.clone(),
        asset_id: request.asset_id,
        passed: request.passed,
        findings_json: request.findings_json.unwrap_or_else(|| json!([])),
        reviewer_kind: ReviewerKind::Automated,
        reviewer_id: None,
    };
    let studio_id = auth.studio_id.clone();
    let teacher_id = auth.teacher_id.clone();
    run_repository(state.repository.clone(), move |repository| {
        repository.record_automated_review_for_studio(&studio_id, &teacher_id, &review)
    })
    .await?;
    let card = get_scoped_card(&state, &auth, &id).await?;
    let exercise = state
        .catalog
        .find(&card.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok((
        StatusCode::CREATED,
        Json(MutationResponse {
            card: flashcard_response(card, exercise),
        }),
    ))
}

async fn submit_review(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<MutationResponse>, ApiError> {
    auth.require(SUBMIT_REVIEW)?;
    let studio_id = auth.studio_id.clone();
    let teacher_id = auth.teacher_id.clone();
    let actor_kind = auth.actor_kind.clone();
    let card_id = id.clone();
    run_repository(state.repository.clone(), move |repository| {
        repository.transition_status_for_actor(
            &studio_id,
            &teacher_id,
            actor_kind,
            &card_id,
            FlashcardStatus::NeedsReview,
        )
    })
    .await?;
    mutation_response(&state, &auth, &id).await
}

async fn approve_flashcard(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(request): Json<ApproveFlashcardRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    auth.require(PUBLISH)?;
    require_publication_references(&state)?;
    let studio_id = auth.studio_id.clone();
    let teacher_id = auth.teacher_id.clone();
    let card_id = id.clone();
    let review_id = request.review_id.unwrap_or_else(|| next_id("review"));
    let findings_json = request.findings_json.unwrap_or_else(|| json!([]));
    run_repository(state.repository.clone(), move |repository| {
        repository.approve_flashcard_for_studio(
            &studio_id,
            &teacher_id,
            &card_id,
            &review_id,
            &findings_json,
        )
    })
    .await?;
    mutation_response(&state, &auth, &id).await
}

async fn publish_flashcard(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<MutationResponse>, ApiError> {
    auth.require(PUBLISH)?;
    require_publication_references(&state)?;
    let studio_id = auth.studio_id.clone();
    let teacher_id = auth.teacher_id.clone();
    let card_id = id.clone();
    run_repository(state.repository.clone(), move |repository| {
        repository.publish_flashcard_for_studio(&studio_id, &teacher_id, &card_id)
    })
    .await?;
    mutation_response(&state, &auth, &id).await
}

async fn get_scoped_card(
    state: &AppState,
    auth: &AuthContext,
    id: &str,
) -> Result<FlashcardCard, ApiError> {
    let studio_id = auth.studio_id.clone();
    let id = id.to_owned();
    run_repository(state.repository.clone(), move |repository| {
        repository.get_flashcard_for_studio(&studio_id, &id)
    })
    .await?
    .ok_or_else(|| ApiError::not_found("flashcard not found"))
}

async fn mutation_response(
    state: &AppState,
    auth: &AuthContext,
    id: &str,
) -> Result<Json<MutationResponse>, ApiError> {
    let card = get_scoped_card(state, auth, id).await?;
    let exercise = state
        .catalog
        .find(&card.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok(Json(MutationResponse {
        card: flashcard_response(card, exercise),
    }))
}

async fn run_repository<T, F>(
    repository: Arc<FlashcardRepository>,
    operation: F,
) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce(&FlashcardRepository) -> anyhow::Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(move || operation(&repository))
        .await
        .map_err(|_| ApiError::internal())?
        .map_err(repository_error)
}

fn validate_visual_brief_request(
    card: &FlashcardCard,
    exercise: &CatalogExercise,
    request: &CreateVisualBriefRequest,
) -> Result<(), ApiError> {
    if request
        .exercise_id
        .as_deref()
        .is_some_and(|value| value != card.source_exercise_id)
        || request
            .style_profile
            .as_deref()
            .is_some_and(|value| value != LOCKED_STYLE_PROFILE)
        || request
            .character_id
            .as_deref()
            .is_some_and(|value| value != LOCKED_CHARACTER_ID)
        || request
            .outfit
            .as_deref()
            .is_some_and(|value| value != LOCKED_OUTFIT)
        || request
            .apparatus
            .as_deref()
            .is_some_and(|value| value != exercise.apparatus)
        || request
            .palette_json
            .as_ref()
            .and_then(|palette| palette.get("cheekAccent"))
            .and_then(Value::as_str)
            .is_some_and(|value| value != DUSTY_ROSE_CHEEK_ACCENT)
    {
        return Err(ApiError::conflict(
            "visual brief source, apparatus, and locked visual contract cannot be changed",
        ));
    }
    Ok(())
}

fn require_publication_references(state: &AppState) -> Result<(), ApiError> {
    if state.publication_references_approved {
        Ok(())
    } else {
        Err(ApiError::conflict(
            "publication references are not approved",
        ))
    }
}

fn flashcard_response(card: FlashcardCard, exercise: &CatalogExercise) -> FlashcardResponse {
    FlashcardResponse {
        id: card.id,
        source_exercise_id: card.source_exercise_id,
        name: exercise.exercise.clone(),
        apparatus: exercise.apparatus.clone(),
        level: exercise_level(exercise),
        category: card.category,
        teaching_copy_json: card.teaching_copy_json,
        style_profile: card.style_profile,
        character_id: card.character_id,
        current_asset_id: card.current_asset_id,
        version: card.version,
        status: card.status,
    }
}

fn exercise_level(exercise: &CatalogExercise) -> Option<String> {
    exercise
        .mps
        .get("difficulty")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn visual_brief_response(brief: VisualBrief) -> VisualBriefResponse {
    VisualBriefResponse {
        id: brief.id,
        card_id: brief.card_id,
        exercise_id: brief.exercise_id,
        style_profile: brief.style_profile,
        character_id: brief.character_id,
        outfit: brief.outfit,
        pose_json: brief.pose_json,
        apparatus: brief.apparatus,
        palette_json: brief.palette_json,
        must_show_json: brief.must_show_json,
        must_not_show_json: brief.must_not_show_json,
        version: brief.version,
    }
}

fn job_response(job: FlashcardJob) -> JobResponse {
    JobResponse {
        id: job.id,
        card_id: job.card_id,
        kind: match job.kind {
            FlashcardJobKind::Generate => "generate",
            FlashcardJobKind::Review => "review",
            FlashcardJobKind::Regenerate => "regenerate",
        },
        status: match job.status {
            FlashcardJobStatus::Queued => "queued",
            FlashcardJobStatus::Running => "running",
            FlashcardJobStatus::Succeeded => "succeeded",
            FlashcardJobStatus::Failed => "failed",
        },
        error: job.error,
    }
}

fn named_contexts(values: &[String]) -> Vec<NamedContext> {
    values
        .iter()
        .cloned()
        .map(|name| NamedContext { name })
        .collect()
}

fn locked_palette(visual_contract: &Value) -> Value {
    visual_contract
        .get("palette")
        .cloned()
        .unwrap_or_else(|| json!({"cheekAccent": DUSTY_ROSE_CHEEK_ACCENT}))
}

fn validate_catalog_provenance(json_source: &str) -> anyhow::Result<()> {
    let document: Value =
        serde_json::from_str(json_source).context("parse canonical catalog JSON")?;
    let source_file = document
        .pointer("/source/file")
        .and_then(Value::as_str);
    let source_path = document
        .pointer("/source/path")
        .and_then(Value::as_str);
    if source_file != Some(CANONICAL_WORKBOOK_FILE) {
        return Err(anyhow!(
            "catalog JSON must declare source.file as {CANONICAL_WORKBOOK_FILE}"
        ));
    }
    if source_path.is_some_and(|path| path != CANONICAL_WORKBOOK_PATH) {
        return Err(anyhow!(
            "catalog JSON source.path must be {CANONICAL_WORKBOOK_PATH} when present"
        ));
    }
    Ok(())
}

fn load_public_visual_contract(json_source: &str) -> anyhow::Result<(Value, bool)> {
    let document: Value =
        serde_json::from_str(json_source).context("parse visual styles JSON")?;
    if document.get("primaryProfile").and_then(Value::as_str) != Some(LOCKED_STYLE_PROFILE) {
        return Err(anyhow!("unsupported primary visual style profile"));
    }
    let profile = document
        .get("profiles")
        .and_then(Value::as_array)
        .and_then(|profiles| {
            profiles
                .iter()
                .find(|profile| profile.get("id").and_then(Value::as_str) == Some(LOCKED_STYLE_PROFILE))
        })
        .ok_or_else(|| anyhow!("locked visual style profile is missing"))?;
    if profile.get("characterReference").and_then(Value::as_str) != Some(LOCKED_CHARACTER_ID)
        || profile
            .pointer("/palette/cheekAccent")
            .and_then(Value::as_str)
            != Some(DUSTY_ROSE_CHEEK_ACCENT)
    {
        return Err(anyhow!("visual contract does not match locked MPS references"));
    }
    let status = profile
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("needs-review");
    let public = sanitize_public_value(json!({
        "schema_version": document.get("schemaVersion").cloned().unwrap_or(Value::Null),
        "style_profile": LOCKED_STYLE_PROFILE,
        "status": status,
        "character_id": LOCKED_CHARACTER_ID,
        "outfit": LOCKED_OUTFIT,
        "visual_language": profile.get("visualLanguage").cloned().unwrap_or_else(|| json!({})),
        "composition": profile.get("composition").cloned().unwrap_or_else(|| json!({})),
        "palette": profile.get("palette").cloned().unwrap_or_else(|| json!({})),
        "locked_character_rules": profile.get("lockedCharacterRules").cloned().unwrap_or_else(|| json!([])),
        "negative_prompt": profile.get("negativePrompt").cloned().unwrap_or_else(|| json!([])),
        "review_checklist": profile.get("reviewChecklist").cloned().unwrap_or_else(|| json!([])),
    }));
    Ok((public, status == "approved"))
}

fn load_public_character_reference(json_source: &str) -> anyhow::Result<(Value, bool)> {
    let document: Value =
        serde_json::from_str(json_source).context("parse visual manifest JSON")?;
    if document.get("primaryStyleProfile").and_then(Value::as_str) != Some(LOCKED_STYLE_PROFILE) {
        return Err(anyhow!("visual manifest style profile does not match"));
    }
    let character = document
        .get("character")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("visual manifest character is missing"))?;
    if character.get("id").and_then(Value::as_str) != Some(LOCKED_CHARACTER_ID) {
        return Err(anyhow!("unsupported character reference"));
    }
    let fixed_attributes = character
        .get("fixedAttributes")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let top = fixed_attributes.get("top").and_then(Value::as_str);
    let bottom = fixed_attributes.get("bottom").and_then(Value::as_str);
    if top != Some("fitted off-white cropped Pilates camisole with thin straps")
        || bottom
            != Some("dark charcoal high-waisted fitted Pilates biker shorts ending mid-thigh")
    {
        return Err(anyhow!("visual manifest outfit does not match the locked outfit"));
    }
    let reference_roles = character
        .get("references")
        .and_then(Value::as_array)
        .map(|references| {
            references
                .iter()
                .map(|reference| {
                    json!({
                        "id": reference.get("id").cloned().unwrap_or(Value::Null),
                        "role": reference.get("role").cloned().unwrap_or(Value::Null),
                        "required": reference.get("required").cloned().unwrap_or(Value::Bool(false)),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let status = character
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("needs-review");
    let public = sanitize_public_value(json!({
        "id": LOCKED_CHARACTER_ID,
        "status": status,
        "version": character.get("version").cloned().unwrap_or(Value::Null),
        "outfit": LOCKED_OUTFIT,
        "fixed_attributes": fixed_attributes,
        "reference_roles": reference_roles,
    }));
    Ok((public, status == "approved"))
}

fn sanitize_public_value(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter(|(key, _)| !is_private_reference_key(key))
                .map(|(key, value)| (key, sanitize_public_value(value)))
                .filter(|(_, value)| !value.is_null())
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(sanitize_public_value)
                .filter(|value| !value.is_null())
                .collect(),
        ),
        Value::String(value) if looks_like_private_reference(&value) => Value::Null,
        other => other,
    }
}

fn is_private_reference_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "canonicalsheet"
            | "canonical_sheet"
            | "image"
            | "path"
            | "previewasset"
            | "preview_asset"
            | "repo_path"
            | "source"
            | "sourcepath"
            | "source_path"
            | "sourceuri"
            | "source_uri"
            | "stylereference"
            | "style_reference"
            | "uri"
            | "url"
    )
}

fn looks_like_private_reference(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("private://")
        || lower.starts_with("file://")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data/reference/")
        || lower.starts_with("docs/")
        || lower.starts_with("private/")
        || lower.starts_with("web/")
        || lower.starts_with("./")
        || lower.starts_with("../")
        || lower.starts_with("~/")
        || trimmed.starts_with('/')
        || (trimmed.len() > 2
            && trimmed.as_bytes()[1] == b':'
            && matches!(trimmed.as_bytes()[2], b'\\' | b'/'))
}

fn next_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after Unix epoch")
        .as_millis();
    let sequence = ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{millis}-{sequence}")
}

fn repository_error(error: anyhow::Error) -> ApiError {
    let message = error.to_string();
    if message.contains("not found") {
        ApiError::not_found("resource not found")
    } else if message.contains("invalid flashcard status transition")
        || message.contains("cannot publish")
        || message.contains("cannot approve")
        || message.contains("cannot continue")
        || message.contains("worker callback")
        || message.contains("worker claim")
        || message.contains("UNIQUE constraint")
    {
        ApiError::conflict(message)
    } else if message.contains("source exercise")
        || message.contains("visual brief")
        || message.contains("patch contains no editable fields")
    {
        ApiError::bad_request(message)
    } else {
        ApiError::internal()
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal server error".to_owned(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "error": {
                    "status": self.status.as_u16(),
                    "message": self.message,
                }
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod routes {
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request};
    use mps_db::{FlashcardAsset, FlashcardAssetStatus};
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    const CATALOG_JSON: &str = r#"{
      "source": {
        "file": "MPS_Database_v1_Core.xlsx",
        "path": "data/reference/MPS_Database_v1_Core.xlsx"
      },
      "tables": {
        "exerciseMaster": [{
          "sourceRow": 4,
          "id": "source_chair_achilles_stretch_row_4",
          "exercise": "Achilles Stretch",
          "apparatus": "Chair",
          "equipmentKey": "chair",
          "familyIds": ["9"],
          "familyNames": ["Footwork"],
          "categories": ["Lower Body Work"],
          "bodyRegions": ["Ankle"],
          "movementGoals": ["Ankle mobility"],
          "progressionStages": ["Supported"],
          "sourceRelationships": ["Category", "Family"],
          "sourcePages": [151],
          "mps": {"difficulty": "Beginner"},
          "readiness": {"sourceMetadataComplete": true}
        }, {
          "sourceRow": 5,
          "id": "source_mat_teaser_row_5",
          "exercise": "Teaser",
          "apparatus": "Mat",
          "equipmentKey": "mat",
          "familyIds": ["10"],
          "familyNames": ["Abdominals"],
          "categories": ["Abdominal Work"],
          "bodyRegions": ["Trunk"],
          "movementGoals": ["Trunk control"],
          "progressionStages": ["Full"],
          "sourceRelationships": ["Category", "Family"],
          "sourcePages": [152],
          "mps": {"difficulty": "Intermediate"},
          "readiness": {"sourceMetadataComplete": true}
        }]
      }
    }"#;

    const STYLES_JSON: &str = r#"{
      "schemaVersion": 1,
      "primaryProfile": "mono-gesture-ink-pilates-v1",
      "profiles": [{
        "id": "mono-gesture-ink-pilates-v1",
        "status": "approved",
        "characterReference": "teacher-01",
        "visualLanguage": {"medium": "ink"},
        "styleReference": "private://style/reference",
        "composition": {"background": "white"},
        "palette": {"ink": "#292526", "cheekAccent": "#D98F9A"},
        "lockedCharacterRules": [],
        "negativePrompt": [],
        "reviewChecklist": []
      }]
    }"#;

    const MANIFEST_JSON: &str = r#"{
      "primaryStyleProfile": "mono-gesture-ink-pilates-v1",
      "character": {
        "id": "teacher-01",
        "status": "approved",
        "version": 1,
        "canonicalSheet": "private/teacher.png",
        "fixedAttributes": {
          "top": "fitted off-white cropped Pilates camisole with thin straps",
          "bottom": "dark charcoal high-waisted fitted Pilates biker shorts ending mid-thigh"
        },
        "references": [{
          "id": "face-front",
          "role": "face_identity",
          "source": "private://teacher-01/face-front",
          "required": true
        }]
      }
    }"#;

    struct TestApp {
        app: Router,
        repository: Arc<FlashcardRepository>,
        catalog: CanonicalCatalog,
    }

    fn test_app() -> TestApp {
        let catalog = CanonicalCatalog::load_json(CATALOG_JSON).expect("load test catalog");
        let repository =
            FlashcardRepository::open_in_memory(catalog.clone()).expect("open repository");
        let state = AppState::from_documents(
            repository,
            catalog.clone(),
            STYLES_JSON,
            MANIFEST_JSON,
        )
        .expect("build app state");
        let repository = state.repository.clone();
        TestApp {
            app: app(state),
            repository,
            catalog,
        }
    }

    fn auth(studio_id: &str) -> AuthContext {
        AuthContext::new(
            studio_id,
            "teacher-01",
            [
                READ_CATALOG,
                WRITE_DRAFT,
                GENERATE_ASSET,
                SUBMIT_REVIEW,
                PUBLISH,
            ],
        )
    }

    fn automated_reviewer(studio_id: &str) -> AuthContext {
        AuthContext::new(studio_id, "automated-review-service", [AUTOMATED_REVIEW])
    }

    fn visual_worker(studio_id: &str) -> AuthContext {
        AuthContext::new(studio_id, "visual-worker", [VISUAL_WORKER])
            .with_actor_kind(AuditActorKind::System)
    }

    fn card(id: &str) -> FlashcardCard {
        FlashcardCard {
            id: id.to_owned(),
            source_exercise_id: "source_chair_achilles_stretch_row_4".to_owned(),
            category: "Lower Body Work".to_owned(),
            teaching_copy_json: json!({"cue": "Lengthen"}),
            style_profile: LOCKED_STYLE_PROFILE.to_owned(),
            character_id: LOCKED_CHARACTER_ID.to_owned(),
            current_asset_id: None,
            version: 1,
            status: FlashcardStatus::Draft,
        }
    }

    fn request(
        method: Method,
        uri: &str,
        auth_context: AuthContext,
        body: Option<Value>,
    ) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        let body = if let Some(value) = body {
            builder = builder.header("content-type", "application/json");
            Body::from(serde_json::to_vec(&value).expect("serialize request body"))
        } else {
            Body::empty()
        };
        let mut request = builder.body(body).expect("build request");
        request.extensions_mut().insert(auth_context);
        request
    }

    fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
        let runtime = tokio::runtime::Runtime::new().expect("create test runtime");
        runtime.block_on(async {
            let response = app.clone().oneshot(request).await.expect("route response");
            let status = response.status();
            let bytes = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read response body");
            let body = if bytes.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(&bytes)
                    .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
            };
            (status, body)
        })
    }

    #[test]
    fn list_returns_only_cards_scoped_to_the_authenticated_studio() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();
        fixture
            .repository
            .create_flashcard_for_studio("studio-b", "teacher-01", &card("card-b"))
            .unwrap();

        let (status, body) = send(
            &fixture.app,
            request(Method::GET, "/api/flashcards", auth("studio-a"), None),
        );

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0]["id"], "card-a");
    }

    #[test]
    fn list_honors_query_category_level_and_status_together() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();
        let mut teaser = card("card-teaser");
        teaser.source_exercise_id = "source_mat_teaser_row_5".to_owned();
        teaser.category = "Abdominal Work".to_owned();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &teaser)
            .unwrap();

        let (status, body) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/flashcards?query=mat&category=Abdominal%20Work&level=Intermediate&status=draft",
                auth("studio-a"),
                None,
            ),
        );

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0]["id"], "card-teaser");
    }

    #[test]
    fn read_routes_return_scoped_compact_context() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();

        let (status, flashcard) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/flashcards/card-a",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(flashcard["source_exercise_id"], "source_chair_achilles_stretch_row_4");

        let (status, context) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/catalog/exercises/source_chair_achilles_stretch_row_4/context",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(context["exercise"]["name"], "Achilles Stretch");
        assert_eq!(context["apparatus"]["equipment_key"], "chair");
        assert_eq!(context["families"][0]["name"], "Footwork");
        assert!(context["exercise"].get("source_pages").is_none());
        assert!(!serde_json::to_string(&context).unwrap().contains("Teaser"));
    }

    #[test]
    fn post_rejects_an_unknown_source_exercise_id() {
        let fixture = test_app();
        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards",
                auth("studio-a"),
                Some(json!({
                    "source_exercise_id": "not-in-the-workbook",
                    "teaching_copy_json": {"cue": "Breathe"}
                })),
            ),
        );

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["message"], "unknown source exercise ID");
    }

    #[test]
    fn patch_does_not_accept_source_exercise_or_style_fields() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();

        for patch in [
            json!({
                "source_exercise_id": "not-in-the-workbook",
                "teaching_copy_json": {"cue": "Changed"}
            }),
            json!({
                "style_profile": "another-style",
                "teaching_copy_json": {"cue": "Changed"}
            }),
        ] {
            let (status, _) = send(
                &fixture.app,
                request(
                    Method::PATCH,
                    "/api/flashcards/card-a",
                    auth("studio-a"),
                    Some(patch),
                ),
            );
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        }
        let persisted = fixture
            .repository
            .get_flashcard_for_studio("studio-a", "card-a")
            .unwrap()
            .unwrap();
        assert_eq!(
            persisted.source_exercise_id,
            "source_chair_achilles_stretch_row_4"
        );
        assert_eq!(persisted.style_profile, LOCKED_STYLE_PROFILE);
    }

    #[test]
    fn publish_returns_conflict_until_all_approval_prerequisites_pass() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/publish",
                auth("studio-a"),
                None,
            ),
        );

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(
            fixture
                .repository
                .get_flashcard_for_studio("studio-a", "card-a")
                .unwrap()
                .unwrap()
                .status,
            FlashcardStatus::Draft
        );
    }

    #[test]
    fn visual_brief_rejects_noncanonical_apparatus() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/visual-briefs",
                auth("studio-a"),
                Some(json!({
                    "pose_json": {"position": "standing"},
                    "apparatus": "Reformer"
                })),
            ),
        );

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(fixture.repository.audit_event_count("card-a").unwrap(), 1);
    }

    #[test]
    fn auth_context_and_publish_scope_are_server_authoritative() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();

        let incomplete = AuthContext::new("", "teacher-01", [READ_CATALOG]);
        let (status, _) = send(
            &fixture.app,
            request(Method::GET, "/api/flashcards", incomplete, None),
        );
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        let draft_only = AuthContext::new(
            "studio-a",
            "teacher-01",
            [READ_CATALOG, WRITE_DRAFT, GENERATE_ASSET, SUBMIT_REVIEW],
        );
        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/publish",
                draft_only,
                Some(json!({
                    "studio_id": "studio-b",
                    "teacher_id": "attacker",
                    "permissions": ["publish"]
                })),
            ),
        );
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn automated_review_requires_the_trusted_service_scope() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/reviews",
                auth("studio-a"),
                Some(json!({
                    "asset_id": "asset-a",
                    "passed": true,
                    "findings_json": []
                })),
            ),
        );

        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn visual_worker_callback_is_service_only_and_persists_atomically() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();
        fixture
            .repository
            .save_visual_brief_for_studio(
                "studio-a",
                "teacher-01",
                &VisualBrief {
                    id: "brief-a".into(),
                    card_id: "card-a".into(),
                    exercise_id: "source_chair_achilles_stretch_row_4".into(),
                    style_profile: LOCKED_STYLE_PROFILE.into(),
                    character_id: LOCKED_CHARACTER_ID.into(),
                    outfit: LOCKED_OUTFIT.into(),
                    pose_json: json!({"landmarks": [], "contact_points": []}),
                    apparatus: "Chair".into(),
                    palette_json: json!({"cheekAccent": DUSTY_ROSE_CHEEK_ACCENT}),
                    must_show_json: json!([]),
                    must_not_show_json: json!(["arrows", "text", "logos", "watermark"]),
                    version: 1,
                },
            )
            .unwrap();
        fixture
            .repository
            .create_job_for_studio(
                "studio-a",
                "teacher-01",
                &FlashcardJob {
                    id: "job-a".into(),
                    card_id: "card-a".into(),
                    kind: FlashcardJobKind::Generate,
                    status: FlashcardJobStatus::Queued,
                    input_json: json!({"brief_id": "brief-a"}),
                    output_json: None,
                    error: None,
                },
            )
            .unwrap();

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::FORBIDDEN);

        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                visual_worker("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "running");

        let (status, body) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/flashcards/card-a/jobs/job-a",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "running");

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                auth("studio-a"),
                Some(json!({
                    "status": "succeeded",
                    "asset": {
                        "id": "asset-a",
                        "brief_id": "brief-a",
                        "repo_path": "object://mps-flashcards/card-a-v1.png",
                        "provider_job_id": "provider-a",
                        "version": 1
                    },
                    "review": {
                        "id": "review-a",
                        "asset_id": "asset-a",
                        "passed": true,
                        "findings_json": []
                    }
                })),
        );
        assert_eq!(status, StatusCode::FORBIDDEN);

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                AuthContext::new("studio-a", "not-a-worker", [VISUAL_WORKER]),
                Some(json!({"status": "failed"})),
            ),
        );
        assert_eq!(status, StatusCode::FORBIDDEN);

        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                visual_worker("studio-a"),
                Some(json!({
                    "status": "succeeded",
                    "asset": {
                        "id": "asset-a",
                        "brief_id": "brief-a",
                        "repo_path": "object://mps-flashcards/card-a-v1.png",
                        "provider_job_id": "provider-a",
                        "version": 1
                    },
                    "review": {
                        "id": "review-a",
                        "asset_id": "asset-a",
                        "passed": true,
                        "findings_json": []
                    }
                })),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "succeeded");
        assert_eq!(
            fixture
                .repository
                .get_flashcard_for_studio("studio-a", "card-a")
                .unwrap()
                .unwrap()
                .status,
            FlashcardStatus::NeedsReview
        );

        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                visual_worker("studio-a"),
                Some(json!({
                    "status": "succeeded",
                    "asset": {
                        "id": "asset-a",
                        "brief_id": "brief-a",
                        "repo_path": "object://mps-flashcards/card-a-v1.png",
                        "provider_job_id": "provider-a",
                        "version": 99
                    },
                    "review": {
                        "id": "review-a",
                        "asset_id": "asset-a",
                        "passed": true,
                        "findings_json": []
                    }
                })),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "succeeded");

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                visual_worker("studio-a"),
                Some(json!({
                    "status": "failed",
                    "error_code": "image_generation_failed"
                })),
            ),
        );
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[test]
    fn every_mutation_route_adds_an_audit_row() {
        let fixture = test_app();

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards",
                auth("studio-a"),
                Some(json!({
                    "id": "card-a",
                    "source_exercise_id": "source_chair_achilles_stretch_row_4",
                    "teaching_copy_json": {"cue": "Lengthen"}
                })),
            ),
        );
        assert_eq!(status, StatusCode::CREATED);
        let mut audits = 1;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        let (status, _) = send(
            &fixture.app,
            request(
                Method::PATCH,
                "/api/flashcards/card-a",
                auth("studio-a"),
                Some(json!({"teaching_copy_json": {"cue": "Breathe"}})),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        audits += 1;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        let (status, brief) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/visual-briefs",
                auth("studio-a"),
                Some(json!({
                    "id": "brief-a",
                    "pose_json": {"position": "standing"},
                    "must_show_json": ["ankle alignment"]
                })),
            ),
        );
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(brief["palette_json"]["cheekAccent"], DUSTY_ROSE_CHEEK_ACCENT);
        audits += 1;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/jobs",
                auth("studio-a"),
                Some(json!({
                    "id": "job-a",
                    "kind": "generate",
                    "brief_id": "brief-a"
                })),
            ),
        );
        assert_eq!(status, StatusCode::ACCEPTED);
        audits += 2;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        let (status, job) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/flashcards/card-a/jobs/job-a",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(job["status"], "queued");
        assert_eq!(fixture.repository.audit_event_count("card-a").unwrap(), audits);

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/submit-review",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        audits += 1;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        fixture
            .repository
            .record_asset(&FlashcardAsset {
                id: "asset-a".to_owned(),
                card_id: "card-a".to_owned(),
                brief_id: "brief-a".to_owned(),
                repo_path: "private/draft.png".to_owned(),
                provider_job_id: Some("provider-secret-id".to_owned()),
                version: 1,
                status: FlashcardAssetStatus::Approved,
            })
            .unwrap();
        audits += 1;

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/reviews",
                automated_reviewer("studio-a"),
                Some(json!({
                    "id": "automated-review-a",
                    "asset_id": "asset-a",
                    "passed": true,
                    "findings_json": []
                })),
            ),
        );
        assert_eq!(status, StatusCode::CREATED);
        audits += 1;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/approve",
                auth("studio-a"),
                Some(json!({"review_id": "teacher-review-a"})),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        audits += 2;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );

        let (status, published) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/publish",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(published["card"]["status"], "published");
        audits += 1;
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            audits
        );
    }

    #[test]
    fn public_character_reference_omits_private_source_paths() {
        let fixture = test_app();

        let (status, visual_contract) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/visual-contract",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        let serialized_visual = serde_json::to_string(&visual_contract).unwrap();
        assert!(!serialized_visual.contains("private://"));
        assert!(!serialized_visual.contains("styleReference"));
        assert_eq!(
            visual_contract["palette"]["cheekAccent"],
            DUSTY_ROSE_CHEEK_ACCENT
        );

        let (status, body) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/character-reference",
                auth("studio-a"),
                None,
            ),
        );

        assert_eq!(status, StatusCode::OK);
        let serialized = serde_json::to_string(&body).unwrap();
        assert!(!serialized.contains("private://"));
        assert!(!serialized.contains("canonicalSheet"));
        assert!(!serialized.contains("private/teacher.png"));
        assert_eq!(body["id"], LOCKED_CHARACTER_ID);
    }

    #[test]
    fn catalog_provenance_requires_the_canonical_workbook() {
        validate_catalog_provenance(CATALOG_JSON).unwrap();
        let missing_optional_path = CATALOG_JSON.replace(
            ",\n        \"path\": \"data/reference/MPS_Database_v1_Core.xlsx\"",
            "",
        );
        validate_catalog_provenance(&missing_optional_path).unwrap();
        let wrong_source = CATALOG_JSON.replace(
            "MPS_Database_v1_Core.xlsx",
            "AI_Generated_Exercises.xlsx",
        );

        assert!(validate_catalog_provenance(&wrong_source).is_err());
    }
}
