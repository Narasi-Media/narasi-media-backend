use crate::adapters::supabase::client::SupabaseClient;
use crate::application::ports::DepositRepository;
use crate::domain::entity::Deposit;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{DepositId, DepositMethod, DepositStatus, Money, UserId};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct DepositRow {
    pub id: String,
    pub user_id: String,
    pub method: String,
    pub requested_amount: i64,
    pub unique_code: i32,
    pub total_to_pay: i64,
    pub balance_credited: i64,
    pub sender_name: Option<String>,
    pub proof_image_url: Option<String>,
    pub status: String,
    pub rejected_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

impl From<DepositRow> for Deposit {
    fn from(row: DepositRow) -> Self {
        let status = match row.status.to_lowercase().as_str() {
            "waitingverification" | "waiting_verification" => DepositStatus::WaitingVerification,
            "success" | "approved" => DepositStatus::Success,
            "rejected" => DepositStatus::Rejected,
            _ => DepositStatus::Pending,
        };

        let method = match row.method.to_lowercase().as_str() {
            "manualbankbca" | "manual_bank_bca" | "bca" => DepositMethod::ManualBankBca,
            "manualbankmandiri" | "manual_bank_mandiri" | "mandiri" => DepositMethod::ManualBankMandiri,
            "manualbankbri" | "manual_bank_bri" | "bri" => DepositMethod::ManualBankBri,
            "automaticgateway" | "automatic_gateway" => DepositMethod::AutomaticGateway,
            _ => DepositMethod::ManualQris,
        };

        let created_at = row.created_at;
        let expired_at = created_at + Duration::hours(24);

        Self {
            id: DepositId::from_string(row.id),
            user_id: UserId::from_string(row.user_id),
            method,
            requested_amount: Money::from_idr(row.requested_amount),
            unique_code: row.unique_code as u16,
            total_to_pay: Money::from_idr(row.total_to_pay),
            balance_credited: Money::from_idr(row.balance_credited),
            sender_name: row.sender_name,
            proof_image_url: row.proof_image_url,
            status,
            rejection_reason: row.rejected_reason,
            created_at,
            updated_at: row.approved_at.unwrap_or(created_at),
            expired_at,
        }
    }
}

impl From<&Deposit> for DepositRow {
    fn from(d: &Deposit) -> Self {
        Self {
            id: d.id.to_string(),
            user_id: d.user_id.to_string(),
            method: format!("{:?}", d.method),
            requested_amount: d.requested_amount.amount(),
            unique_code: d.unique_code as i32,
            total_to_pay: d.total_to_pay.amount(),
            balance_credited: d.balance_credited.amount(),
            sender_name: d.sender_name.clone(),
            proof_image_url: d.proof_image_url.clone(),
            status: format!("{:?}", d.status),
            rejected_reason: d.rejection_reason.clone(),
            created_at: d.created_at,
            approved_at: if d.status == DepositStatus::Success {
                Some(d.updated_at)
            } else {
                None
            },
        }
    }
}

#[derive(Clone)]
pub struct SupabaseDepositRepository {
    client: Arc<SupabaseClient>,
}

impl SupabaseDepositRepository {
    pub fn new(client: Arc<SupabaseClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DepositRepository for SupabaseDepositRepository {
    async fn find_by_id(&self, id: &DepositId) -> Result<Option<Deposit>, DomainError> {
        let filter = format!("eq.{}", id);
        let row: Option<DepositRow> = self
            .client
            .select_one("deposits", &[("id", &filter), ("limit", "1")])
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, deposit: &Deposit) -> Result<(), DomainError> {
        let row = DepositRow::from(deposit);
        self.client.upsert("deposits", &row, "id").await
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Deposit>, DomainError> {
        let filter = format!("eq.{}", user_id);
        let rows: Vec<DepositRow> = self
            .client
            .select("deposits", &[("user_id", &filter), ("order", "created_at.desc")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_pending(&self) -> Result<Vec<Deposit>, DomainError> {
        let rows: Vec<DepositRow> = self
            .client
            .select(
                "deposits",
                &[
                    ("status", "in.(Pending,WaitingVerification)"),
                    ("order", "created_at.desc"),
                ],
            )
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_all(&self) -> Result<Vec<Deposit>, DomainError> {
        let rows: Vec<DepositRow> = self
            .client
            .select("deposits", &[("order", "created_at.desc")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
