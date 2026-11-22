use crate::application::ports::{OrderRepository, ServiceRepository, SmmProviderGateway};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::OrderId;
use std::sync::Arc;

pub struct ClaimRefillUseCase {
    order_repo: Arc<dyn OrderRepository>,
    service_repo: Arc<dyn ServiceRepository>,
    provider_gateway: Arc<dyn SmmProviderGateway>,
}

impl ClaimRefillUseCase {
    pub fn new(
        order_repo: Arc<dyn OrderRepository>,
        service_repo: Arc<dyn ServiceRepository>,
        provider_gateway: Arc<dyn SmmProviderGateway>,
    ) -> Self {
        Self {
            order_repo,
            service_repo,
            provider_gateway,
        }
    }

    pub async fn execute(&self, order_id: &OrderId) -> Result<String, DomainError> {
        let order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Order {}", order_id)))?;

        if !order.status.is_eligible_for_refill() {
            return Err(DomainError::RefillNotEligible {
                id: order.id.to_string(),
            });
        }

        let service = self
            .service_repo
            .find_by_id(&order.service_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Service {}", order.service_id)))?;

        if !service.refill_supported {
            return Err(DomainError::RefillNotEligible {
                id: order.id.to_string(),
            });
        }

        let provider_order_id = order
            .provider_order_id
            .as_deref()
            .ok_or_else(|| DomainError::RefillNotEligible {
                id: order.id.to_string(),
            })?;

        let refill_id = self.provider_gateway.request_refill(provider_order_id).await?;
        Ok(refill_id)
    }
}
