use crate::application::ports::DepositRepository;
use crate::domain::entity::Deposit;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{DepositId, DepositStatus, UserId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct InMemoryDepositRepository {
    deposits: Arc<RwLock<HashMap<DepositId, Deposit>>>,
}

impl InMemoryDepositRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DepositRepository for InMemoryDepositRepository {
    async fn find_by_id(&self, id: &DepositId) -> Result<Option<Deposit>, DomainError> {
        let guard = self.deposits.read().await;
        Ok(guard.get(id).cloned())
    }

    async fn save(&self, deposit: &Deposit) -> Result<(), DomainError> {
        let mut guard = self.deposits.write().await;
        guard.insert(deposit.id.clone(), deposit.clone());
        Ok(())
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Deposit>, DomainError> {
        let guard = self.deposits.read().await;
        let mut list: Vec<Deposit> = guard
            .values()
            .filter(|d| &d.user_id == user_id)
            .cloned()
            .collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }

    async fn list_pending(&self) -> Result<Vec<Deposit>, DomainError> {
        let guard = self.deposits.read().await;
        let mut list: Vec<Deposit> = guard
            .values()
            .filter(|d| {
                d.status == DepositStatus::Pending
                    || d.status == DepositStatus::WaitingVerification
            })
            .cloned()
            .collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }

    async fn list_all(&self) -> Result<Vec<Deposit>, DomainError> {
        let guard = self.deposits.read().await;
        let mut list: Vec<Deposit> = guard.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }
}
