use crate::domain::errors::DomainError;
use crate::domain::value_objects::{Money, UserId, UserRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceLog {
    pub id: String,
    pub user_id: UserId,
    pub log_type: String, // "debit" or "credit"
    pub amount: Money,
    pub initial_balance: Money,
    pub final_balance: Money,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub whatsapp_number: String,
    pub password_hash: String,
    pub balance: Money,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        username: String,
        full_name: String,
        email: String,
        whatsapp_number: String,
        password_hash: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            username,
            full_name,
            email,
            whatsapp_number,
            password_hash,
            balance: Money::ZERO,
            role: UserRole::Member,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn debit(&mut self, amount: Money, reason: String) -> Result<BalanceLog, DomainError> {
        if self.balance < amount {
            return Err(DomainError::InsufficientBalance {
                current: self.balance.amount(),
                required: amount.amount(),
            });
        }

        let initial = self.balance;
        self.balance = self.balance - amount;
        self.updated_at = Utc::now();

        Ok(BalanceLog {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: self.id.clone(),
            log_type: "debit".to_string(),
            amount,
            initial_balance: initial,
            final_balance: self.balance,
            description: reason,
            created_at: self.updated_at,
        })
    }

    pub fn credit(&mut self, amount: Money, reason: String) -> BalanceLog {
        let initial = self.balance;
        self.balance = self.balance + amount;
        self.updated_at = Utc::now();

        BalanceLog {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: self.id.clone(),
            log_type: "credit".to_string(),
            amount,
            initial_balance: initial,
            final_balance: self.balance,
            description: reason,
            created_at: self.updated_at,
        }
    }
}
