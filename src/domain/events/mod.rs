use crate::domain::value_objects::{DepositId, Money, OrderId, OrderStatus, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    OrderPlaced {
        order_id: OrderId,
        user_id: UserId,
        total_price: Money,
        occurred_at: DateTime<Utc>,
    },
    OrderStatusUpdated {
        order_id: OrderId,
        old_status: OrderStatus,
        new_status: OrderStatus,
        refund_amount: Money,
        occurred_at: DateTime<Utc>,
    },
    DepositRequested {
        deposit_id: DepositId,
        user_id: UserId,
        total_to_pay: Money,
        occurred_at: DateTime<Utc>,
    },
    DepositApproved {
        deposit_id: DepositId,
        user_id: UserId,
        balance_credited: Money,
        occurred_at: DateTime<Utc>,
    },
}
