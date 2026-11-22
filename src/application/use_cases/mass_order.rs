use crate::application::use_cases::place_order::{PlaceOrderInput, PlaceOrderUseCase};
use crate::domain::entity::Order;
use crate::domain::value_objects::{ServiceId, UserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassOrderItemResult {
    pub line: usize,
    pub raw: String,
    pub success: bool,
    pub order: Option<Order>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassOrderResult {
    pub total_lines: usize,
    pub successful_orders: usize,
    pub failed_orders: usize,
    pub items: Vec<MassOrderItemResult>,
}

pub struct MassOrderUseCase {
    place_order: Arc<PlaceOrderUseCase>,
}

impl MassOrderUseCase {
    pub fn new(place_order: Arc<PlaceOrderUseCase>) -> Self {
        Self { place_order }
    }

    pub async fn execute(&self, user_id: UserId, raw_text: &str) -> MassOrderResult {
        let mut items = Vec::new();
        let mut success_count = 0;
        let mut failed_count = 0;

        for (idx, line) in raw_text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                failed_count += 1;
                items.push(MassOrderItemResult {
                    line: idx + 1,
                    raw: trimmed.to_string(),
                    success: false,
                    order: None,
                    error: Some("Format baris harus: ID_Layanan|Target|Jumlah".to_string()),
                });
                continue;
            }

            let service_id = ServiceId::from_string(parts[0]);
            let target = parts[1].to_string();
            let quantity = match parts[2].parse::<u32>() {
                Ok(q) if q > 0 => q,
                _ => {
                    failed_count += 1;
                    items.push(MassOrderItemResult {
                        line: idx + 1,
                        raw: trimmed.to_string(),
                        success: false,
                        order: None,
                        error: Some("Jumlah pesanan harus berupa angka valid".to_string()),
                    });
                    continue;
                }
            };

            let input = PlaceOrderInput {
                user_id: user_id.clone(),
                service_id,
                target,
                quantity,
                custom_comments: None,
            };

            match self.place_order.execute(input).await {
                Ok(order) => {
                    success_count += 1;
                    items.push(MassOrderItemResult {
                        line: idx + 1,
                        raw: trimmed.to_string(),
                        success: true,
                        order: Some(order),
                        error: None,
                    });
                }
                Err(err) => {
                    failed_count += 1;
                    items.push(MassOrderItemResult {
                        line: idx + 1,
                        raw: trimmed.to_string(),
                        success: false,
                        order: None,
                        error: Some(err.to_string()),
                    });
                }
            }
        }

        MassOrderResult {
            total_lines: items.len(),
            successful_orders: success_count,
            failed_orders: failed_count,
            items,
        }
    }
}
