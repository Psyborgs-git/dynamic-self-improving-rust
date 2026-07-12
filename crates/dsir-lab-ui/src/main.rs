//! Local experiment UI for dsir lab runs.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use dsir::LocalRegistry;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    registry: Arc<Mutex<LocalRegistry>>,
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let runs = state
        .registry
        .lock()
        .ok()
        .and_then(|r| r.list_runs().ok())
        .unwrap_or_default();
    let mut rows = String::new();
    for run in &runs {
        rows.push_str(&format!(
            "<tr>\
                <td>{id}</td>\
                <td>{program}</td>\
                <td>{opt}</td>\
                <td>{train:.3}</td>\
                <td>{val:.3}</td>\
                <td><form method='post' action='/promote/{id}'><button type='submit'>Promote</button></form></td>\
             </tr>",
            id = run.id,
            program = run.program_id,
            opt = run.optimizer,
            train = run.avg_train,
            val = run.avg_val,
        ));
    }
    let promoted = state
        .registry
        .lock()
        .ok()
        .map(|r| r.root().join("promoted/meta.json"))
        .filter(|p| p.exists())
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_else(|| "{\"promoted\":false}".into());

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <title>dsir lab</title>
  <style>
    :root {{ --ink:#1c1917; --paper:#f5f0e8; --accent:#0f766e; --line:#d6d3d1; }}
    body {{ margin:0; font-family: "IBM Plex Sans", "Source Sans 3", sans-serif; background:
      radial-gradient(circle at top left, #ccfbf1 0%, transparent 40%),
      linear-gradient(160deg, #f5f0e8, #e7e5e4); color:var(--ink); }}
    main {{ max-width:960px; margin:2rem auto; padding:0 1rem; }}
    h1 {{ font-family: "Fraunces", "Iowan Old Style", serif; font-weight:600; letter-spacing:-0.02em; }}
    table {{ width:100%; border-collapse:collapse; background:rgba(255,255,255,0.7); }}
    th, td {{ border-bottom:1px solid var(--line); padding:0.6rem 0.5rem; text-align:left; }}
    th {{ font-size:0.85rem; text-transform:uppercase; letter-spacing:0.04em; color:#57534e; }}
    button {{ background:var(--accent); color:white; border:0; padding:0.35rem 0.7rem; cursor:pointer; }}
    pre {{ background:#1c1917; color:#fafaf9; padding:1rem; overflow:auto; }}
  </style>
</head>
<body>
  <main>
    <h1>dsir experiment lab</h1>
    <p>Compare optimizer runs, inspect scores, and promote a program.</p>
    <table>
      <thead><tr><th>Run</th><th>Program</th><th>Optimizer</th><th>Train</th><th>Val</th><th></th></tr></thead>
      <tbody>{rows}</tbody>
    </table>
    <h2>Promoted</h2>
    <pre>{promoted}</pre>
  </main>
</body>
</html>"#
    ))
}

async fn api_runs(State(state): State<AppState>) -> Json<serde_json::Value> {
    let runs = state
        .registry
        .lock()
        .ok()
        .and_then(|r| r.list_runs().ok())
        .unwrap_or_default();
    Json(serde_json::to_value(runs).unwrap_or(serde_json::json!([])))
}

async fn promote(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    if let Ok(registry) = state.registry.lock() {
        if let Ok(run) = registry.get_run(&id) {
            let _ = registry.promote(&run);
        }
    }
    index(State(state)).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let workdir = std::env::var("DSIR_LAB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("dsir-lab-ui"));
    let bind = std::env::var("DSIR_LAB_UI_BIND").unwrap_or_else(|_| "127.0.0.1:8790".into());
    let registry = LocalRegistry::open(&workdir)?;
    let state = AppState {
        registry: Arc::new(Mutex::new(registry)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/runs", get(api_runs))
        .route("/promote/{id}", post(promote))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = bind.parse()?;
    tracing::info!(%addr, workdir = %workdir.display(), "dsir-lab-ui listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
