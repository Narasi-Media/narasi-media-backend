use crate::domain::errors::DomainError;
use crate::domain::value_objects::OrderStatus;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ProviderOrderResult {
    pub provider_order_id: String,
}

#[derive(Debug, Clone)]
pub struct ProviderStatusResult {
    pub status: OrderStatus,
    pub start_count: u32,
    pub remains: u32,
    pub error_message: Option<String>,
}

#[async_trait]
pub trait SmmProviderGateway: Send + Sync {
    async fn dispatch_order(
        &self,
        service_id: &str,
        target: &str,
        quantity: u32,
        custom_comments: Option<&[String]>,
    ) -> Result<ProviderOrderResult, DomainError>;

    async fn check_status(
        &self,
        provider_order_id: &str,
    ) -> Result<ProviderStatusResult, DomainError>;

    async fn request_refill(&self, provider_order_id: &str) -> Result<String, DomainError>;
}

#[async_trait]
pub trait FileStorageGateway: Send + Sync {
    async fn store_payment_proof(
        &self,
        filename: &str,
        data: &[u8],
    ) -> Result<String, DomainError>;
}
