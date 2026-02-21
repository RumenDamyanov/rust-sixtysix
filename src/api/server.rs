//! Minimal HTTP server exposing the engine via JSON endpoints.

use crate::engine::{Action, Engine, EngineError};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

/// Shared application state.
type AppState = Arc<Engine>;

/// Create an axum Router wired to the given engine.
pub fn create_router(engine: Arc<Engine>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/games", get(list_games))
        .route("/sessions", get(list_sessions).post(create_session))
        .route(
            "/sessions/:id",
            get(get_session).post(apply_action).delete(delete_session),
        )
        .with_state(engine)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn healthz() -> &'static str {
    "ok"
}

async fn list_games(State(engine): State<AppState>) -> impl IntoResponse {
    let games = engine.games();
    Json(serde_json::json!({ "games": games }))
}

#[derive(Deserialize)]
struct CreateSessionParams {
    game: Option<String>,
    seed: Option<i64>,
}

async fn create_session(
    State(engine): State<AppState>,
    Query(params): Query<CreateSessionParams>,
) -> Result<impl IntoResponse, AppError> {
    let game = params.game.ok_or(AppError::bad_request("missing game"))?;
    let seed = params.seed.unwrap_or(0);
    let session = engine.create_session(&game, seed)?;
    Ok((StatusCode::CREATED, Json(session)))
}

#[derive(Deserialize)]
struct ListSessionsParams {
    game: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
}

async fn list_sessions(
    State(engine): State<AppState>,
    Query(params): Query<ListSessionsParams>,
) -> Result<impl IntoResponse, AppError> {
    let game = params.game.unwrap_or_default();
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(0);
    let sessions = engine.list_sessions(&game, offset, limit)?;
    Ok(Json(serde_json::json!({ "sessions": sessions })))
}

async fn get_session(
    State(engine): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let session = engine.get_session(&id)?;
    Ok(Json(session))
}

async fn apply_action(
    State(engine): State<AppState>,
    Path(id): Path<String>,
    Json(action): Json<Action>,
) -> Result<impl IntoResponse, AppError> {
    let session = engine.apply_action(&id, action)?;
    Ok(Json(session))
}

async fn delete_session(
    State(engine): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    engine.delete_session(&id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(msg: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.to_string(),
        }
    }
}

impl From<EngineError> for AppError {
    fn from(err: EngineError) -> Self {
        let status = match &err {
            EngineError::GameNotFound | EngineError::SessionNotFound => StatusCode::NOT_FOUND,
            EngineError::Conflict => StatusCode::CONFLICT,
            EngineError::Validation(_) | EngineError::Store(_) => StatusCode::BAD_REQUEST,
        };
        Self {
            status,
            message: err.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::SixtySix;
    use crate::store::Memory;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let mem = Arc::new(Memory::new());
        let engine = Arc::new(Engine::new(mem));
        engine.register(Arc::new(SixtySix));
        create_router(engine)
    }

    #[tokio::test]
    async fn server_flow() {
        let app = test_app();

        // list games
        let resp = app
            .clone()
            .oneshot(Request::get("/games").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("sixtysix"));

        // create session
        let resp = app
            .clone()
            .oneshot(
                Request::post("/sessions?game=sixtysix")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let sess: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = sess["id"].as_str().unwrap();

        // apply closeStock action
        let resp = app
            .clone()
            .oneshot(
                Request::post(format!("/sessions/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"type":"closeStock"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // get session
        let resp = app
            .clone()
            .oneshot(
                Request::get(format!("/sessions/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("version"));

        // delete session
        let resp = app
            .clone()
            .oneshot(
                Request::delete(format!("/sessions/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }
}
