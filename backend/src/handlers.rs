use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use sqlx::Row;
use uuid::Uuid;

use crate::{AppState, CreateAccountRequest, EmailAccount, InboxQuery, SendEmailRequest, UpdateAccountRequest};

pub async fn get_accounts(State(state): State<AppState>) -> Result<Json<Vec<EmailAccount>>, StatusCode> {
    let rows = sqlx::query("SELECT id, email, display_name, is_active FROM accounts")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let accounts: Vec<EmailAccount> = rows
        .into_iter()
        .map(|row| EmailAccount {
            id: row.get::<String, _>(0),
            email: row.get::<String, _>(1),
            display_name: row.get::<String, _>(2),
            is_active: row.get::<bool, _>(3),
        })
        .collect();

    Ok(Json(accounts))
}

pub async fn create_account(
    State(state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<EmailAccount>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO accounts (id, email, display_name, password, is_active) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.email)
    .bind(&req.display_name)
    .bind(&req.password)
    .bind(req.is_active)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account = EmailAccount {
        id,
        email: req.email,
        display_name: req.display_name,
        is_active: req.is_active,
    };

    Ok(Json(account))
}

pub async fn update_account(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<EmailAccount>, StatusCode> {
    if let Some(is_active) = req.is_active {
        sqlx::query("UPDATE accounts SET is_active = ? WHERE id = ?")
            .bind(is_active)
            .bind(&id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let row = sqlx::query("SELECT id, email, display_name, is_active FROM accounts WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let account = EmailAccount {
        id: row.get::<String, _>(0),
        email: row.get::<String, _>(1),
        display_name: row.get::<String, _>(2),
        is_active: row.get::<bool, _>(3),
    };

    Ok(Json(account))
}

pub async fn send_email(
    State(_state): State<AppState>,
    Json(req): Json<SendEmailRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement email sending using lettre
    Ok(Json(serde_json::json!({
        "status": "sent",
        "message": "Email sent successfully"
    })))
}

pub async fn get_inbox(
    State(_state): State<AppState>,
    Query(params): Query<InboxQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement IMAP inbox retrieval
    Ok(Json(serde_json::json!([])))
}

