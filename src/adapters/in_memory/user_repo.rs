use crate::application::ports::UserRepository;
use crate::domain::entity::{BalanceLog, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct InMemoryUserRepository {
    users: Arc<RwLock<HashMap<UserId, User>>>,
    logs: Arc<RwLock<Vec<BalanceLog>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let guard = self.users.read().await;
        Ok(guard.get(id).cloned())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let guard = self.users.read().await;
        Ok(guard
            .values()
            .find(|u| u.username.eq_ignore_ascii_case(username))
            .cloned())
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let mut guard = self.users.write().await;
        guard.insert(user.id.clone(), user.clone());
        Ok(())
    }

    async fn save_balance_log(&self, log: &BalanceLog) -> Result<(), DomainError> {
        let mut guard = self.logs.write().await;
        guard.push(log.clone());
        Ok(())
    }

    async fn list_balance_logs(&self, user_id: &UserId) -> Result<Vec<BalanceLog>, DomainError> {
        let guard = self.logs.read().await;
        let mut filtered: Vec<BalanceLog> = guard
            .iter()
            .filter(|l| &l.user_id == user_id)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(filtered)
    }
}
