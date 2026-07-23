use std::env;

use anyhow::{Context, Result};
use axum::extract::{Request, State};
use axum::http::{header::AUTHORIZATION, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Json;
use mps_server::{
    app, AppState, AuthContext, ServerConfig, AUTOMATED_REVIEW, DEFAULT_CATALOG_EXPORT_PATH,
};
use serde_json::json;

#[derive(Clone)]
struct StaticTokenAuth {
    teacher_bearer_token: String,
    teacher_context: AuthContext,
    automated_review_bearer_token: String,
    automated_review_context: AuthContext,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig {
        database_path: required_env("MPS_DATABASE_PATH")?,
        catalog_path: env::var("MPS_CATALOG_PATH")
            .unwrap_or_else(|_| DEFAULT_CATALOG_EXPORT_PATH.to_owned()),
        visual_styles_path: required_env("MPS_VISUAL_STYLES_PATH")?,
        visual_manifest_path: required_env("MPS_VISUAL_MANIFEST_PATH")?,
    };
    let teacher_bearer_token = required_env("MPS_AUTH_TOKEN")?;
    let teacher_permissions = required_env("MPS_PERMISSIONS")?
        .split(',')
        .map(str::trim)
        .filter(|permission| !permission.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if teacher_permissions
        .iter()
        .any(|permission| permission.as_str() == AUTOMATED_REVIEW)
    {
        anyhow::bail!(
            "MPS_PERMISSIONS must not grant the service-only {AUTOMATED_REVIEW} scope"
        );
    }
    let automated_review_bearer_token = required_env("MPS_AUTOMATED_REVIEW_TOKEN")?;
    if teacher_bearer_token == automated_review_bearer_token {
        anyhow::bail!("MPS_AUTH_TOKEN and MPS_AUTOMATED_REVIEW_TOKEN must differ");
    }
    let auth = StaticTokenAuth {
        teacher_bearer_token,
        teacher_context: AuthContext::new(
            required_env("MPS_STUDIO_ID")?,
            required_env("MPS_TEACHER_ID")?,
            teacher_permissions,
        ),
        automated_review_bearer_token,
        automated_review_context: AuthContext::new(
            required_env("MPS_STUDIO_ID")?,
            required_env("MPS_AUTOMATED_REVIEW_SERVICE_ID")?,
            [AUTOMATED_REVIEW],
        ),
    };
    let state = tokio::task::spawn_blocking(move || AppState::load(&config))
        .await
        .context("join MPS server initialization task")??;
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
    let context = match presented {
        Some(token) if token == auth.teacher_bearer_token.as_str() => auth.teacher_context,
        Some(token) if token == auth.automated_review_bearer_token.as_str() => {
            auth.automated_review_context
        }
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"status": 401, "message": "invalid bearer token"}})),
            )
                .into_response()
        }
    };
    request.extensions_mut().insert(context);
    next.run(request).await
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be configured server-side"))
}
