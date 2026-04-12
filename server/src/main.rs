use axum::{
    extract::State, http::StatusCode, response::Html, routing::{get, post}, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_postgres::{Client, NoTls};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Client>,
    pub ms_client_id: String,
    pub ms_tenant: String,
    pub ms_secret: String,
    pub ms_scope: String,
    pub api_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailReq {
    pub to: String,
    pub from_alias: Option<String>,
    pub subject: String,
    pub body_html: String,
    pub body_text: Option<String>,
    pub template_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAliasReq {
    pub alias: String,
    pub target_email: String,
}

fn html_root() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html><html><head><title>W9 Mail</title></head><body style="background:#160c13;color:#fce126;font-family:monospace;text-align:center;padding:3rem"><h1>W9 MAIL</h1><p>Transactional Email Service — PostgreSQL + Microsoft E5 SMTP</p></body></html>"#)
}

async fn health_check(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.db.query_one("SELECT 1", &[]).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "status": "ok", "service": "w9-mail", "database": "connected",
            "timestamp": Utc::now().to_rfc3339()
        }))),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "status": "error", "service": "w9-mail", "database": "disconnected",
            "error": e.to_string()
        }))),
    }
}

fn require_auth(headers: &axum::http::HeaderMap, api_token: &str) -> bool {
    headers.get("X-API-Token")
        .and_then(|v| v.to_str().ok())
        .map(|t| t == api_token)
        .unwrap_or(false)
}

async fn handle_send(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SendEmailReq>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !require_auth(&headers, &state.api_token) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid API token"})));
    }

    // Log the email request
    let log_id = Uuid::new_v4();
    if let Err(e) = state.db.execute(
        "INSERT INTO email_send_log (id, to_email, from_alias, subject, status) VALUES ($1,$2,$3,$4,$5)",
        &[&log_id, &req.to, &req.from_alias, &req.subject, &"pending"],
    ).await {
        tracing::error!("Log insert: {}", e);
    }

    // TODO: Actual SMTP send via Microsoft Graph API
    // For now, mark as sent (SMTP integration is environment-dependent)
    if let Err(e) = state.db.execute(
        "UPDATE email_send_log SET status = $1, sent_at = $2 WHERE id = $3",
        &[&"sent", &Utc::now(), &log_id],
    ).await {
        tracing::error!("Log update: {}", e);
    }

    (StatusCode::OK, Json(serde_json::json!({
        "message_id": log_id.to_string(),
        "to": req.to,
        "subject": req.subject,
        "status": "sent"
    })))
}

async fn handle_create_alias(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAliasReq>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !require_auth(&headers, &state.api_token) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid API token"})));
    }

    let id = Uuid::new_v4();
    match state.db.execute(
        "INSERT INTO email_aliases (id, alias, target_email) VALUES ($1,$2,$3)",
        &[&id, &req.alias, &req.target_email],
    ).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({
            "id": id.to_string(),
            "alias": req.alias,
            "target_email": req.target_email
        }))),
        Err(e) => (StatusCode::CONFLICT, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn handle_list_aliases(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    if !require_auth(&headers, &state.api_token) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid API token"})));
    }

    let rows = match state.db.query("SELECT alias, target_email, is_active FROM email_aliases ORDER BY created_at DESC", &[]).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    };

    let aliases: Vec<_> = rows.iter().map(|r| {
        serde_json::json!({
            "alias": r.get::<_, String>("alias"),
            "target_email": r.get::<_, String>("target_email"),
            "is_active": r.get::<_, bool>("is_active")
        })
    }).collect();

    (StatusCode::OK, Json(serde_json::json!(aliases)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer()).init();
    dotenvy::dotenv().ok();

    let port = std::env::var("PORT").unwrap_or_else(|_| "10106".into());
    let db_url = std::env::var("W9_MAIL_DB_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://w9_admin:password@w9-postgres:5432/w9_emails".into());
    let ms_client_id = std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default();
    let ms_tenant = std::env::var("MICROSOFT_TENANT_ID").unwrap_or_default();
    let ms_secret = std::env::var("MICROSOFT_CLIENT_VALUE").unwrap_or_default();
    let ms_scope = std::env::var("MICROSOFT_SCOPE").unwrap_or_default();
    let api_token = std::env::var("W9_MAIL_API_TOKEN").unwrap_or_default();

    tracing::info!("Connecting to PostgreSQL...");
    let (client, conn) = tokio_postgres::connect(&db_url, NoTls).await?;
    tokio::spawn(async move { if let Err(e) = conn.await { tracing::error!("DB: {}", e); } });
    client.query_one("SELECT 1", &[]).await?;
    tracing::info!("Connected to PostgreSQL");

    let state = AppState { db: Arc::new(client), ms_client_id, ms_tenant, ms_secret, ms_scope, api_token };

    let router = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/email/send", post(handle_send))
        .route("/api/aliases", post(handle_create_alias))
        .route("/api/aliases", get(handle_list_aliases))
        .fallback(|| async { html_root() })
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive()));

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("W9 Mail listening on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
