use crate::application::ports::DepositRepository;
use crate::domain::entity::Deposit;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{DepositMethod, Money, UserId};
use rand::Rng;
use std::sync::Arc;

pub struct RequestDepositInput {
    pub user_id: UserId,
    pub method: DepositMethod,
    pub amount: Money,
    pub bonus_percentage: f64,
}

pub struct RequestDepositUseCase {
    deposit_repo: Arc<dyn DepositRepository>,
}

impl RequestDepositUseCase {
    pub fn new(deposit_repo: Arc<dyn DepositRepository>) -> Self {
        Self { deposit_repo }
    }

    pub async fn execute(&self, input: RequestDepositInput) -> Result<Deposit, DomainError> {
        let unique_code: u16 = {
            let mut rng = rand::thread_rng();
            rng.gen_range(100..=999)
        };

        let deposit = Deposit::create_manual(
            input.user_id,
            input.method,
            input.amount,
            unique_code,
            input.bonus_percentage,
        )?;

        self.deposit_repo.save(&deposit).await?;
        Ok(deposit)
    }
}
