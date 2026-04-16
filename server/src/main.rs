use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_postgres::{Client, NoTls};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

const CSS: &str = include_str!("../infra/templates/voxel.css");
const W9_DB_URL: &str = "https://db.w9.nu";

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Client>,
    pub api_token: String,
    pub http_client: reqwest::Client,
}

/// Send email via SMTP (Office 365 / Microsoft E5)
fn send_email_via_smtp(
    username: &str,
    password: &str,
    from: &str,
    to: &str,
    subject: &str,
    body_html: &str,
    _body_text: &str,
) -> Result<String, String> {
    let email = Message::builder()
        .from(
            from.parse()
                .map_err(|e| format!("Invalid from address: {}", e))?,
        )
        .to(to
            .parse()
            .map_err(|e| format!("Invalid to address: {}", e))?)
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body_html.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(username.to_string(), password.to_string());

    let mailer = SmtpTransport::starttls_relay("smtp.office365.com")
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map(|_| "sent".to_string())
        .map_err(|e| format!("SMTP send failed: {}", e))
}

/// Look up E5 user credentials by sender email or alias
async fn lookup_e5_credentials(state: &AppState, from: &str) -> Result<(String, String), String> {
    match state.db.query_opt(
        "SELECT email, app_password FROM e5_users WHERE email = $1",
        &[&from],
    ).await {
        Ok(Some(row)) => Ok((row.get::<_, String>(0), row.get::<_, String>(1))),
        Ok(None) => {
            match state.db.query_opt(
                "SELECT eu.email, eu.app_password FROM email_aliases ea JOIN e5_users eu ON ea.user_email = eu.email WHERE ea.alias = $1",
                &[&from],
            ).await {
                Ok(Some(row)) => Ok((row.get::<_, String>(0), row.get::<_, String>(1))),
                Ok(None) => Err(format!("No E5 user found for sender: {}", from)),
                Err(e) => Err(format!("DB error looking up E5 user: {}", e)),
            }
        }
        Err(e) => Err(format!("DB error looking up E5 user: {}", e)),
    }
}

// ============================================================
// Layout
// ============================================================
fn layout(title: &str, body: &str, nav: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><link rel="icon" type="image/svg+xml" href="/w9-logo/favicon.svg"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>{title} — W9 Mail</title><style>{CSS}</style></head><body><div class="app"><nav class="nav"><div class="nav-inner"><a href="/" class="brand"><img src="/w9-logo/workmark-transparent.svg" alt="W9 Labs"/><span class="brand-text">Mail</span></a><div class="nav-links">{nav}</div></div></nav><main class="app-main">{body}</main><footer class="footer"><img class="footer-logo" src="/w9-logo/workmark-transparent.svg" alt="W9 Labs"/><p>W9 Mail — Transactional Email Service</p><p class="text-xs text-muted">Microsoft E5 SMTP + Admin Panel</p></footer></div></body></html>"#,
        title = title,
        CSS = CSS,
        nav = nav,
        body = body
    )
}
fn public_layout(title: &str, body: &str) -> String {
    layout(
        title,
        body,
        r#"<a href="/login" onclick="const w=window.open(this.href,'w9-mail-login','width=520,height=720'); if (w) { w.focus(); return false; }">Admin Login</a>"#,
    )
}
fn admin_layout(title: &str, body: &str) -> String {
    layout(
        title,
        body,
        r#"<a href="/dashboard">Dashboard</a><a href="/tokens">API Tokens</a><a href="/users">E5 Users</a><a href="/aliases">Aliases</a><a href="/log">Send Log</a><a href="/logout">Logout</a>"#,
    )
}

// ============================================================
// OAuth Session (from w9-db)
// ============================================================
fn set_mail_session(jar: CookieJar, token: String) -> CookieJar {
    jar.add(
        axum_extra::extract::cookie::Cookie::build(("w9_mail_session", token))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(time::Duration::days(7))
            .finish(),
    )
}
fn clear_mail_session(jar: CookieJar) -> CookieJar {
    jar.remove(axum_extra::extract::cookie::Cookie::named(
        "w9_mail_session",
    ))
}
fn get_mail_session(jar: &CookieJar) -> Option<String> {
    jar.get("w9_mail_session").map(|c| c.value().to_string())
}

fn popup_close_html(target: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>W9 Mail Login</title></head><body><script>(function(){{const target = {target:?}; if (window.opener && !window.opener.closed) {{ try {{ window.opener.location.href = target; window.opener.focus(); }} catch (_) {{}} window.close(); }} else {{ window.location.replace(target); }}}})();</script><p>Signing you in…</p></body></html>"#
    )
}

async fn verify_w9_session(state: &AppState, token: &str) -> Option<serde_json::Value> {
    let res = state
        .http_client
        .get(format!("{}/api/auth/me", W9_DB_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?;
    if res.status().is_success() {
        res.json().await.ok()
    } else {
        None
    }
}

// ============================================================
// Pages: Public
// ============================================================
fn home_html() -> String {
    public_layout(
        "W9 Mail",
        r#"<div class="hero"><img class="hero-logo" src="/w9-logo/logo-landscape-transparent.svg" alt="W9 Labs"/><h1>W9 Mail</h1><p class="hero-sub">Transactional Email Service powered by Microsoft E5 SMTP</p><p class="hero-muted">Send emails, manage aliases, and track delivery for the W9 Network</p><div class="hero-actions"><a href="https://db.w9.nu/oauth/authorize?redirect_uri=https://mail.w9.nu/oauth/callback&response_type=code&client_id=w9-mail" class="btn" onclick="const w=window.open(this.href,'w9-mail-login','width=520,height=720'); if (w) { w.focus(); return false; }">Admin Login</a></div></div><div class="grid"><div class="card"><h3>📤 Send Emails</h3><p>API endpoint for other W9 services to send transactional emails via Office 365 SMTP (port 587 STARTTLS).</p></div><div class="card"><h3>🏷️ Alias Management</h3><p>Create and manage email aliases per E5 user for different service identities.</p></div><div class="card"><h3>🔑 API Tokens</h3><p>Generate API tokens for w9-db, w9-reminders, and other services to send emails.</p></div></div>"#,
    )
}

fn login_html(err: Option<&str>) -> String {
    let alert = err
        .map(|e| format!(r#"<div class="alert alert--err">{}</div>"#, e))
        .unwrap_or_default();
    public_layout(
        "Admin Login",
        &format!(
            r#"<div class="card" style="max-width:420px;margin:3rem auto"><h1>🔐 Admin Login</h1>{}<p class="text-sm text-muted mb-2">Sign in with your W9 DB account (admin role required)</p><a href="{}/oauth/authorize?redirect_uri={}/oauth/callback&response_type=code&client_id=w9-mail" class="btn" style="width:100%;text-align:center" onclick="const w=window.open(this.href,'w9-mail-login','width=520,height=720'); if (w) {{ w.focus(); return false; }}">Login with W9 DB</a></div>"#,
            alert,
            W9_DB_URL,
            std::env::var("PUBLIC_URL").unwrap_or_else(|_| "https://mail.w9.nu".into())
        ),
    )
}

// ============================================================
// Pages: Admin
// ============================================================
fn dashboard_html(user: &serde_json::Value, stats: &[(String, String, String)]) -> String {
    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("admin");
    let rows: String = stats.iter().map(|(date, sent, failed)| {
        format!(r#"<tr><td>{}</td><td><span class="badge badge--ok">{}</span></td><td><span class="badge badge--err">{}</span></td></tr>"#, date, sent, failed)
    }).collect();
    admin_layout(
        "Dashboard",
        &format!(
            r#"<div class="hero"><h1>📧 Welcome, {}</h1><p class="text-sm">Email Service Dashboard</p></div><div class="card"><h2>Recent Send Stats</h2><table><tr><th>Date</th><th>Sent</th><th>Failed</th></tr>{}</table></div>"#,
            email, rows
        ),
    )
}

fn tokens_html(tokens: &[(String, String, String, String)], msg: Option<&str>) -> String {
    let alert = msg
        .map(|m| format!(r#"<div class="alert alert--ok">{}</div>"#, m))
        .unwrap_or_default();
    let rows: String = tokens.iter().map(|(id, name, prefix, created)| {
        format!(r#"<tr><td>{}</td><td>{}</td><td class="text-xs">{}</td><td><a href="/tokens/revoke/{}" class="btn btn--sm">Revoke</a></td></tr>"#, name, prefix, created, id)
    }).collect();
    admin_layout(
        "API Tokens",
        &format!(
            r#"<div class="card" style="max-width:700px;margin:2rem auto"><h1>🔑 API Tokens</h1>{}<form method="POST" action="/tokens"><label>Token Name</label><input type="text" name="name" required placeholder="w9-db"/><button type="submit" class="btn mt-1" style="width:100%">Generate Token</button></form><h2 class="mt-3">Active Tokens</h2><table><tr><th>Name</th><th>Token (prefix)</th><th>Created</th><th>Action</th></tr>{}</table></div>"#,
            alert, rows
        ),
    )
}

fn users_html(users: &[(String, String, String)], msg: Option<&str>) -> String {
    let alert = msg
        .map(|m| format!(r#"<div class="alert alert--ok">{}</div>"#, m))
        .unwrap_or_default();
    let rows: String = users.iter().map(|(email, aliases_count, added)| {
        format!(r#"<tr><td>{}</td><td>{}</td><td class="text-xs">{}</td><td><a href="/users/remove/{}" class="btn btn--sm btn--ghost">Remove</a></td></tr>"#, email, aliases_count, added, email)
    }).collect();
    admin_layout(
        "E5 Users",
        &format!(
            r#"<div class="card" style="max-width:700px;margin:2rem auto"><h1>👥 E5 User Accounts</h1>{}<form method="POST" action="/users"><label>Email (Microsoft E5)</label><input type="email" name="email" required placeholder="user@w9.nu"/><label>App Password</label><input type="password" name="app_password" required placeholder="xxxx-xxxx-xxxx-xxxx"/><button type="submit" class="btn mt-1" style="width:100%">Add User</button></form><h2 class="mt-3">Configured Users</h2><table><tr><th>Email</th><th>Aliases</th><th>Added</th><th>Action</th></tr>{}</table></div>"#,
            alert, rows
        ),
    )
}

fn aliases_html(aliases: &[(String, String, String)], msg: Option<&str>) -> String {
    let alert = msg
        .map(|m| format!(r#"<div class="alert alert--ok">{}</div>"#, m))
        .unwrap_or_default();
    let rows: String = aliases
        .iter()
        .map(|(alias, user_email, created)| {
            format!(
                r#"<tr><td>{}</td><td>{}</td><td class="text-xs">{}</td></tr>"#,
                alias, user_email, created
            )
        })
        .collect();
    admin_layout(
        "Aliases",
        &format!(
            r#"<div class="card" style="max-width:700px;margin:2rem auto"><h1>🏷️ Email Aliases</h1>{}<form method="POST" action="/aliases"><label>Alias Email</label><input type="email" name="alias" required placeholder="noreply@w9.nu"/><label>Target User</label><input type="email" name="user_email" required placeholder="admin@w9.nu"/><button type="submit" class="btn mt-1" style="width:100%">Add Alias</button></form><h2 class="mt-3">Active Aliases</h2><table><tr><th>Alias</th><th>Target User</th><th>Created</th></tr>{}</table></div>"#,
            alert, rows
        ),
    )
}

fn log_html(logs: &[(String, String, String, String, String)]) -> String {
    let rows: String = logs.iter().map(|(to, subject, status, sent, created)| {
        let badge = match status.as_str() { "sent" => r#"<span class="badge badge--ok">Sent</span>"#, "failed" => r#"<span class="badge badge--err">Failed</span>"#, _ => r#"<span class="badge badge--warn">Pending</span>"# };
        format!(r#"<tr><td>{}</td><td class="text-sm">{}</td><td>{}</td><td class="text-xs">{}</td><td class="text-xs">{}</td></tr>"#, to, subject, badge, sent, created)
    }).collect();
    admin_layout(
        "Send Log",
        &format!(
            r#"<div class="card" style="max-width:900px;margin:2rem auto"><h1>📊 Send Log</h1><table><tr><th>To</th><th>Subject</th><th>Status</th><th>Sent At</th><th>Created</th></tr>{}</table></div>"#,
            rows
        ),
    )
}

// ============================================================
// Form Structs
// ============================================================
#[derive(Debug, Deserialize)]
struct GenerateTokenReq {
    name: String,
}
#[derive(Debug, Deserialize)]
struct AddUserReq {
    email: String,
    app_password: String,
}
#[derive(Debug, Deserialize)]
struct AddAliasReq {
    alias: String,
    user_email: String,
}
#[derive(Debug, Deserialize)]
struct SendEmailApiReq {
    to: String,
    from_alias: Option<String>,
    subject: String,
    body_html: String,
    body_text: Option<String>,
}

// ============================================================
// Handlers: Public
// ============================================================
async fn home() -> Html<String> {
    Html(home_html())
}
async fn login_page(jar: CookieJar) -> impl IntoResponse {
    if get_mail_session(&jar).is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    Html(login_html(None)).into_response()
}

// OAuth callback from w9-db
async fn oauth_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(q): Query<serde_json::Value>,
) -> impl IntoResponse {
    let code = match q.get("code").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            return Html(login_html(Some("OAuth: No authorization code received"))).into_response()
        }
    };
    let res = state
        .http_client
        .post(format!("{}/oauth/token", W9_DB_URL))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            (
                "redirect_uri",
                &format!(
                    "{}/oauth/callback",
                    std::env::var("PUBLIC_URL").unwrap_or_else(|_| "https://mail.w9.nu".into())
                ),
            ),
        ])
        .send()
        .await;
    let token_data = match res {
        Ok(r) => match r.json::<serde_json::Value>().await {
            Ok(v) => v,
            Err(_) => {
                return Html(login_html(Some("OAuth: Failed to parse token response")))
                    .into_response()
            }
        },
        Err(_) => return Html(login_html(Some("OAuth: Failed to exchange code"))).into_response(),
    };
    let access_token = match token_data.get("access_token").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return Html(login_html(Some("OAuth: No access token in response"))).into_response()
        }
    };
    let user = match verify_w9_session(&state, &access_token).await {
        Some(u) => u,
        None => return Html(login_html(Some("OAuth: Failed to verify session"))).into_response(),
    };
    let role = user.get("role").and_then(|v| v.as_str()).unwrap_or("");
    if role != "admin" {
        return Html(login_html(Some("Access denied: Admin role required"))).into_response();
    }
    let jar = set_mail_session(jar, access_token);
    (jar, Html(popup_close_html("/dashboard"))).into_response()
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (clear_mail_session(jar), Redirect::to("/")).into_response()
}

// ============================================================
// Handlers: Admin (require auth)
// ============================================================
async fn require_admin(jar: &CookieJar, state: &AppState) -> Option<serde_json::Value> {
    let token = get_mail_session(jar)?;
    let user = verify_w9_session(state, &token).await?;
    if user.get("role").and_then(|v| v.as_str()) != Some("admin") {
        return None;
    }
    Some(user)
}

async fn dashboard(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let stats = match state.db.query("SELECT TO_CHAR(created_at, 'YYYY-MM-DD') as day, COUNT(*) FILTER (WHERE status='sent') as sent, COUNT(*) FILTER (WHERE status='failed') as failed FROM email_send_log GROUP BY day ORDER BY day DESC LIMIT 7", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get::<_,String>(0), r.get::<_,i64>(1).to_string(), r.get::<_,i64>(2).to_string())).collect(),
        Err(_) => Vec::new(),
    };
    Html(dashboard_html(&user, &stats)).into_response()
}

async fn tokens_page(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let tokens = match state.db.query("SELECT id::text, name, LEFT(token, 8) as prefix, created_at::text FROM mail_api_tokens ORDER BY created_at DESC", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get(1), r.get(2), r.get(3))).collect(),
        Err(_) => Vec::new(),
    };
    Html(tokens_html(&tokens, None)).into_response()
}

async fn tokens_create(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<GenerateTokenReq>,
) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let token = format!(
        "mail-{}-{}-{}",
        form.name,
        Uuid::new_v4(),
        Utc::now().timestamp()
    );
    let id = Uuid::new_v4();
    let _ = state
        .db
        .execute(
            "INSERT INTO mail_api_tokens (id, name, token) VALUES ($1,$2,$3)",
            &[&id, &form.name, &token],
        )
        .await;
    let tokens = match state.db.query("SELECT id::text, name, LEFT(token, 8) as prefix, created_at::text FROM mail_api_tokens ORDER BY created_at DESC", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get(1), r.get(2), r.get(3))).collect(),
        Err(_) => Vec::new(),
    };
    Html(tokens_html(
        &tokens,
        Some(&format!("✅ Token generated: {} (save it now!)", token)),
    ))
    .into_response()
}

async fn tokens_revoke(
    State(state): State<AppState>,
    jar: CookieJar,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let _ = state
        .db
        .execute("DELETE FROM mail_api_tokens WHERE id::text = $1", &[&id])
        .await;
    Redirect::to("/tokens").into_response()
}

async fn users_page(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let users = match state.db.query("SELECT email, (SELECT COUNT(*) FROM email_aliases ea WHERE ea.user_email = eu.email) as aliases, added_at::text FROM e5_users eu ORDER BY added_at DESC", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get::<_,i64>(1).to_string(), r.get(2))).collect(),
        Err(_) => Vec::new(),
    };
    Html(users_html(&users, None)).into_response()
}

async fn users_create(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<AddUserReq>,
) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let _ = state.db.execute("INSERT INTO e5_users (email, app_password) VALUES ($1,$2) ON CONFLICT (email) DO UPDATE SET app_password = $2", &[&form.email, &form.app_password]).await;
    let users = match state.db.query("SELECT email, (SELECT COUNT(*) FROM email_aliases ea WHERE ea.user_email = eu.email) as aliases, added_at::text FROM e5_users eu ORDER BY added_at DESC", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get::<_,i64>(1).to_string(), r.get(2))).collect(),
        Err(_) => Vec::new(),
    };
    Html(users_html(
        &users,
        Some(&format!("✅ User {} added", form.email)),
    ))
    .into_response()
}

async fn aliases_page(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let aliases = match state.db.query("SELECT alias, user_email, created_at::text FROM email_aliases ORDER BY created_at DESC", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get(1), r.get(2))).collect(),
        Err(_) => Vec::new(),
    };
    Html(aliases_html(&aliases, None)).into_response()
}

async fn aliases_create(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<AddAliasReq>,
) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let id = Uuid::new_v4();
    let _ = state
        .db
        .execute(
            "INSERT INTO email_aliases (id, alias, user_email) VALUES ($1,$2,$3)",
            &[&id, &form.alias, &form.user_email],
        )
        .await;
    let aliases = match state.db.query("SELECT alias, user_email, created_at::text FROM email_aliases ORDER BY created_at DESC", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get(1), r.get(2))).collect(),
        Err(_) => Vec::new(),
    };
    Html(aliases_html(
        &aliases,
        Some(&format!("✅ Alias {} created", form.alias)),
    ))
    .into_response()
}

async fn log_page(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let _user = match require_admin(&jar, &state).await {
        Some(u) => u,
        None => return (clear_mail_session(jar), Redirect::to("/login")).into_response(),
    };
    let logs = match state.db.query("SELECT to_email, subject, status, COALESCE(sent_at::text, '—') as sent_at, created_at::text FROM email_send_log ORDER BY created_at DESC LIMIT 50", &[]).await {
        Ok(rows) => rows.iter().map(|r| (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4))).collect(),
        Err(_) => Vec::new(),
    };
    Html(log_html(&logs)).into_response()
}

// ============================================================
// API: Send Email (for other projects)
// ============================================================
async fn api_send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SendEmailApiReq>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Verify API token
    let api_token = match headers.get("X-API-Token").and_then(|v| v.to_str().ok()) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error":"Missing X-API-Token header"})),
            )
        }
    };
    let valid = match state
        .db
        .query_one(
            "SELECT COUNT(*) FROM mail_api_tokens WHERE token = $1",
            &[&api_token],
        )
        .await
    {
        Ok(r) => {
            let c: i64 = r.get(0);
            c > 0
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error":"Database error"})),
            )
        }
    };
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"Invalid API token"})),
        );
    }

    // Determine sender: use alias if specified, otherwise find first E5 user
    let from: String = match &req.from_alias {
        Some(alias) => {
            match state
                .db
                .query_one(
                    "SELECT alias FROM email_aliases WHERE alias = $1",
                    &[&alias],
                )
                .await
            {
                Ok(row) => row.get(0),
                Err(_) => {
                    tracing::warn!("Alias '{}' not found, using first E5 user", alias);
                    match state
                        .db
                        .query_opt("SELECT email FROM e5_users LIMIT 1", &[])
                        .await
                    {
                        Ok(Some(row)) => row.get::<_, String>(0),
                        _ => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({"error":"No E5 user configured"})),
                            )
                        }
                    }
                }
            }
        }
        None => {
            match state
                .db
                .query_opt("SELECT email FROM e5_users LIMIT 1", &[])
                .await
            {
                Ok(Some(row)) => row.get::<_, String>(0),
                _ => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error":"No E5 user configured"})),
                    )
                }
            }
        }
    };

    // Look up E5 credentials for SMTP auth
    let (smtp_username, smtp_password) = match lookup_e5_credentials(&state, &from).await {
        Ok(creds) => creds,
        Err(e) => {
            tracing::error!("❌ Failed to lookup E5 credentials for {}: {}", from, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("No SMTP credentials for {}", from)})),
            );
        }
    };

    // Log the email
    let log_id = Uuid::new_v4();
    let _ = state.db.execute("INSERT INTO email_send_log (id, to_email, from_alias, subject, status) VALUES ($1,$2,$3,$4,$5)",
        &[&log_id, &req.to, &req.from_alias, &req.subject, &"pending"]).await;

    let body_text = req
        .body_text
        .clone()
        .unwrap_or_else(|| req.body_html.clone());

    // Send via SMTP (Office 365)
    match send_email_via_smtp(
        &smtp_username,
        &smtp_password,
        &from,
        &req.to,
        &req.subject,
        &req.body_html,
        &body_text,
    ) {
        Ok(_) => {
            tracing::info!(
                "✅ Email sent via SMTP: {} → {} ({})",
                from,
                req.to,
                req.subject
            );
            let _ = state
                .db
                .execute(
                    "UPDATE email_send_log SET status = $1, sent_at = $2 WHERE id = $3",
                    &[&"sent", &Utc::now(), &log_id],
                )
                .await;
            (
                StatusCode::OK,
                Json(
                    serde_json::json!({"message_id": log_id.to_string(), "to": req.to, "subject": req.subject, "status": "sent"}),
                ),
            )
        }
        Err(e) => {
            tracing::error!(
                "❌ SMTP send failed: {} → {} ({}) — {}",
                from,
                req.to,
                req.subject,
                e
            );
            let _ = state.db.execute("UPDATE email_send_log SET status = $1, error_message = $2, sent_at = $3 WHERE id = $4",
                &[&"failed", &e, &Utc::now(), &log_id]).await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"error": e, "to": req.to, "subject": req.subject, "status": "failed"}),
                ),
            )
        }
    }
}

async fn api_health(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.query_one("SELECT 1", &[]).await {
        Ok(_) => (
            StatusCode::OK,
            Json(
                serde_json::json!({"status":"ok","service":"w9-mail","database":"connected","timestamp":Utc::now().to_rfc3339()}),
            ),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status":"error","error":e.to_string()})),
        ),
    }
}

// ============================================================
// Main
// ============================================================
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    dotenvy::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "10106".into());
    let db_url = std::env::var("W9_MAIL_DB_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://w9_admin:password@w9-postgres:5432/w9_emails".into());
    let api_token = std::env::var("W9_MAIL_API_TOKEN").unwrap_or_default();
    tracing::info!("Connecting to PostgreSQL...");
    let (client, conn) = tokio_postgres::connect(&db_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("DB: {}", e);
        }
    });
    client.query_one("SELECT 1", &[]).await?;
    tracing::info!("Connected to PostgreSQL");
    let state = AppState {
        db: Arc::new(client),
        api_token,
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?,
    };
    let router = Router::new()
        .nest_service("/w9-logo", ServeDir::new("public/w9-logo"))
        .route("/", get(home))
        .route("/login", get(login_page))
        .route("/oauth/callback", get(oauth_callback))
        .route("/logout", get(logout))
        .route("/dashboard", get(dashboard))
        .route("/tokens", get(tokens_page))
        .route("/tokens", post(tokens_create))
        .route("/tokens/revoke/:id", get(tokens_revoke))
        .route("/users", get(users_page))
        .route("/users", post(users_create))
        .route("/aliases", get(aliases_page))
        .route("/aliases", post(aliases_create))
        .route("/log", get(log_page))
        .route("/api/email/send", post(api_send))
        .route("/api/health", get(api_health))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("W9 Mail listening on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
