use crate::adapters::supabase::client::SupabaseClient;
use crate::application::ports::UserRepository;
use crate::domain::entity::{BalanceLog, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{Money, UserId, UserRole};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct UserRow {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub whatsapp_number: String,
    pub telegram_username: Option<String>,
    pub password_hash: String,
    pub balance: i64,
    pub role: String,
    pub api_key: Option<String>,
    pub webhook_url: Option<String>,
    pub two_factor_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        let role = match row.role.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "reseller" => UserRole::Reseller,
            _ => UserRole::Member,
        };

        Self {
            id: UserId::from_string(row.id),
            username: row.username,
            full_name: row.full_name,
            email: row.email,
            whatsapp_number: row.whatsapp_number,
            password_hash: row.password_hash,
            balance: Money::from_idr(row.balance),
            role,
            api_key: row.api_key,
            webhook_url: row.webhook_url,
            two_factor_enabled: row.two_factor_enabled,
            telegram_username: row.telegram_username,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<&User> for UserRow {
    fn from(u: &User) -> Self {
        Self {
            id: u.id.to_string(),
            username: u.username.clone(),
            full_name: u.full_name.clone(),
            email: u.email.clone(),
            whatsapp_number: u.whatsapp_number.clone(),
            telegram_username: u.telegram_username.clone(),
            password_hash: u.password_hash.clone(),
            balance: u.balance.amount(),
            role: format!("{:?}", u.role),
            api_key: u.api_key.clone(),
            webhook_url: u.webhook_url.clone(),
            two_factor_enabled: u.two_factor_enabled,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BalanceLogRow {
    pub id: String,
    pub user_id: String,
    pub log_type: String,
    pub amount: i64,
    pub initial_balance: i64,
    pub final_balance: i64,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<BalanceLogRow> for BalanceLog {
    fn from(row: BalanceLogRow) -> Self {
        Self {
            id: row.id,
            user_id: UserId::from_string(row.user_id),
            log_type: row.log_type,
            amount: Money::from_idr(row.amount),
            initial_balance: Money::from_idr(row.initial_balance),
            final_balance: Money::from_idr(row.final_balance),
            description: row.description,
            created_at: row.created_at,
        }
    }
}

impl From<&BalanceLog> for BalanceLogRow {
    fn from(l: &BalanceLog) -> Self {
        Self {
            id: l.id.clone(),
            user_id: l.user_id.to_string(),
            log_type: l.log_type.clone(),
            amount: l.amount.amount(),
            initial_balance: l.initial_balance.amount(),
            final_balance: l.final_balance.amount(),
            description: l.description.clone(),
            created_at: l.created_at,
        }
    }
}

#[derive(Clone)]
pub struct SupabaseUserRepository {
    client: Arc<SupabaseClient>,
}

impl SupabaseUserRepository {
    pub fn new(client: Arc<SupabaseClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserRepository for SupabaseUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let filter = format!("eq.{}", id);
        let row: Option<UserRow> = self
            .client
            .select_one("users", &[("id", &filter), ("limit", "1")])
            .await?;

        Ok(row.map(Into::into))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let filter = format!("ilike.{}", username);
        let row: Option<UserRow> = self
            .client
            .select_one("users", &[("username", &filter), ("limit", "1")])
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let row = UserRow::from(user);
        self.client.upsert("users", &row, "id").await
    }

    async fn save_balance_log(&self, log: &BalanceLog) -> Result<(), DomainError> {
        let row = BalanceLogRow::from(log);
        let _: Option<BalanceLogRow> = self.client.insert("balance_logs", &row).await?;
        Ok(())
    }

    async fn list_balance_logs(&self, user_id: &UserId) -> Result<Vec<BalanceLog>, DomainError> {
        let filter = format!("eq.{}", user_id);
        let rows: Vec<BalanceLogRow> = self
            .client
            .select(
                "balance_logs",
                &[
                    ("user_id", &filter),
                    ("order", "created_at.desc"),
                    ("limit", "50"),
                ],
            )
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
