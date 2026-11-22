use crate::application::ports::{OrderRepository, SmmProviderGateway, UserRepository};
use crate::domain::entity::Order;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{Money, OrderId, OrderStatus};
use std::sync::Arc;

pub struct SyncOrderUseCase {
    order_repo: Arc<dyn OrderRepository>,
    user_repo: Arc<dyn UserRepository>,
    provider_gateway: Arc<dyn SmmProviderGateway>,
}

impl SyncOrderUseCase {
    pub fn new(
        order_repo: Arc<dyn OrderRepository>,
        user_repo: Arc<dyn UserRepository>,
        provider_gateway: Arc<dyn SmmProviderGateway>,
    ) -> Self {
        Self {
            order_repo,
            user_repo,
            provider_gateway,
        }
    }

    pub async fn execute(&self, order_id: &OrderId) -> Result<Order, DomainError> {
        let mut order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Order {}", order_id)))?;

        let provider_order_id = match &order.provider_order_id {
            Some(pid) => pid.clone(),
            None => return Ok(order),
        };

        let result = self.provider_gateway.check_status(&provider_order_id).await?;

        let old_status = order.status;
        order.update_status(
            result.status,
            result.start_count,
            result.remains,
            result.error_message,
        );

        if (order.status == OrderStatus::Partial || order.status == OrderStatus::Canceled)
            && old_status != OrderStatus::Partial
            && old_status != OrderStatus::Canceled
        {
            let refund_amount = order.calculate_partial_refund();
            if refund_amount > Money::ZERO {
                if let Some(mut user) = self.user_repo.find_by_id(&order.user_id).await? {
                    let log = user.credit(
                        refund_amount,
                        format!(
                            "Pengembalian saldo untuk pesanan #{} (status: {:?})",
                            order.id, order.status
                        ),
                    );
                    self.user_repo.save(&user).await?;
                    self.user_repo.save_balance_log(&log).await?;
                }
            }
        }

        self.order_repo.save(&order).await?;
        Ok(order)
    }
}
