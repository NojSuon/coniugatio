mod conjugate;
mod languages;
mod otomi;

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tower_http::services::ServeDir;

use languages::Catalog;

struct AppState {
    catalog: Catalog,
    static_dir: PathBuf,
}

type Shared = Arc<AppState>;

#[tokio::main]
async fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let catalog = Catalog::load(&root.join("data/languages")).expect("could not load language data");
    let static_dir = root.join("static");

    let state = Arc::new(AppState { catalog, static_dir: static_dir.clone() });

    let app = Router::new()
        .route("/api/languages", get(list_languages))
        .route("/api/languages/{slug}", get(language_detail))
        .route("/api/languages/{slug}/conjugate", get(conjugate_verb))
        .route("/", get(home_page))
        .route("/{slug}", get(language_page))
        .nest_service("/static", ServeDir::new(static_dir))
        .with_state(state);

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Mati Mati coniugatio → http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("could not bind port");
    axum::serve(listener, app).await.expect("server error");
}

fn not_found(message: &str) -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": message }))).into_response()
}

async fn list_languages(State(s): State<Shared>) -> impl IntoResponse {
    Json(s.catalog.summaries())
}

async fn language_detail(State(s): State<Shared>, Path(slug): Path<String>) -> Response {
    match s.catalog.get(&slug) {
        Some(lang) => Json(lang.raw.clone()).into_response(),
        None => not_found("Esa lengua todavía no está en Mati Mati."),
    }
}

#[derive(Deserialize)]
struct ConjugateQuery {
    verb: String,
}

async fn conjugate_verb(
    State(s): State<Shared>,
    Path(slug): Path<String>,
    Query(q): Query<ConjugateQuery>,
) -> Response {
    let Some(lang) = s.catalog.get(&slug) else {
        return not_found("Esa lengua todavía no está en Mati Mati.");
    };
    let Some(conj) = &lang.conjugation else {
        return not_found("El conjugador de esta lengua está en preparación.");
    };
    match conj.conjugate(&q.verb) {
        Ok(result) => Json(result).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn serve_html(s: &AppState, file: &str) -> Response {
    match tokio::fs::read_to_string(s.static_dir.join(file)).await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "missing page").into_response(),
    }
}

async fn home_page(State(s): State<Shared>) -> Response {
    serve_html(&s, "index.html").await
}

/// `/otomi`, `/itza`, ... all share one page; the script reads the slug from
/// the URL and fetches that language's data.
async fn language_page(State(s): State<Shared>, Path(slug): Path<String>) -> Response {
    if s.catalog.get(&slug).is_some() {
        serve_html(&s, "language.html").await
    } else {
        (StatusCode::NOT_FOUND, serve_html(&s, "404.html").await).into_response()
    }
}
