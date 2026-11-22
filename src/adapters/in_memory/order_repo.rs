use crate::application::ports::OrderRepository;
use crate::domain::entity::Order;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{OrderId, OrderStatus, UserId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct InMemoryOrderRepository {
    orders: Arc<RwLock<HashMap<OrderId, Order>>>,
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, DomainError> {
        let guard = self.orders.read().await;
        Ok(guard.get(id).cloned())
    }

    async fn save(&self, order: &Order) -> Result<(), DomainError> {
        let mut guard = self.orders.write().await;
        guard.insert(order.id.clone(), order.clone());
        Ok(())
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Order>, DomainError> {
        let guard = self.orders.read().await;
        let mut list: Vec<Order> = guard
            .values()
            .filter(|o| &o.user_id == user_id)
            .cloned()
            .collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }

    async fn list_all(&self) -> Result<Vec<Order>, DomainError> {
        let guard = self.orders.read().await;
        let mut list: Vec<Order> = guard.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }

    async fn list_active(&self) -> Result<Vec<Order>, DomainError> {
        let guard = self.orders.read().await;
        let list: Vec<Order> = guard
            .values()
            .filter(|o| o.status == OrderStatus::Pending || o.status == OrderStatus::Processing)
            .cloned()
            .collect();
        Ok(list)
    }
}
