use crate::application::ports::{DepositRepository, UserRepository};
use crate::domain::entity::Deposit;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::DepositId;
use std::sync::Arc;

pub struct ApproveDepositUseCase {
    deposit_repo: Arc<dyn DepositRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl ApproveDepositUseCase {
    pub fn new(
        deposit_repo: Arc<dyn DepositRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            deposit_repo,
            user_repo,
        }
    }

    pub async fn execute(&self, deposit_id: &DepositId) -> Result<Deposit, DomainError> {
        let mut deposit = self
            .deposit_repo
            .find_by_id(deposit_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Deposit {}", deposit_id)))?;

        let balance_credited = deposit.approve()?;

        let mut user = self
            .user_repo
            .find_by_id(&deposit.user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("User {}", deposit.user_id)))?;

        let balance_log = user.credit(
            balance_credited,
            format!("Top-up deposit via QRIS #{}", deposit.id),
        );

        self.user_repo.save(&user).await?;
        self.user_repo.save_balance_log(&balance_log).await?;
        self.deposit_repo.save(&deposit).await?;

        Ok(deposit)
    }

    pub async fn reject(&self, deposit_id: &DepositId, reason: String) -> Result<Deposit, DomainError> {
        let mut deposit = self
            .deposit_repo
            .find_by_id(deposit_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Deposit {}", deposit_id)))?;

        deposit.reject(reason)?;
        self.deposit_repo.save(&deposit).await?;
        Ok(deposit)
    }
}
