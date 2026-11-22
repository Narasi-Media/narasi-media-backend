use crate::domain::entity::{BalanceLog, Category, Deposit, Order, Service, Ticket, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{CategoryId, DepositId, OrderId, ServiceId, TicketId, UserId};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn save_balance_log(&self, log: &BalanceLog) -> Result<(), DomainError>;
    async fn list_balance_logs(&self, user_id: &UserId) -> Result<Vec<BalanceLog>, DomainError>;
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, DomainError>;
    async fn save(&self, order: &Order) -> Result<(), DomainError>;
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Order>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Order>, DomainError>;
    async fn list_active(&self) -> Result<Vec<Order>, DomainError>;
}

#[async_trait]
pub trait ServiceRepository: Send + Sync {
    async fn find_by_id(&self, id: &ServiceId) -> Result<Option<Service>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Service>, DomainError>;
    async fn list_by_category(&self, category_id: &CategoryId) -> Result<Vec<Service>, DomainError>;
    async fn list_categories(&self) -> Result<Vec<Category>, DomainError>;
    async fn find_category_by_id(&self, id: &CategoryId) -> Result<Option<Category>, DomainError>;
    async fn save(&self, service: &Service) -> Result<(), DomainError>;
    async fn delete(&self, id: &ServiceId) -> Result<(), DomainError>;
    async fn save_category(&self, category: &Category) -> Result<(), DomainError>;
    async fn delete_category(&self, id: &CategoryId) -> Result<(), DomainError>;
}

#[async_trait]
pub trait DepositRepository: Send + Sync {
    async fn find_by_id(&self, id: &DepositId) -> Result<Option<Deposit>, DomainError>;
    async fn save(&self, deposit: &Deposit) -> Result<(), DomainError>;
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Deposit>, DomainError>;
    async fn list_pending(&self) -> Result<Vec<Deposit>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Deposit>, DomainError>;
}

#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn find_by_id(&self, id: &TicketId) -> Result<Option<Ticket>, DomainError>;
    async fn save(&self, ticket: &Ticket) -> Result<(), DomainError>;
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Ticket>, DomainError>;
    async fn list_all(&self) -> Result<Vec<Ticket>, DomainError>;
}
