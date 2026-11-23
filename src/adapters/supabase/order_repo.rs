use crate::adapters::supabase::client::SupabaseClient;
use crate::application::ports::OrderRepository;
use crate::domain::entity::Order;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{Money, OrderId, OrderStatus, ServiceId, UserId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct OrderRow {
    pub id: String,
    pub user_id: String,
    pub service_id: String,
    pub target: String,
    pub quantity: i32,
    pub start_count: i32,
    pub remains: i32,
    pub unit_price: i64,
    pub total_price: i64,
    pub status: String,
    pub provider_order_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OrderRow> for Order {
    fn from(row: OrderRow) -> Self {
        let status = match row.status.to_lowercase().as_str() {
            "processing" => OrderStatus::Processing,
            "inprogress" | "in_progress" => OrderStatus::InProgress,
            "success" | "completed" => OrderStatus::Success,
            "partial" => OrderStatus::Partial,
            "canceled" | "cancelled" => OrderStatus::Canceled,
            "error" => OrderStatus::Error,
            _ => OrderStatus::Pending,
        };

        Self {
            id: OrderId::from_string(row.id),
            user_id: UserId::from_string(row.user_id),
            service_id: ServiceId::from_string(row.service_id),
            provider_order_id: row.provider_order_id,
            target: row.target,
            quantity: row.quantity as u32,
            start_count: row.start_count as u32,
            remains: row.remains as u32,
            price_per_k: Money::from_idr(row.unit_price),
            total_price: Money::from_idr(row.total_price),
            custom_comments: None,
            status,
            error_message: row.error_message,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<&Order> for OrderRow {
    fn from(o: &Order) -> Self {
        Self {
            id: o.id.to_string(),
            user_id: o.user_id.to_string(),
            service_id: o.service_id.to_string(),
            target: o.target.clone(),
            quantity: o.quantity as i32,
            start_count: o.start_count as i32,
            remains: o.remains as i32,
            unit_price: o.price_per_k.amount(),
            total_price: o.total_price.amount(),
            status: format!("{:?}", o.status),
            provider_order_id: o.provider_order_id.clone(),
            error_message: o.error_message.clone(),
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct SupabaseOrderRepository {
    client: Arc<SupabaseClient>,
}

impl SupabaseOrderRepository {
    pub fn new(client: Arc<SupabaseClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl OrderRepository for SupabaseOrderRepository {
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, DomainError> {
        let filter = format!("eq.{}", id);
        let row: Option<OrderRow> = self
            .client
            .select_one("orders", &[("id", &filter), ("limit", "1")])
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, order: &Order) -> Result<(), DomainError> {
        let row = OrderRow::from(order);
        self.client.upsert("orders", &row, "id").await
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Order>, DomainError> {
        let filter = format!("eq.{}", user_id);
        let rows: Vec<OrderRow> = self
            .client
            .select("orders", &[("user_id", &filter), ("order", "created_at.desc"), ("limit", "100")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_all(&self) -> Result<Vec<Order>, DomainError> {
        let rows: Vec<OrderRow> = self
            .client
            .select("orders", &[("order", "created_at.desc"), ("limit", "200")])
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_active(&self) -> Result<Vec<Order>, DomainError> {
        let rows: Vec<OrderRow> = self
            .client
            .select(
                "orders",
                &[
                    ("status", "in.(Pending,Processing,Inprogress)"),
                    ("order", "created_at.desc"),
                    ("limit", "100"),
                ],
            )
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
