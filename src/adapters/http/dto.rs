use crate::domain::value_objects::{DepositMethod, OrderStatus, ServiceType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub user_id: String,
    pub service_id: String,
    pub target: String,
    pub quantity: u32,
    pub custom_comments: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct MassOrderRequest {
    pub user_id: String,
    pub orders_raw: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestDepositRequest {
    pub user_id: String,
    pub method: DepositMethod,
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct SubmitDepositProofRequest {
    pub sender_name: String,
    pub proof_image_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RejectDepositRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub id: Option<String>,
    pub category_id: String,
    pub name: String,
    pub service_type: Option<ServiceType>,
    pub price_original: i64,
    pub price_user: i64,
    pub min_order: u32,
    pub max_order: u32,
    pub refill_supported: bool,
    pub refill_days: u32,
    pub description: String,
    pub provider_id: Option<String>,
    pub provider_service_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceRequest {
    pub category_id: String,
    pub name: String,
    pub service_type: Option<ServiceType>,
    pub price_original: i64,
    pub price_user: i64,
    pub min_order: u32,
    pub max_order: u32,
    pub refill_supported: bool,
    pub refill_days: u32,
    pub description: String,
    pub provider_id: Option<String>,
    pub provider_service_id: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct BatchMarkupRequest {
    pub percentage: f64,
    pub category_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
    pub icon: String,
    pub sort_order: u32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
    pub remains: Option<u32>,
    pub start_count: Option<u32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileDto {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub whatsapp_number: String,
    pub telegram_username: Option<String>,
    pub role: String,
    pub tier: String,
    pub balance: i64,
    pub balance_formatted: String,
    pub total_orders: usize,
    pub total_spent: i64,
    pub total_deposit: i64,
    pub api_key: String,
    pub webhook_url: Option<String>,
    pub two_factor_enabled: bool,
    pub registered_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub whatsapp_number: Option<String>,
    pub telegram_username: Option<String>,
    pub webhook_url: Option<String>,
    pub two_factor_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
}
