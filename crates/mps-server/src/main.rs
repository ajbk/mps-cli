use std::env;

use anyhow::{Context, Result};
use axum::extract::{Request, State};
use axum::http::{header::AUTHORIZATION, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Json;
use mps_server::{
    app, AppState, AuthContext, ServerConfig, DEFAULT_CATALOG_EXPORT_PATH,
};
use serde_json::json;

#[derive(Clone)]
struct StaticTokenAuth {
    bearer_token: String,
    context: AuthContext,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig {
        database_path: required_env("MPS_DATABASE_PATH")?,
        catalog_path: env::var("MPS_CATALOG_PATH")
            .unwrap_or_else(|_| DEFAULT_CATALOG_EXPORT_PATH.to_owned()),
        visual_styles_path: env::var("MPS_VISUAL_STYLES_PATH")
            .unwrap_or_else(|_| "data/reference/pilates_visual_styles.json".to_owned()),
        visual_manifest_path: env::var("MPS_VISUAL_MANIFEST_PATH")
            .unwrap_or_else(|_| "data/reference/pilates_visual_manifest.json".to_owned()),
    };
    let auth = StaticTokenAuth {
        bearer_token: required_env("MPS_AUTH_TOKEN")?,
        context: AuthContext::new(
            required_env("MPS_STUDIO_ID")?,
            required_env("MPS_TEACHER_ID")?,
            required_env("MPS_PERMISSIONS")?
                .split(',')
                .map(str::trim)
                .filter(|permission| !permission.is_empty())
                .map(str::to_owned),
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
    if presented != Some(auth.bearer_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"status": 401, "message": "invalid bearer token"}})),
        )
            .into_response();
    }
    request.extensions_mut().insert(auth.context);
    next.run(request).await
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be configured server-side"))
}
