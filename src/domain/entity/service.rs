use crate::domain::errors::DomainError;
use crate::domain::value_objects::{CategoryId, Money, ServiceId, ServiceType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub icon: String,
    pub sort_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: ServiceId,
    pub category_id: CategoryId,
    pub provider_id: Option<String>,
    pub provider_service_id: Option<String>,
    pub name: String,
    pub service_type: ServiceType,
    pub price_original: Money,
    pub price_user: Money, // Harga jual per 1.000 ke pengguna
    pub min_order: u32,
    pub max_order: u32,
    pub refill_supported: bool,
    pub refill_days: u32,
    pub description: String,
    pub is_active: bool,
}

impl Service {
    pub fn calculate_price(&self, quantity: u32) -> Result<Money, DomainError> {
        if !self.is_active {
            return Err(DomainError::ServiceInactive);
        }

        if quantity < self.min_order || quantity > self.max_order {
            return Err(DomainError::InvalidOrderQuantity {
                quantity,
                min: self.min_order,
                max: self.max_order,
            });
        }

        Ok(Money::calculate_order_price(quantity, self.price_user))
    }
}
