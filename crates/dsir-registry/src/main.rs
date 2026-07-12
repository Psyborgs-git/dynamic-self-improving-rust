//! Remote HTTP run registry for dsir lab artifacts.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use dsir::{GraphState, LocalRegistry, RunRecord};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    registry: Arc<Mutex<LocalRegistry>>,
    token: Option<String>,
}

#[derive(Deserialize)]
struct UploadBody {
    run: RunRecord,
    artifact: GraphState,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn unauthorized() -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorBody {
            error: "unauthorized".into(),
        }),
    )
}

fn check_auth(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let Some(expected) = &state.token else {
        return Ok(());
    };
    let Some(header) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(unauthorized());
    };
    let Ok(value) = header.to_str() else {
        return Err(unauthorized());
    };
    let Some(token) = value.strip_prefix("Bearer ") else {
        return Err(unauthorized());
    };
    if token != expected {
        return Err(unauthorized());
    }
    Ok(())
}

async fn list_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<RunRecord>>, (StatusCode, Json<ErrorBody>)> {
    check_auth(&state, &headers)?;
    let registry = state.registry.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "lock poisoned".into(),
            }),
        )
    })?;
    registry.list_runs().map(Json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })
}

async fn upload_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UploadBody>,
) -> Result<Json<RunRecord>, (StatusCode, Json<ErrorBody>)> {
    check_auth(&state, &headers)?;
    let mut registry = state.registry.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "lock poisoned".into(),
            }),
        )
    })?;
    let run = registry.record_run(body.run).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    registry.save_artifact(&run.id, &body.artifact).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(Json(run))
}

async fn get_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<RunRecord>, (StatusCode, Json<ErrorBody>)> {
    check_auth(&state, &headers)?;
    let registry = state.registry.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "lock poisoned".into(),
            }),
        )
    })?;
    registry.get_run(&id).map(Json).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })
}

async fn get_promoted(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorBody>)> {
    check_auth(&state, &headers)?;
    let path = state
        .registry
        .lock()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "lock poisoned".into(),
                }),
            )
        })?
        .root()
        .join("promoted/meta.json");
    if !path.exists() {
        return Ok(Json(serde_json::json!({ "promoted": false })));
    }
    let bytes = std::fs::read(path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    let value = serde_json::from_slice(&bytes).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(Json(value))
}

async fn promote_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorBody>)> {
    check_auth(&state, &headers)?;
    let registry = state.registry.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "lock poisoned".into(),
            }),
        )
    })?;
    let run = registry.get_run(&id).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    let path = registry.promote(&run).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(Json(serde_json::json!({
        "promoted": true,
        "path": path.display().to_string(),
        "run_id": run.id,
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let workdir = std::env::var("DSIR_REGISTRY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("dsir-registry"));
    let token = std::env::var("DSIR_REGISTRY_TOKEN").ok();
    let bind = std::env::var("DSIR_REGISTRY_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());

    let registry = LocalRegistry::open(&workdir)?;
    let state = AppState {
        registry: Arc::new(Mutex::new(registry)),
        token,
    };

    let app = Router::new()
        .route("/v1/runs", get(list_runs).post(upload_run))
        .route("/v1/runs/{id}", get(get_run))
        .route("/v1/runs/{id}/promote", post(promote_run))
        .route("/v1/promoted", get(get_promoted))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = bind.parse()?;
    tracing::info!(%addr, workdir = %workdir.display(), "dsir-registry listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
