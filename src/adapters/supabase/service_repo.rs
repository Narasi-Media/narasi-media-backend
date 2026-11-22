use crate::adapters::supabase::client::SupabaseClient;
use crate::application::ports::ServiceRepository;
use crate::domain::entity::{Category, Service};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{CategoryId, Money, ServiceId, ServiceType};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct CategoryRow {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sort_order: i32,
}

impl From<CategoryRow> for Category {
    fn from(row: CategoryRow) -> Self {
        Self {
            id: CategoryId::from_string(row.id),
            name: row.name,
            icon: row.icon,
            sort_order: row.sort_order as u32,
        }
    }
}

impl From<&Category> for CategoryRow {
    fn from(c: &Category) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name.clone(),
            icon: c.icon.clone(),
            sort_order: c.sort_order as i32,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceRow {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub service_type: String,
    pub price_original: i64,
    pub price_user: i64,
    pub min_order: i32,
    pub max_order: i32,
    pub refill_supported: bool,
    pub refill_days: i32,
    pub description: String,
    pub provider_id: Option<String>,
    pub provider_service_id: Option<String>,
    pub is_active: bool,
}

impl From<ServiceRow> for Service {
    fn from(row: ServiceRow) -> Self {
        let service_type = match row.service_type.to_lowercase().as_str() {
            "customcomments" => ServiceType::CustomComments,
            _ => ServiceType::Default,
        };

        Self {
            id: ServiceId::from_string(row.id),
            category_id: CategoryId::from_string(row.category_id),
            provider_id: row.provider_id,
            provider_service_id: row.provider_service_id,
            name: row.name,
            service_type,
            price_original: Money::from_idr(row.price_original),
            price_user: Money::from_idr(row.price_user),
            min_order: row.min_order as u32,
            max_order: row.max_order as u32,
            refill_supported: row.refill_supported,
            refill_days: row.refill_days as u32,
            description: row.description,
            is_active: row.is_active,
        }
    }
}

impl From<&Service> for ServiceRow {
    fn from(s: &Service) -> Self {
        Self {
            id: s.id.to_string(),
            category_id: s.category_id.to_string(),
            name: s.name.clone(),
            service_type: format!("{:?}", s.service_type),
            price_original: s.price_original.amount(),
            price_user: s.price_user.amount(),
            min_order: s.min_order as i32,
            max_order: s.max_order as i32,
            refill_supported: s.refill_supported,
            refill_days: s.refill_days as i32,
            description: s.description.clone(),
            provider_id: s.provider_id.clone(),
            provider_service_id: s.provider_service_id.clone(),
            is_active: s.is_active,
        }
    }
}

#[derive(Clone)]
pub struct SupabaseServiceRepository {
    client: Arc<SupabaseClient>,
}

impl SupabaseServiceRepository {
    pub fn new(client: Arc<SupabaseClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ServiceRepository for SupabaseServiceRepository {
    async fn find_by_id(&self, id: &ServiceId) -> Result<Option<Service>, DomainError> {
        let filter = format!("eq.{}", id);
        let row: Option<ServiceRow> = self
            .client
            .select_one("services", &[("id", &filter), ("limit", "1")])
            .await?;

        Ok(row.map(Into::into))
    }

    async fn list_all(&self) -> Result<Vec<Service>, DomainError> {
        let rows: Vec<ServiceRow> = self
            .client
            .select("services", &[("order", "name.asc")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_by_category(&self, category_id: &CategoryId) -> Result<Vec<Service>, DomainError> {
        let filter = format!("eq.{}", category_id);
        let rows: Vec<ServiceRow> = self
            .client
            .select("services", &[("category_id", &filter), ("order", "price_user.asc")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_categories(&self) -> Result<Vec<Category>, DomainError> {
        let rows: Vec<CategoryRow> = self
            .client
            .select("categories", &[("order", "sort_order.asc")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>, DomainError> {
        let filter = format!("eq.{}", id);
        let row: Option<CategoryRow> = self
            .client
            .select_one("categories", &[("id", &filter), ("limit", "1")])
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, service: &Service) -> Result<(), DomainError> {
        let row = ServiceRow::from(service);
        self.client.upsert("services", &row, "id").await
    }

    async fn delete(&self, id: &ServiceId) -> Result<(), DomainError> {
        let filter = format!("eq.{}", id);
        self.client.delete("services", &[("id", &filter)]).await
    }

    async fn save_category(&self, category: &Category) -> Result<(), DomainError> {
        let row = CategoryRow::from(category);
        self.client.upsert("categories", &row, "id").await
    }

    async fn delete_category(&self, id: &CategoryId) -> Result<(), DomainError> {
        let filter = format!("eq.{}", id);
        self.client.delete("categories", &[("id", &filter)]).await
    }
}
