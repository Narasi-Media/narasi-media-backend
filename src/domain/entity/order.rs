use crate::domain::errors::DomainError;
use crate::domain::value_objects::{Money, OrderId, OrderStatus, ServiceId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub service_id: ServiceId,
    pub provider_order_id: Option<String>,
    pub target: String,
    pub quantity: u32,
    pub start_count: u32,
    pub remains: u32,
    pub price_per_k: Money,
    pub total_price: Money,
    pub custom_comments: Option<Vec<String>>,
    pub status: OrderStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn create(
        user_id: UserId,
        service_id: ServiceId,
        target: String,
        quantity: u32,
        price_per_k: Money,
        custom_comments: Option<Vec<String>>,
    ) -> Result<Self, DomainError> {
        if target.trim().is_empty() {
            return Err(DomainError::EmptyOrderTarget);
        }

        let total_price = Money::calculate_order_price(quantity, price_per_k);
        let now = Utc::now();

        Ok(Self {
            id: OrderId::new(),
            user_id,
            service_id,
            provider_order_id: None,
            target,
            quantity,
            start_count: 0,
            remains: quantity,
            price_per_k,
            total_price,
            custom_comments,
            status: OrderStatus::Pending,
            error_message: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn set_processing(&mut self, provider_order_id: String) {
        self.provider_order_id = Some(provider_order_id);
        self.status = OrderStatus::Processing;
        self.updated_at = Utc::now();
    }

    pub fn update_status(
        &mut self,
        new_status: OrderStatus,
        start_count: u32,
        remains: u32,
        error_msg: Option<String>,
    ) {
        self.status = new_status;
        self.start_count = start_count;
        self.remains = remains;
        self.error_message = error_msg;
        self.updated_at = Utc::now();
    }

    pub fn calculate_partial_refund(&self) -> Money {
        if self.status != OrderStatus::Partial && self.status != OrderStatus::Canceled {
            return Money::ZERO;
        }

        if self.status == OrderStatus::Canceled {
            return self.total_price;
        }

        Money::calculate_partial_refund(self.remains, self.quantity, self.total_price)
    }
}
