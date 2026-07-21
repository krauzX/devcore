use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use devcore_academic::SemesterStore;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    store: Arc<SemesterStore>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let project_root = std::env::var("DEVCORE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let store = SemesterStore::open(&project_root).expect("Failed to open academic store");
    let state = AppState {
        store: Arc::new(store),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/semesters", get(list_semesters))
        .route("/api/semesters/current", get(current_semester))
        .route("/api/semester/:id/courses", get(courses_for_semester))
        .route("/api/semester/:id/sgpa", get(sgpa_for_semester))
        .route("/api/papers", get(list_papers))
        .route("/api/papers/stats", get(paper_stats))
        .route("/api/upcoming", get(upcoming_events))
        .route("/api/dashboard", get(dashboard))
        .route("/api/activity", get(activity))
        .route("/api/system", get(system_info))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3847".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("DevCore API running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": "0.1.0" }))
}

async fn list_semesters(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let semesters = state
        .store
        .list_semesters()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(semesters)))
}

async fn current_semester(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let sem = state
        .store
        .current_semester()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(sem)))
}

async fn courses_for_semester(
    State(state): State<AppState>,
    Path(semester_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let courses = devcore_academic::Course::list_for_semester(&conn, &semester_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(courses)))
}

async fn sgpa_for_semester(
    State(state): State<AppState>,
    Path(semester_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sgpa = devcore_academic::GradeEntry::compute_sgpa(&conn, &semester_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "semester": semester_id, "sgpa": sgpa })))
}

async fn list_papers(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let papers = devcore_academic::Paper::list(&conn, None)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(papers)))
}

async fn paper_stats(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stats =
        devcore_academic::Paper::stats(&conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(stats)))
}

async fn upcoming_events(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let events = devcore_academic::AcademicEvent::this_week(&conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(events)))
}

async fn dashboard(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let sem = state
        .store
        .current_semester()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let courses = devcore_academic::Course::list_for_semester(
        &conn,
        sem.as_ref().map(|s| s.id.as_str()).unwrap_or(""),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let paper_stats =
        devcore_academic::Paper::stats(&conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let upcoming = devcore_academic::AcademicEvent::this_week(&conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sgpa = devcore_academic::GradeEntry::compute_sgpa(
        &conn,
        sem.as_ref().map(|s| s.id.as_str()).unwrap_or(""),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "semester": sem,
        "courses": courses,
        "paper_stats": paper_stats,
        "upcoming_events": upcoming,
        "sgpa": sgpa,
    })))
}

async fn activity(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let conn = state
        .store
        .conn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let events = devcore_academic::AcademicEvent::this_week(&conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_events = events.len();
    let mut categories: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut recent_events = Vec::new();

    for event in &events {
        let cat = format!("{}", event.event_type);
        *categories.entry(cat).or_insert(0) += 1;
        recent_events.push(json!({
            "id": event.id,
            "title": event.title,
            "type": format!("{}", event.event_type),
            "date": event.date.to_string(),
            "time": event.time,
        }));
    }

    let total_minutes: u32 = categories.values().sum::<u32>() * 30;

    Ok(Json(json!({
        "total_events": total_events,
        "total_minutes": total_minutes,
        "categories": categories,
        "recent_events": recent_events,
    })))
}

async fn system_info() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "crate_count": 7,
        "test_count": 31,
        "languages": ["rust", "typescript"],
    }))
}
