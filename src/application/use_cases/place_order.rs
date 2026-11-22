use crate::application::ports::{OrderRepository, ServiceRepository, SmmProviderGateway, UserRepository};
use crate::domain::entity::Order;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{ServiceId, ServiceType, UserId};
use std::sync::Arc;

pub struct PlaceOrderInput {
    pub user_id: UserId,
    pub service_id: ServiceId,
    pub target: String,
    pub quantity: u32,
    pub custom_comments: Option<Vec<String>>,
}

pub struct PlaceOrderUseCase {
    user_repo: Arc<dyn UserRepository>,
    order_repo: Arc<dyn OrderRepository>,
    service_repo: Arc<dyn ServiceRepository>,
    provider_gateway: Option<Arc<dyn SmmProviderGateway>>,
}

impl PlaceOrderUseCase {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        order_repo: Arc<dyn OrderRepository>,
        service_repo: Arc<dyn ServiceRepository>,
        provider_gateway: Option<Arc<dyn SmmProviderGateway>>,
    ) -> Self {
        Self {
            user_repo,
            order_repo,
            service_repo,
            provider_gateway,
        }
    }

    pub async fn execute(&self, input: PlaceOrderInput) -> Result<Order, DomainError> {
        let service = self
            .service_repo
            .find_by_id(&input.service_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Service {}", input.service_id)))?;

        if !service.is_active {
            return Err(DomainError::ServiceInactive);
        }

        let quantity = if service.service_type == ServiceType::CustomComments {
            match &input.custom_comments {
                Some(comments) if !comments.is_empty() => comments.len() as u32,
                _ => return Err(DomainError::EmptyCommentsForCustomService),
            }
        } else {
            input.quantity
        };

        let total_price = service.calculate_price(quantity)?;

        let mut user = self
            .user_repo
            .find_by_id(&input.user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("User {}", input.user_id)))?;

        let balance_log = user.debit(
            total_price,
            format!(
                "Pemesanan {} untuk target {}",
                service.name, input.target
            ),
        )?;

        let mut order = Order::create(
            input.user_id.clone(),
            input.service_id.clone(),
            input.target.clone(),
            quantity,
            service.price_user,
            input.custom_comments.clone(),
        )?;

        if let Some(gateway) = &self.provider_gateway {
            let provider_service_id = service
                .provider_service_id
                .as_deref()
                .unwrap_or(service.id.as_str());

            let comments_ref = input.custom_comments.as_deref();
            if let Ok(res) = gateway
                .dispatch_order(
                    provider_service_id,
                    &input.target,
                    quantity,
                    comments_ref,
                )
                .await
            {
                order.set_processing(res.provider_order_id);
            }
        }

        self.user_repo.save(&user).await?;
        self.user_repo.save_balance_log(&balance_log).await?;
        self.order_repo.save(&order).await?;

        Ok(order)
    }
}
