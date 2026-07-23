use std::env;

use anyhow::{Context, Result};
use axum::extract::{Request, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Json;
use mps_server::{
    app, AppState, AuthContext, ServerConfig, AUTOMATED_REVIEW, DEFAULT_CATALOG_EXPORT_PATH,
    browser_csrf_is_valid, csrf_cookie, session_cookie, session_token_from_cookie, GENERATE_ASSET,
    READ_CATALOG, SUBMIT_REVIEW, VISUAL_WORKER, WRITE_DRAFT,
};
use mps_db::AuditActorKind;
use serde_json::json;

#[derive(Clone)]
struct StaticTokenAuth {
    teacher_bearer_token: String,
    teacher_context: AuthContext,
    automated_review_bearer_token: String,
    automated_review_context: AuthContext,
    mcp_service_bearer_token: String,
    visual_worker_bearer_token: String,
    visual_worker_context: AuthContext,
    browser_sessions: mps_server::BrowserSessionStore,
    session_cookie_secure: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig {
        database_path: required_env("MPS_DATABASE_PATH")?,
        catalog_path: env::var("MPS_CATALOG_PATH")
            .unwrap_or_else(|_| DEFAULT_CATALOG_EXPORT_PATH.to_owned()),
        visual_styles_path: required_env("MPS_VISUAL_STYLES_PATH")?,
        visual_manifest_path: required_env("MPS_VISUAL_MANIFEST_PATH")?,
        asset_root: env::var("MPS_ASSET_ROOT").ok(),
        session_cookie_secure: env::var("MPS_SESSION_COOKIE_SECURE")
            .map(|value| value.trim().to_ascii_lowercase() != "false")
            .unwrap_or(true),
    };
    let teacher_bearer_token = required_env("MPS_AUTH_TOKEN")?;
    let teacher_permissions = required_env("MPS_PERMISSIONS")?
        .split(',')
        .map(str::trim)
        .filter(|permission| !permission.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if teacher_permissions.iter().any(|permission| {
        matches!(permission.as_str(), AUTOMATED_REVIEW | VISUAL_WORKER)
    })
    {
        anyhow::bail!(
            "MPS_PERMISSIONS must not grant service-only review or worker scopes"
        );
    }
    let automated_review_bearer_token = required_env("MPS_AUTOMATED_REVIEW_TOKEN")?;
    if teacher_bearer_token == automated_review_bearer_token {
        anyhow::bail!("MPS_AUTH_TOKEN and MPS_AUTOMATED_REVIEW_TOKEN must differ");
    }
    let mcp_service_bearer_token = required_env("MPS_MCP_SERVICE_TOKEN")?;
    if mcp_service_bearer_token == teacher_bearer_token
        || mcp_service_bearer_token == automated_review_bearer_token
    {
        anyhow::bail!("MPS_MCP_SERVICE_TOKEN must differ from human and review service tokens");
    }
    let visual_worker_bearer_token = required_nonempty_env("MPS_VISUAL_WORKER_TOKEN")?;
    if visual_worker_bearer_token == teacher_bearer_token
        || visual_worker_bearer_token == automated_review_bearer_token
        || visual_worker_bearer_token == mcp_service_bearer_token
    {
        anyhow::bail!("MPS_VISUAL_WORKER_TOKEN must differ from other service tokens");
    }
    let studio_id = required_env("MPS_STUDIO_ID")?;
    let auth = StaticTokenAuth {
        teacher_bearer_token,
        teacher_context: AuthContext::new(
            studio_id.clone(),
            required_env("MPS_TEACHER_ID")?,
            teacher_permissions,
        ),
        automated_review_bearer_token,
        automated_review_context: AuthContext::new(
            studio_id.clone(),
            required_env("MPS_AUTOMATED_REVIEW_SERVICE_ID")?,
            [AUTOMATED_REVIEW],
        ),
        mcp_service_bearer_token,
        visual_worker_bearer_token,
        visual_worker_context: AuthContext::new(
            studio_id,
            required_nonempty_env("MPS_VISUAL_WORKER_SERVICE_ID")?,
            [VISUAL_WORKER],
        )
        .with_actor_kind(AuditActorKind::System),
        browser_sessions: Default::default(),
        session_cookie_secure: true,
    };
    let state = tokio::task::spawn_blocking(move || AppState::load(&config))
        .await
        .context("join MPS server initialization task")??;
    let browser_sessions = state.browser_session_store();
    let session_cookie_secure = state.session_cookie_secure();
    let auth = StaticTokenAuth {
        browser_sessions,
        session_cookie_secure,
        ..auth
    };
    let application = app(state).layer(middleware::from_fn_with_state(auth, authenticate));
    let bind_address =
        env::var("MPS_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("bind MPS server at {bind_address}"))?;
    axum::serve(listener, application)
        .await
        .context("serve MPS HTTP API")
}

async fn authenticate(
    State(auth): State<StaticTokenAuth>,
    mut request: Request,
    next: Next,
) -> Response {
    let presented = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if request.uri().path() == "/api/session"
        && request.method() == axum::http::Method::POST
    {
        if presented != Some(auth.teacher_bearer_token.as_str()) {
            return unauthorized();
        }
        let credentials = match auth.browser_sessions.issue(auth.teacher_context.clone()) {
            Ok(credentials) => credentials,
            Err(_) => return internal_error(),
        };
        return (
            StatusCode::NO_CONTENT,
            [
                (
                    axum::http::header::SET_COOKIE,
                    session_cookie(&credentials.session_token, auth.session_cookie_secure),
                ),
                (
                    axum::http::header::SET_COOKIE,
                    csrf_cookie(&credentials.csrf_token, auth.session_cookie_secure),
                ),
            ],
        )
            .into_response();
    }

    let session_token = session_token_from_cookie(request.headers());
    let (context, browser_session) = match presented {
        Some(token) if token == auth.teacher_bearer_token.as_str() => (auth.teacher_context, false),
        Some(token) if token == auth.automated_review_bearer_token.as_str() => {
            (auth.automated_review_context, false)
        }
        Some(token) if token == auth.mcp_service_bearer_token.as_str() => {
            match mcp_identity(request.headers()) {
                Some((studio_id, teacher_id)) => (
                    AuthContext::new(
                        studio_id,
                        teacher_id,
                        [READ_CATALOG, WRITE_DRAFT, GENERATE_ASSET, SUBMIT_REVIEW],
                    )
                    .with_actor_kind(AuditActorKind::Chatgpt),
                    false,
                ),
                None => return unauthorized(),
            }
        }
        Some(token) if token == auth.visual_worker_bearer_token.as_str() => {
            (auth.visual_worker_context, false)
        }
        _ => match session_token
            .as_deref()
            .and_then(|token| auth.browser_sessions.authenticate(token))
        {
            Some(context) => (context, true),
            None => return unauthorized(),
        },
    };
    if browser_session
        && matches!(
            request.method(),
            &axum::http::Method::POST
                | &axum::http::Method::PATCH
                | &axum::http::Method::PUT
                | &axum::http::Method::DELETE
        )
        && !browser_csrf_is_valid(
            request.headers(),
            session_token.as_deref().unwrap_or_default(),
            &auth.browser_sessions,
        )
    {
        return csrf_forbidden();
    }
    request.extensions_mut().insert(context);
    next.run(request).await
}

fn mcp_identity(headers: &HeaderMap) -> Option<(String, String)> {
    if headers.get("x-mps-actor-kind")?.to_str().ok()? != "chatgpt" {
        return None;
    }
    let studio_id = headers.get("x-mps-studio-id")?.to_str().ok()?.trim();
    let teacher_id = headers.get("x-mps-teacher-id")?.to_str().ok()?.trim();
    (!studio_id.is_empty() && !teacher_id.is_empty())
        .then(|| (studio_id.to_owned(), teacher_id.to_owned()))
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": {"status": 401, "message": "authenticated MPS session or server-side bearer handoff is required"}})),
    )
        .into_response()
}

fn csrf_forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error": {"message": "missing or invalid MPS CSRF token"}})),
    )
        .into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": {"status": 500, "message": "internal server error"}})),
    )
        .into_response()
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be configured server-side"))
}

fn required_nonempty_env(name: &str) -> Result<String> {
    let value = required_env(name)?;
    if value.trim().is_empty() {
        anyhow::bail!("{name} must not be empty");
    }
    Ok(value)
}
