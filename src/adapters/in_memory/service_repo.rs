use crate::application::ports::ServiceRepository;
use crate::domain::entity::{Category, Service};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{CategoryId, ServiceId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct InMemoryServiceRepository {
    services: Arc<RwLock<HashMap<ServiceId, Service>>>,
    categories: Arc<RwLock<HashMap<CategoryId, Category>>>,
}

impl InMemoryServiceRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ServiceRepository for InMemoryServiceRepository {
    async fn find_by_id(&self, id: &ServiceId) -> Result<Option<Service>, DomainError> {
        let guard = self.services.read().await;
        Ok(guard.get(id).cloned())
    }

    async fn list_all(&self) -> Result<Vec<Service>, DomainError> {
        let guard = self.services.read().await;
        let mut list: Vec<Service> = guard.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }

    async fn list_by_category(&self, category_id: &CategoryId) -> Result<Vec<Service>, DomainError> {
        let guard = self.services.read().await;
        let list: Vec<Service> = guard
            .values()
            .filter(|s| &s.category_id == category_id && s.is_active)
            .cloned()
            .collect();
        Ok(list)
    }

    async fn list_categories(&self) -> Result<Vec<Category>, DomainError> {
        let guard = self.categories.read().await;
        let mut list: Vec<Category> = guard.values().cloned().collect();
        list.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
        Ok(list)
    }

    async fn save(&self, service: &Service) -> Result<(), DomainError> {
        let mut guard = self.services.write().await;
        guard.insert(service.id.clone(), service.clone());
        Ok(())
    }

    async fn save_category(&self, category: &Category) -> Result<(), DomainError> {
        let mut guard = self.categories.write().await;
        guard.insert(category.id.clone(), category.clone());
        Ok(())
    }
}
