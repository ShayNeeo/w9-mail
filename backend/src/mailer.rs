use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SenderKind {
    Account,
    Alias,
}

impl SenderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SenderKind::Account => "account",
            SenderKind::Alias => "alias",
        }
    }
}

impl TryFrom<String> for SenderKind {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "account" => Ok(SenderKind::Account),
            "alias" => Ok(SenderKind::Alias),
            other => Err(anyhow!("Unknown sender type: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedSender {
    pub header_from: String,
    pub auth_email: String,
    pub auth_password: String,
}

#[derive(Debug, Clone)]
pub struct SenderSummary {
    pub sender_type: SenderKind,
    pub sender_id: String,
    pub email: String,
    pub display_label: String,
    pub via_display: Option<String>,
    pub is_active: bool,
    pub credentials: ResolvedSender,
}

pub async fn resolve_sender_by_email(
    db: &SqlitePool,
    email: &str,
) -> anyhow::Result<ResolvedSender> {
    if let Some(row) = sqlx::query(
        "SELECT email, password FROM accounts WHERE email = ? AND is_active = 1",
    )
    .bind(email)
    .fetch_optional(db)
    .await?
    {
        return Ok(ResolvedSender {
            header_from: row.get::<String, _>(0),
            auth_email: row.get::<String, _>(0),
            auth_password: row.get::<String, _>(1),
        });
    }

    if let Some(row) = sqlx::query(
        r#"
        SELECT aliases.alias_email,
               accounts.email,
               accounts.password,
               aliases.is_active,
               accounts.is_active
        FROM aliases
        JOIN accounts ON aliases.account_id = accounts.id
        WHERE aliases.alias_email = ?
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await?
    {
        let alias_active = row.get::<bool, _>(3);
        let account_active = row.get::<bool, _>(4);
        if alias_active && account_active {
            return Ok(ResolvedSender {
                header_from: row.get::<String, _>(0),
                auth_email: row.get::<String, _>(1),
                auth_password: row.get::<String, _>(2),
            });
        }
    }

    Err(anyhow!(
        "Sender account or alias not found or inactive for {}",
        email
    ))
}

async fn summarize_account_by_id(db: &SqlitePool, account_id: &str) -> anyhow::Result<SenderSummary> {
    let row = sqlx::query(
        "SELECT id, email, display_name, password, is_active FROM accounts WHERE id = ?",
    )
    .bind(account_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| anyhow!("Account not found"))?;

    let is_active = row.get::<bool, _>(4);
    if !is_active {
        return Err(anyhow!("Account is inactive"));
    }

    let email = row.get::<String, _>(1);
    let display_name = row.get::<String, _>(2);
    let password = row.get::<String, _>(3);

    Ok(SenderSummary {
        sender_type: SenderKind::Account,
        sender_id: row.get::<String, _>(0),
        email: email.clone(),
        display_label: display_name.clone(),
        via_display: None,
        is_active,
        credentials: ResolvedSender {
            header_from: email.clone(),
            auth_email: email,
            auth_password: password,
        },
    })
}

async fn summarize_alias_by_id(db: &SqlitePool, alias_id: &str) -> anyhow::Result<SenderSummary> {
    let row = sqlx::query(
        r#"
        SELECT 
            aliases.id,
            aliases.alias_email,
            aliases.display_name,
            aliases.is_active,
            accounts.id,
            accounts.email,
            accounts.display_name,
            accounts.password,
            accounts.is_active
        FROM aliases
        JOIN accounts ON aliases.account_id = accounts.id
        WHERE aliases.id = ?
        "#,
    )
    .bind(alias_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| anyhow!("Alias not found"))?;

    let alias_active = row.get::<bool, _>(3);
    let account_active = row.get::<bool, _>(8);
    if !alias_active {
        return Err(anyhow!("Alias is inactive"));
    }
    if !account_active {
        return Err(anyhow!("Underlying account is inactive"));
    }

    let alias_email = row.get::<String, _>(1);
    let alias_display = row.get::<Option<String>, _>(2);
    let account_email = row.get::<String, _>(5);
    let account_display = row.get::<String, _>(6);
    let password = row.get::<String, _>(7);

    Ok(SenderSummary {
        sender_type: SenderKind::Alias,
        sender_id: row.get::<String, _>(0),
        email: alias_email.clone(),
        display_label: alias_display.unwrap_or_else(|| alias_email.clone()),
        via_display: Some(format!("{} ({})", account_display, account_email)),
        is_active: alias_active && account_active,
        credentials: ResolvedSender {
            header_from: alias_email,
            auth_email: account_email,
            auth_password: password,
        },
    })
}

pub async fn summarize_sender(
    db: &SqlitePool,
    sender_type: SenderKind,
    sender_id: &str,
) -> anyhow::Result<SenderSummary> {
    match sender_type {
        SenderKind::Account => summarize_account_by_id(db, sender_id).await,
        SenderKind::Alias => summarize_alias_by_id(db, sender_id).await,
    }
}

pub async fn get_default_sender_summary(
    db: &SqlitePool,
) -> anyhow::Result<Option<SenderSummary>> {
    let row = sqlx::query("SELECT sender_type, sender_id FROM default_sender WHERE singleton = 1")
        .fetch_optional(db)
        .await?;

    if let Some(row) = row {
        let sender_type: SenderKind = row.get::<String, _>(0).try_into()?;
        let sender_id = row.get::<String, _>(1);
        let summary = summarize_sender(db, sender_type, &sender_id).await?;
        Ok(Some(summary))
    } else {
        Ok(None)
    }
}

pub async fn upsert_default_sender(
    db: &SqlitePool,
    sender_type: SenderKind,
    sender_id: &str,
) -> anyhow::Result<SenderSummary> {
    let summary = summarize_sender(db, sender_type, sender_id).await?;

    sqlx::query(
        r#"
        INSERT INTO default_sender (singleton, sender_type, sender_id)
        VALUES (1, ?, ?)
        ON CONFLICT(singleton) DO UPDATE SET sender_type = excluded.sender_type, sender_id = excluded.sender_id
        "#,
    )
    .bind(sender_type.as_str())
    .bind(&summary.sender_id)
    .execute(db)
    .await?;

    Ok(summary)
}

pub async fn delete_default_if_matches(
    db: &SqlitePool,
    sender_type: SenderKind,
    sender_id: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "DELETE FROM default_sender WHERE singleton = 1 AND sender_type = ? AND sender_id = ?",
    )
    .bind(sender_type.as_str())
    .bind(sender_id)
    .execute(db)
    .await?;
    Ok(())
}


