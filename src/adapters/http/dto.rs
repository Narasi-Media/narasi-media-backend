use crate::domain::value_objects::DepositMethod;
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
