use crate::application::ports::{
    FileStorageGateway, ProviderOrderResult, ProviderStatusResult, SmmProviderGateway,
};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::OrderStatus;
use async_trait::async_trait;
use uuid::Uuid;

pub struct MockSmmProviderGateway;

#[async_trait]
impl SmmProviderGateway for MockSmmProviderGateway {
    async fn dispatch_order(
        &self,
        _service_id: &str,
        _target: &str,
        _quantity: u32,
        _custom_comments: Option<&[String]>,
    ) -> Result<ProviderOrderResult, DomainError> {
        Ok(ProviderOrderResult {
            provider_order_id: format!("SMM-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
        })
    }

    async fn check_status(
        &self,
        _provider_order_id: &str,
    ) -> Result<ProviderStatusResult, DomainError> {
        Ok(ProviderStatusResult {
            status: OrderStatus::Success,
            start_count: 100,
            remains: 0,
            error_message: None,
        })
    }

    async fn request_refill(&self, provider_order_id: &str) -> Result<String, DomainError> {
        Ok(format!("REFILL-{}", provider_order_id))
    }
}

pub struct MockFileStorageGateway;

#[async_trait]
impl FileStorageGateway for MockFileStorageGateway {
    async fn store_payment_proof(
        &self,
        filename: &str,
        _data: &[u8],
    ) -> Result<String, DomainError> {
        Ok(format!("/uploads/receipts/{}-{}", Uuid::new_v4(), filename))
    }
}
