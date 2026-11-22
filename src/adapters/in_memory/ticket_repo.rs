use crate::application::ports::TicketRepository;
use crate::domain::entity::Ticket;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{TicketId, UserId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct InMemoryTicketRepository {
    tickets: Arc<RwLock<HashMap<TicketId, Ticket>>>,
}

impl InMemoryTicketRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TicketRepository for InMemoryTicketRepository {
    async fn find_by_id(&self, id: &TicketId) -> Result<Option<Ticket>, DomainError> {
        let guard = self.tickets.read().await;
        Ok(guard.get(id).cloned())
    }

    async fn save(&self, ticket: &Ticket) -> Result<(), DomainError> {
        let mut guard = self.tickets.write().await;
        guard.insert(ticket.id.clone(), ticket.clone());
        Ok(())
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Ticket>, DomainError> {
        let guard = self.tickets.read().await;
        let mut list: Vec<Ticket> = guard
            .values()
            .filter(|t| &t.user_id == user_id)
            .cloned()
            .collect();
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }

    async fn list_all(&self) -> Result<Vec<Ticket>, DomainError> {
        let guard = self.tickets.read().await;
        let mut list: Vec<Ticket> = guard.values().cloned().collect();
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }
}
