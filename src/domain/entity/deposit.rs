use crate::domain::errors::DomainError;
use crate::domain::value_objects::{DepositId, DepositMethod, DepositStatus, Money, UserId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub id: DepositId,
    pub user_id: UserId,
    pub method: DepositMethod,
    pub requested_amount: Money,
    pub unique_code: u16,
    pub total_to_pay: Money,
    pub balance_credited: Money,
    pub proof_image_url: Option<String>,
    pub sender_name: Option<String>,
    pub status: DepositStatus,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}

impl Deposit {
    pub const MIN_DEPOSIT_IDR: i64 = 10_000;

    pub fn create_manual(
        user_id: UserId,
        method: DepositMethod,
        requested_amount: Money,
        unique_code: u16,
        bonus_percentage: f64,
    ) -> Result<Self, DomainError> {
        if requested_amount.amount() < Self::MIN_DEPOSIT_IDR {
            return Err(DomainError::DepositBelowMinimum {
                requested: requested_amount.amount(),
                min: Self::MIN_DEPOSIT_IDR,
            });
        }

        let total_to_pay = Money::from_idr(requested_amount.amount() + unique_code as i64);
        let bonus = (requested_amount.amount() as f64 * (bonus_percentage / 100.0)).floor() as i64;
        let balance_credited = Money::from_idr(requested_amount.amount() + bonus);

        let now = Utc::now();
        let expired_at = now + Duration::hours(24);

        Ok(Self {
            id: DepositId::new(),
            user_id,
            method,
            requested_amount,
            unique_code,
            total_to_pay,
            balance_credited,
            proof_image_url: None,
            sender_name: None,
            status: DepositStatus::Pending,
            rejection_reason: None,
            created_at: now,
            updated_at: now,
            expired_at,
        })
    }

    pub fn submit_proof(
        &mut self,
        sender_name: String,
        proof_image_url: String,
    ) -> Result<(), DomainError> {
        if self.status != DepositStatus::Pending {
            return Err(DomainError::InvalidDepositStateTransition {
                current_status: format!("{:?}", self.status),
            });
        }

        self.sender_name = Some(sender_name);
        self.proof_image_url = Some(proof_image_url);
        self.status = DepositStatus::WaitingVerification;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn approve(&mut self) -> Result<Money, DomainError> {
        if self.status != DepositStatus::Pending
            && self.status != DepositStatus::WaitingVerification
        {
            return Err(DomainError::InvalidDepositStateTransition {
                current_status: format!("{:?}", self.status),
            });
        }

        self.status = DepositStatus::Success;
        self.updated_at = Utc::now();
        Ok(self.balance_credited)
    }

    pub fn reject(&mut self, reason: String) -> Result<(), DomainError> {
        if self.status != DepositStatus::Pending
            && self.status != DepositStatus::WaitingVerification
        {
            return Err(DomainError::InvalidDepositStateTransition {
                current_status: format!("{:?}", self.status),
            });
        }

        self.status = DepositStatus::Rejected;
        self.rejection_reason = Some(reason);
        self.updated_at = Utc::now();
        Ok(())
    }
}
