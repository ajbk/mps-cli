use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context};
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::{header, request::Parts, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{async_trait, Json, Router};
use mps_db::{
    AuditActorKind, FlashcardAsset, FlashcardAssetStatus, FlashcardJob, FlashcardJobKind,
    FlashcardJobStatus, FlashcardListFilter, FlashcardPatch, FlashcardRepository,
    FlashcardReview, FlashcardVisualDetails, FlashcardWorkerCompletion, ReviewerKind,
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
pub const BROWSER_SESSION_COOKIE: &str = "mps_session";
pub const BROWSER_CSRF_COOKIE: &str = "mps_csrf";
pub const BROWSER_CSRF_HEADER: &str = "x-mps-csrf";
pub const BROWSER_SESSION_TTL_SECONDS: u64 = 30 * 60;

#[derive(Clone, Default)]
pub struct BrowserSessionStore {
    sessions: Arc<Mutex<HashMap<String, BrowserSession>>>,
}

#[derive(Clone)]
struct BrowserSession {
    context: AuthContext,
    expires_at: SystemTime,
    csrf_token: String,
}

#[derive(Debug, Clone)]
pub struct BrowserSessionCredentials {
    pub session_token: String,
    pub csrf_token: String,
}

impl BrowserSessionStore {
    pub fn issue(&self, context: AuthContext) -> anyhow::Result<BrowserSessionCredentials> {
        let mut session_bytes = [0_u8; 32];
        getrandom::getrandom(&mut session_bytes).context("generate browser session token")?;
        let session_token = session_bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let mut csrf_bytes = [0_u8; 32];
        getrandom::getrandom(&mut csrf_bytes).context("generate browser CSRF token")?;
        let csrf_token = csrf_bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let expires_at = SystemTime::now() + Duration::from_secs(BROWSER_SESSION_TTL_SECONDS);
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| anyhow!("browser session store is unavailable"))?;
        sessions.retain(|_, session| session.expires_at > SystemTime::now());
        sessions.insert(
            session_token.clone(),
            BrowserSession {
                context,
                expires_at,
                csrf_token: csrf_token.clone(),
            },
        );
        Ok(BrowserSessionCredentials {
            session_token,
            csrf_token,
        })
    }

    pub fn authenticate(&self, token: &str) -> Option<AuthContext> {
        let mut sessions = self.sessions.lock().ok()?;
        let session = sessions.get(token).cloned()?;
        if session.expires_at <= SystemTime::now() {
            sessions.remove(token);
            return None;
        }
        Some(session.context)
    }

    pub fn csrf_valid(&self, session_token: &str, csrf_token: &str) -> bool {
        let mut sessions = match self.sessions.lock() {
            Ok(sessions) => sessions,
            Err(_) => return false,
        };
        let Some(session) = sessions.get(session_token) else {
            return false;
        };
        if session.expires_at <= SystemTime::now() {
            sessions.remove(session_token);
            return false;
        }
        session.csrf_token == csrf_token
    }

    pub fn revoke(&self, token: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(token);
        }
    }
}

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
    browser_sessions: BrowserSessionStore,
    asset_root: Option<Arc<PathBuf>>,
    session_cookie_secure: bool,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub database_path: String,
    pub catalog_path: String,
    pub visual_styles_path: String,
    pub visual_manifest_path: String,
    pub asset_root: Option<String>,
    pub session_cookie_secure: bool,
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
        let state = Self::from_documents(
            repository,
            catalog,
            &visual_styles_json,
            &visual_manifest_json,
        )?;
        Ok(state
            .with_asset_root(config.asset_root.as_deref())
            .with_session_cookie_secure(config.session_cookie_secure))
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
            browser_sessions: BrowserSessionStore::default(),
            asset_root: None,
            session_cookie_secure: true,
        })
    }

    pub fn with_asset_root(mut self, root: Option<&str>) -> Self {
        self.asset_root = root
            .filter(|value| !value.trim().is_empty())
            .map(|value| Arc::new(PathBuf::from(value)));
        self
    }

    pub fn with_session_cookie_secure(mut self, secure: bool) -> Self {
        self.session_cookie_secure = secure;
        self
    }

    pub fn browser_session_store(&self) -> BrowserSessionStore {
        self.browser_sessions.clone()
    }

    pub fn session_cookie_secure(&self) -> bool {
        self.session_cookie_secure
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/session",
            get(session_status)
                .post(create_browser_session)
                .delete(clear_browser_session),
        )
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
            "/api/flashcards/:id/assets/:asset_id",
            get(get_flashcard_asset),
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

#[derive(Debug, Serialize)]
struct SessionResponse {
    authenticated: bool,
    studio_id: String,
    teacher_id: String,
    expires_in_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    csrf_token: Option<String>,
}

async fn create_browser_session(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Response, ApiError> {
    let credentials = state
        .browser_sessions
        .issue(auth)
        .map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::NO_CONTENT,
        [
            (
                header::SET_COOKIE,
                session_cookie(&credentials.session_token, state.session_cookie_secure),
            ),
            (
                header::SET_COOKIE,
                csrf_cookie(&credentials.csrf_token, state.session_cookie_secure),
            ),
        ],
    )
        .into_response())
}

async fn session_status(
    State(state): State<AppState>,
    auth: AuthContext,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let csrf_token = session_token_from_cookie(&headers).and_then(|session_token| {
        csrf_token_from_cookie(&headers).filter(|csrf_token| {
            state
                .browser_sessions
                .csrf_valid(&session_token, csrf_token)
        })
    });
    Ok(Json(SessionResponse {
        authenticated: true,
        studio_id: auth.studio_id,
        teacher_id: auth.teacher_id,
        expires_in_seconds: BROWSER_SESSION_TTL_SECONDS,
        csrf_token,
    }))
}

async fn clear_browser_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(token) = session_token_from_cookie(&headers) {
        state.browser_sessions.revoke(&token);
    }
    Ok((
        StatusCode::NO_CONTENT,
        [
            (header::SET_COOKIE, clear_session_cookie(state.session_cookie_secure)),
            (header::SET_COOKIE, clear_csrf_cookie(state.session_cookie_secure)),
        ],
    )
        .into_response())
}

pub fn session_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value.split(';').find_map(|pair| {
                let (name, token) = pair.trim().split_once('=')?;
                (name == BROWSER_SESSION_COOKIE && !token.is_empty()).then(|| token.to_owned())
            })
        })
}

pub fn csrf_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value.split(';').find_map(|pair| {
                let (name, token) = pair.trim().split_once('=')?;
                (name == BROWSER_CSRF_COOKIE && !token.is_empty()).then(|| token.to_owned())
            })
        })
}

pub fn browser_csrf_is_valid(
    headers: &HeaderMap,
    session_token: &str,
    sessions: &BrowserSessionStore,
) -> bool {
    let Some(header_token) = headers
        .get(BROWSER_CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    csrf_token_from_cookie(headers).as_deref() == Some(header_token)
        && sessions.csrf_valid(session_token, header_token)
}

pub fn session_cookie(token: &str, secure: bool) -> String {
    format!(
        "{BROWSER_SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={BROWSER_SESSION_TTL_SECONDS}{}",
        if secure { "; Secure" } else { "" }
    )
}

fn clear_session_cookie(secure: bool) -> String {
    format!(
        "{BROWSER_SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
        if secure { "; Secure" } else { "" }
    )
}

pub fn csrf_cookie(token: &str, secure: bool) -> String {
    format!(
        "{BROWSER_CSRF_COOKIE}={token}; Path=/; SameSite=Lax; Max-Age={BROWSER_SESSION_TTL_SECONDS}{}",
        if secure { "; Secure" } else { "" }
    )
}

fn clear_csrf_cookie(secure: bool) -> String {
    format!(
        "{BROWSER_CSRF_COOKIE}=; Path=/; SameSite=Lax; Max-Age=0{}",
        if secure { "; Secure" } else { "" }
    )
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
    asset: Option<AssetResponse>,
    automated_review: Option<AutomatedReviewResponse>,
    version: i64,
    status: FlashcardStatus,
}

#[derive(Debug, Serialize, Clone)]
struct AssetResponse {
    id: String,
    version: i64,
    status: &'static str,
    url: String,
}

#[derive(Debug, Serialize, Clone)]
struct AutomatedReviewResponse {
    id: String,
    asset_id: String,
    status: &'static str,
    findings: Value,
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
    output_json: Option<Value>,
    asset: Option<AssetResponse>,
    automated_review: Option<AutomatedReviewResponse>,
}

#[derive(Debug, Serialize)]
struct WorkerClaimResponse {
    #[serde(flatten)]
    job: JobResponse,
    claim_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerClaimRequest {
    claim_id: String,
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
    claim_id: String,
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
                    claim_id: self.claim_id,
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
                    claim_id: self.claim_id,
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
    let studio_id = auth.studio_id.clone();
    let list_studio_id = studio_id.clone();
    let filter = FlashcardListFilter {
        status: query.status,
        category: query.category,
    };
    let cards = run_repository(state.repository.clone(), move |repository| {
        repository.list_flashcards_for_studio(&list_studio_id, filter)
    })
    .await?;
    let query_text = query.query.unwrap_or_default().trim().to_lowercase();
    let level = query.level.unwrap_or_default().trim().to_lowercase();
    let mut responses = Vec::new();
    for card in cards {
        let Some(exercise) = state.catalog.find(&card.source_exercise_id) else {
            continue;
        };
        let response = flashcard_response_for_studio(&state, &studio_id, card, exercise).await?;
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
        if matches_query && matches_level {
            responses.push(response);
        }
    }
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
    let lookup_studio_id = studio_id.clone();
    let requested_id = id.clone();
    let card = run_repository(state.repository.clone(), move |repository| {
        repository.get_flashcard_for_studio(&lookup_studio_id, &requested_id)
    })
    .await?
    .ok_or_else(|| ApiError::not_found("flashcard not found"))?;
    let exercise = state
        .catalog
        .find(&card.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok(Json(
        flashcard_response_for_studio(&state, &studio_id, card, exercise).await?,
    ))
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
    let write_studio_id = studio_id.clone();
    let teacher_id = auth.teacher_id;
    let actor_kind = auth.actor_kind;
    let created = run_repository(state.repository.clone(), move |repository| {
        repository.create_flashcard_for_actor(&write_studio_id, &teacher_id, actor_kind, &card)
    })
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(
            flashcard_response_for_studio(&state, &studio_id, created, &exercise).await?,
        ),
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
    let write_studio_id = studio_id.clone();
    let teacher_id = auth.teacher_id;
    let actor_kind = auth.actor_kind;
    let updated = run_repository(state.repository.clone(), move |repository| {
        repository.update_flashcard_for_actor(&write_studio_id, &teacher_id, actor_kind, &id, patch)
    })
    .await?;
    let exercise = state
        .catalog
        .find(&updated.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok(Json(
        flashcard_response_for_studio(&state, &studio_id, updated, exercise).await?,
    ))
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
    let lookup_studio_id = studio_id.clone();
    let lookup_card_id = id.clone();
    let lookup_job_id = job_id.clone();
    let job = run_repository(state.repository.clone(), move |repository| {
        repository.get_job_for_studio(&lookup_studio_id, &lookup_card_id, &lookup_job_id)
    })
    .await?
    .ok_or_else(|| ApiError::not_found("flashcard job not found"))?;
    Ok(Json(job_response_for_studio(&state, &studio_id, job).await?))
}

async fn get_flashcard_asset(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((card_id, asset_id)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    auth.require(READ_CATALOG)?;
    let studio_id = auth.studio_id.clone();
    let lookup_card_id = card_id.clone();
    let lookup_asset_id = asset_id.clone();
    let asset = run_repository(state.repository.clone(), move |repository| {
        repository.asset_for_studio(&studio_id, &lookup_card_id, &lookup_asset_id)
    })
    .await?
    .ok_or_else(|| ApiError::not_found("flashcard asset not found"))?;
    let key = safe_asset_key(&asset.repo_path)
        .ok_or_else(|| ApiError::not_found("flashcard asset is not publicly available"))?;
    let root = state
        .asset_root
        .as_ref()
        .ok_or_else(|| ApiError::not_found("flashcard asset storage is not configured"))?;
    let root = fs::canonicalize(root.as_ref())
        .map_err(|_| ApiError::not_found("flashcard asset storage is not available"))?;
    let candidate = fs::canonicalize(root.join(&key))
        .map_err(|_| ApiError::not_found("flashcard asset is not available"))?;
    if !candidate.starts_with(&root) {
        return Err(ApiError::not_found("flashcard asset is not available"));
    }
    let bytes = fs::read(candidate)
        .map_err(|_| ApiError::not_found("flashcard asset is not available"))?;
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, asset_content_type(&key)),
            (header::CACHE_CONTROL, "private, max-age=60"),
        ],
        Body::from(bytes),
    )
        .into_response())
}

async fn claim_visual_job(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((card_id, job_id)): Path<(String, String)>,
    Json(request): Json<WorkerClaimRequest>,
) -> Result<Json<WorkerClaimResponse>, ApiError> {
    auth.require(VISUAL_WORKER)?;
    if auth.actor_kind != AuditActorKind::System {
        return Err(ApiError::forbidden("visual worker service identity is required"));
    }
    validate_worker_claim_id(&request.claim_id)?;
    let studio_id = auth.studio_id.clone();
    let worker_id = auth.teacher_id;
    let claim_id = request.claim_id;
    let claimed = run_repository(state.repository.clone(), move |repository| {
        repository.claim_job_for_worker(
            &studio_id,
            &worker_id,
            &card_id,
            &job_id,
            &claim_id,
        )
    })
    .await?;
    worker_claim_response(claimed)
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
    let studio_id = auth.studio_id.clone();
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
    Ok(Json(
        job_response_for_studio(&state, &auth.studio_id, completed).await?,
    ))
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
    let review_studio_id = studio_id.clone();
    let teacher_id = auth.teacher_id.clone();
    run_repository(state.repository.clone(), move |repository| {
        repository.record_automated_review_for_studio(&review_studio_id, &teacher_id, &review)
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
            card: flashcard_response_for_studio(&state, &studio_id, card, exercise).await?,
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
        repository.transition_generating_to_needs_review_if_idle_for_actor(
            &studio_id,
            &teacher_id,
            actor_kind,
            &card_id,
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
    let studio_id = auth.studio_id.clone();
    let card = get_scoped_card(state, auth, id).await?;
    let exercise = state
        .catalog
        .find(&card.source_exercise_id)
        .ok_or_else(|| ApiError::conflict("flashcard source is no longer canonical"))?;
    Ok(Json(MutationResponse {
        card: flashcard_response_for_studio(&state, &studio_id, card, exercise).await?,
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

async fn flashcard_response_for_studio(
    state: &AppState,
    studio_id: &str,
    card: FlashcardCard,
    exercise: &CatalogExercise,
) -> Result<FlashcardResponse, ApiError> {
    let card_id = card.id.clone();
    let studio_id = studio_id.to_owned();
    let details = run_repository(state.repository.clone(), move |repository| {
        repository.visual_details_for_studio(&studio_id, &card_id)
    })
    .await?;
    Ok(flashcard_response(card, exercise, &details))
}

fn flashcard_response(
    card: FlashcardCard,
    exercise: &CatalogExercise,
    details: &FlashcardVisualDetails,
) -> FlashcardResponse {
    let asset = details
        .asset
        .as_ref()
        .and_then(|asset| asset_response(&card.id, asset));
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
        asset,
        automated_review: details
            .automated_review
            .as_ref()
            .map(automated_review_response),
        version: card.version,
        status: card.status,
    }
}

fn asset_response(card_id: &str, asset: &FlashcardAsset) -> Option<AssetResponse> {
    safe_asset_key(&asset.repo_path)?;
    Some(AssetResponse {
        id: asset.id.clone(),
        version: asset.version,
        status: asset_status_label(&asset.status),
        url: format!("/api/flashcards/{card_id}/assets/{}", asset.id),
    })
}

fn automated_review_response(review: &FlashcardReview) -> AutomatedReviewResponse {
    AutomatedReviewResponse {
        id: review.id.clone(),
        asset_id: review.asset_id.clone(),
        status: if review.passed { "passed" } else { "failed" },
        findings: sanitize_public_findings(&review.findings_json),
    }
}

fn asset_status_label(status: &FlashcardAssetStatus) -> &'static str {
    match status {
        FlashcardAssetStatus::Generating => "generating",
        FlashcardAssetStatus::NeedsReview => "needs-review",
        FlashcardAssetStatus::Rejected => "rejected",
        FlashcardAssetStatus::Approved => "approved",
    }
}

fn safe_asset_key(repo_path: &str) -> Option<String> {
    let key = repo_path
        .strip_prefix("object://mps-flashcards/")
        .or_else(|| repo_path.strip_prefix("web/assets/flashcard-images/"))
        .or_else(|| repo_path.strip_prefix("assets/flashcard-images/"))?;
    if key.is_empty()
        || key.starts_with('/')
        || key.contains('\\')
        || key.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
    {
        return None;
    }
    let extension = key.rsplit('.').next()?.to_ascii_lowercase();
    matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp").then(|| key.to_owned())
}

fn asset_content_type(key: &str) -> &'static str {
    match key.rsplit('.').next().unwrap_or_default().to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "image/png",
    }
}

fn sanitize_public_findings(value: &Value) -> Value {
    let Some(findings) = value.as_array() else {
        return json!([]);
    };
    Value::Array(
        findings
            .iter()
            .take(128)
            .filter_map(|finding| {
                let object = finding.as_object()?;
                let mut safe = serde_json::Map::new();
                for key in ["code", "severity", "check", "message"] {
                    let Some(value) = object.get(key) else {
                        continue;
                    };
                    let Some(text) = value.as_str() else {
                        continue;
                    };
                    let text = text
                        .chars()
                        .filter(|character| !character.is_control() || *character == '\n')
                        .take(500)
                        .collect::<String>();
                    if !text.is_empty() {
                        safe.insert(key.to_owned(), Value::String(text));
                    }
                }
                (!safe.is_empty()).then_some(Value::Object(safe))
            })
            .collect(),
    )
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
        output_json: job.output_json.map(sanitize_public_value),
        asset: None,
        automated_review: None,
    }
}

fn job_visual_ids(job: &FlashcardJob) -> (Option<String>, Option<String>) {
    let output = job.output_json.as_ref();
    let asset_id = output
        .and_then(|value| value.get("asset_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let review_id = output
        .and_then(|value| value.get("review_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    (asset_id, review_id)
}

async fn job_response_for_studio(
    state: &AppState,
    studio_id: &str,
    job: FlashcardJob,
) -> Result<JobResponse, ApiError> {
    let card_id = job.card_id.clone();
    let studio_id = studio_id.to_owned();
    let (asset_id, review_id) = job_visual_ids(&job);
    let details = run_repository(state.repository.clone(), move |repository| {
        repository.visual_details_for_job_for_studio(
            &studio_id,
            &card_id,
            asset_id.as_deref(),
            review_id.as_deref(),
        )
    })
    .await?;
    let mut response = job_response(job);
    response.asset = details
        .asset
        .as_ref()
        .and_then(|asset| asset_response(&response.card_id, asset));
    response.automated_review = details
        .automated_review
        .as_ref()
        .map(automated_review_response);
    Ok(response)
}

fn worker_claim_response(job: FlashcardJob) -> Result<Json<WorkerClaimResponse>, ApiError> {
    let claim_id = job
        .input_json
        .get("claim_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(ApiError::internal)?
        .to_owned();
    Ok(Json(WorkerClaimResponse {
        job: job_response(job.clone()),
        claim_id,
    }))
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
        || lower.starts_with("object://")
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

fn validate_worker_claim_id(value: &str) -> Result<(), ApiError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(ApiError::bad_request("worker claim ID is invalid"));
    }
    Ok(())
}

fn repository_error(error: anyhow::Error) -> ApiError {
    let message = error.to_string();
    if message.contains("not found") {
        ApiError::not_found("resource not found")
    } else if message.contains("invalid flashcard status transition")
        || message.contains("generation job is active")
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
        visual_worker_as(studio_id, "visual-worker")
    }

    fn visual_worker_as(studio_id: &str, worker_id: &str) -> AuthContext {
        AuthContext::new(studio_id, worker_id, [VISUAL_WORKER])
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
    fn browser_session_is_opaque_http_only_and_revocable() {
        let store = BrowserSessionStore::default();
        let context = auth("studio-a");
        let credentials = store.issue(context.clone()).expect("issue session");
        assert_eq!(credentials.session_token.len(), 64);
        assert_eq!(credentials.csrf_token.len(), 64);
        assert_eq!(
            store.authenticate(&credentials.session_token).unwrap().studio_id,
            "studio-a"
        );
        let cookie = session_cookie(&credentials.session_token, true);
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(!cookie.contains("studio-a"));
        let csrf = csrf_cookie(&credentials.csrf_token, true);
        assert!(!csrf.contains("HttpOnly"));
        assert!(csrf.contains("Secure"));

        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            format!(
                "{}={}; {}={}",
                BROWSER_SESSION_COOKIE,
                credentials.session_token,
                BROWSER_CSRF_COOKIE,
                credentials.csrf_token
            )
            .parse()
            .unwrap(),
        );
        headers.insert(
            BROWSER_CSRF_HEADER,
            credentials.csrf_token.parse().unwrap(),
        );
        assert!(browser_csrf_is_valid(
            &headers,
            &credentials.session_token,
            &store
        ));
        headers.remove(BROWSER_CSRF_HEADER);
        assert!(!browser_csrf_is_valid(
            &headers,
            &credentials.session_token,
            &store
        ));
        headers.insert(BROWSER_CSRF_HEADER, "wrong-token".parse().unwrap());
        assert!(!browser_csrf_is_valid(
            &headers,
            &credentials.session_token,
            &store
        ));
        store.revoke(&credentials.session_token);
        assert!(store.authenticate(&credentials.session_token).is_none());
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
    fn card_and_job_responses_expose_only_sanitized_asset_and_latest_review() {
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
                    pose_json: json!({"position": "standing"}),
                    apparatus: "Chair".into(),
                    palette_json: json!({"cheekAccent": DUSTY_ROSE_CHEEK_ACCENT}),
                    must_show_json: json!([]),
                    must_not_show_json: json!(["arrows"]),
                    version: 1,
                },
            )
            .unwrap();
        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/jobs",
                auth("studio-a"),
                Some(json!({"id": "job-a", "kind": "generate", "brief_id": "brief-a"})),
            ),
        );
        assert_eq!(status, StatusCode::ACCEPTED);
        fixture
            .repository
            .record_asset(&FlashcardAsset {
                id: "asset-a".into(),
                card_id: "card-a".into(),
                brief_id: "brief-a".into(),
                repo_path: "object://mps-flashcards/card-a-v1.png".into(),
                provider_job_id: Some("provider-secret".into()),
                version: 1,
                status: FlashcardAssetStatus::NeedsReview,
            })
            .unwrap();
        fixture
            .repository
            .record_review(&FlashcardReview {
                id: "review-a".into(),
                card_id: "card-a".into(),
                asset_id: "asset-a".into(),
                passed: false,
                findings_json: json!([{"code": "visual_check_failed", "check": "pose", "message": "Pose needs work"}]),
                reviewer_kind: ReviewerKind::Automated,
                reviewer_id: None,
            })
            .unwrap();

        let (status, body) = send(
            &fixture.app,
            request(Method::GET, "/api/flashcards/card-a", auth("studio-a"), None),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["asset"]["id"], "asset-a");
        assert_eq!(body["asset"]["url"], "/api/flashcards/card-a/assets/asset-a");
        assert!(body["asset"].get("repo_path").is_none());
        assert!(body["asset"].get("provider_job_id").is_none());
        assert_eq!(body["automated_review"]["status"], "failed");
        assert_eq!(body["automated_review"]["findings"][0]["message"], "Pose needs work");

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
        assert!(job["asset"].is_null());
        assert!(job["automated_review"].is_null());
        assert!(serde_json::to_string(&job).unwrap().find("object://").is_none());
    }

    #[test]
    fn submit_review_rejects_a_card_with_an_active_generation_job() {
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
                    pose_json: json!({}),
                    apparatus: "Chair".into(),
                    palette_json: json!({"cheekAccent": DUSTY_ROSE_CHEEK_ACCENT}),
                    must_show_json: json!([]),
                    must_not_show_json: json!([]),
                    version: 1,
                },
            )
            .unwrap();
        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/jobs",
                auth("studio-a"),
                Some(json!({"id": "job-a", "kind": "generate", "brief_id": "brief-a"})),
            ),
        );
        assert_eq!(status, StatusCode::ACCEPTED);

        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/submit-review",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("generation job"));
        assert_eq!(fixture.repository.get_flashcard("card-a").unwrap().unwrap().status, FlashcardStatus::Generating);
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

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                visual_worker("studio-a"),
                Some(json!({"claim_id": "../unsafe"})),
            ),
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                visual_worker("studio-a"),
                Some(json!({"claim_id": "claim-1"})),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "running");
        let claim_id = body["claim_id"].as_str().unwrap().to_owned();

        let (status, repeated_claim) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                visual_worker("studio-a"),
                Some(json!({"claim_id": "claim-1"})),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(repeated_claim["status"], "running");
        assert_eq!(repeated_claim["claim_id"].as_str(), Some(claim_id.as_str()));

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                visual_worker_as("studio-a", "other-worker"),
                Some(json!({"claim_id": "claim-other"})),
            ),
        );
        assert_eq!(status, StatusCode::CONFLICT);

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/claim",
                visual_worker("studio-a"),
                Some(json!({"claim_id": "claim-1-retry"})),
            ),
        );
        assert_eq!(status, StatusCode::CONFLICT);

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
        assert!(body.get("claim_id").is_none());

        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                auth("studio-a"),
                Some(json!({
                    "claim_id": claim_id,
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
                    "claim_id": claim_id,
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
        assert_eq!(body["asset"]["id"], "asset-a");
        assert_eq!(body["automated_review"]["id"], "review-a");

        fixture
            .repository
            .transition_status_for_studio(
                "studio-a",
                "teacher-01",
                "card-a",
                FlashcardStatus::RevisionRequested,
            )
            .unwrap();
        fixture
            .repository
            .save_visual_brief_for_studio(
                "studio-a",
                "teacher-01",
                &VisualBrief {
                    id: "brief-b".into(),
                    card_id: "card-a".into(),
                    exercise_id: "source_chair_achilles_stretch_row_4".into(),
                    style_profile: LOCKED_STYLE_PROFILE.into(),
                    character_id: LOCKED_CHARACTER_ID.into(),
                    outfit: LOCKED_OUTFIT.into(),
                    pose_json: json!({"position": "standing-v2"}),
                    apparatus: "Chair".into(),
                    palette_json: json!({"cheekAccent": DUSTY_ROSE_CHEEK_ACCENT}),
                    must_show_json: json!([]),
                    must_not_show_json: json!(["arrows"]),
                    version: 2,
                },
            )
            .unwrap();
        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/flashcards/card-a/jobs",
                auth("studio-a"),
                Some(json!({"id": "job-b", "kind": "regenerate", "brief_id": "brief-b"})),
            ),
        );
        assert_eq!(status, StatusCode::ACCEPTED);
        let (status, claim) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-b/claim",
                visual_worker("studio-a"),
                Some(json!({"claim_id": "claim-b"})),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        let claim_b = claim["claim_id"].as_str().unwrap();
        let (status, _) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-b/complete",
                visual_worker("studio-a"),
                Some(json!({
                    "claim_id": claim_b,
                    "status": "succeeded",
                    "asset": {
                        "id": "asset-b",
                        "brief_id": "brief-b",
                        "repo_path": "object://mps-flashcards/card-a-v2.png",
                        "provider_job_id": "provider-b",
                        "version": 2
                    },
                    "review": {
                        "id": "review-b",
                        "asset_id": "asset-b",
                        "passed": true,
                        "findings_json": [{"check": "pose", "message": "Updated pose"}]
                    }
                })),
            ),
        );
        assert_eq!(status, StatusCode::OK);
        let (status, old_job) = send(
            &fixture.app,
            request(
                Method::GET,
                "/api/flashcards/card-a/jobs/job-a",
                auth("studio-a"),
                None,
            ),
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(old_job["asset"]["id"], "asset-a");
        assert_eq!(old_job["automated_review"]["id"], "review-a");
        assert_eq!(old_job["automated_review"]["findings"], json!([]));

        let (status, body) = send(
            &fixture.app,
            request(
                Method::POST,
                "/api/internal/flashcards/card-a/jobs/job-a/complete",
                visual_worker("studio-a"),
                Some(json!({
                    "claim_id": claim_id,
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
                    "claim_id": claim_id,
                    "status": "failed",
                    "error_code": "image_generation_failed"
                })),
            ),
        );
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[test]
    fn concurrent_submit_review_requests_have_one_atomic_transition() {
        let fixture = test_app();
        fixture
            .repository
            .create_flashcard_for_studio("studio-a", "teacher-01", &card("card-a"))
            .unwrap();
        fixture
            .repository
            .transition_status_for_studio(
                "studio-a",
                "teacher-01",
                "card-a",
                FlashcardStatus::Generating,
            )
            .unwrap();
        let baseline_audits = fixture.repository.audit_event_count("card-a").unwrap();

        let runtime = tokio::runtime::Runtime::new().expect("create test runtime");
        let (first, second) = runtime.block_on(async {
            tokio::join!(
                fixture.app.clone().oneshot(request(
                    Method::POST,
                    "/api/flashcards/card-a/submit-review",
                    auth("studio-a"),
                    None,
                )),
                fixture.app.clone().oneshot(request(
                    Method::POST,
                    "/api/flashcards/card-a/submit-review",
                    auth("studio-a"),
                    None,
                )),
            )
        });
        let first_status = first.expect("first submit response").status();
        let second_status = second.expect("second submit response").status();

        assert!(
            (first_status == StatusCode::OK && second_status == StatusCode::CONFLICT)
                || (first_status == StatusCode::CONFLICT && second_status == StatusCode::OK),
            "concurrent submit requests must yield exactly one winner: {first_status} / {second_status}"
        );
        assert_eq!(
            fixture
                .repository
                .get_flashcard_for_studio("studio-a", "card-a")
                .unwrap()
                .unwrap()
                .status,
            FlashcardStatus::NeedsReview
        );
        assert_eq!(
            fixture.repository.audit_event_count("card-a").unwrap(),
            baseline_audits + 1
        );
    }

    #[test]
    fn concurrent_visual_worker_claims_are_exclusive() {
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

        let runtime = tokio::runtime::Runtime::new().expect("create test runtime");
        let (first, second) = runtime.block_on(async {
            tokio::join!(
                fixture.app.clone().oneshot(request(
                    Method::POST,
                    "/api/internal/flashcards/card-a/jobs/job-a/claim",
                    visual_worker("studio-a"),
                    Some(json!({"claim_id": "attempt-a"})),
                )),
                fixture.app.clone().oneshot(request(
                    Method::POST,
                    "/api/internal/flashcards/card-a/jobs/job-a/claim",
                    visual_worker("studio-a"),
                    Some(json!({"claim_id": "attempt-b"})),
                )),
            )
        });
        let first_status = first.expect("first claim response").status();
        let second_status = second.expect("second claim response").status();

        assert!(
            (first_status == StatusCode::OK && second_status == StatusCode::CONFLICT)
                || (first_status == StatusCode::CONFLICT && second_status == StatusCode::OK),
            "concurrent claims must yield exactly one winner: {first_status} / {second_status}"
        );
        assert_eq!(
            fixture
                .repository
                .get_job_for_studio("studio-a", "card-a", "job-a")
                .unwrap()
                .unwrap()
                .status,
            FlashcardJobStatus::Running
        );
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
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(fixture.repository.audit_event_count("card-a").unwrap(), audits);

        // Worker completion owns generating -> needs-review. This direct transition
        // keeps this audit-route test focused without duplicating the worker test.
        fixture
            .repository
            .transition_status_for_studio(
                "studio-a",
                "teacher-01",
                "card-a",
                FlashcardStatus::NeedsReview,
            )
            .unwrap();
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
