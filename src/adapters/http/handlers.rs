use crate::adapters::http::dto::{
    ApiResponse, MassOrderRequest, PlaceOrderRequest, RejectDepositRequest, RequestDepositRequest,
    SubmitDepositProofRequest,
};
use crate::application::use_cases::{PlaceOrderInput, RequestDepositInput};
use crate::domain::value_objects::{DepositId, Money, ServiceId, UserId};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::ok("Buzzer Media Backend is running", "Healthy"))
}

pub async fn get_dashboard_stats(
    State(state): State<Arc<AppState>>,
    Path(user_id_str): Path<String>,
) -> impl IntoResponse {
    let user_id = UserId::from_string(user_id_str);
    let user = match state.user_repo.find_by_id(&user_id).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("User not found")),
            )
                .into_response()
        }
    };

    let orders = state
        .order_repo
        .list_by_user(&user_id)
        .await
        .unwrap_or_default();
    let deposits = state
        .deposit_repo
        .list_by_user(&user_id)
        .await
        .unwrap_or_default();

    let total_spent = orders.iter().map(|o| o.total_price.amount()).sum::<i64>();
    let total_deposit = deposits
        .iter()
        .filter(|d| d.status == crate::domain::value_objects::DepositStatus::Success)
        .map(|d| d.balance_credited.amount())
        .sum::<i64>();

    let stats = serde_json::json!({
        "balance": user.balance.amount(),
        "balance_formatted": user.balance.format_idr(),
        "total_orders": orders.len(),
        "total_spent": total_spent,
        "total_deposit": total_deposit,
        "username": user.username,
        "full_name": user.full_name,
    });

    Json(ApiResponse::ok(stats, "Stats retrieved")).into_response()
}

pub async fn list_categories(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.service_repo.list_categories().await {
        Ok(cats) => Json(ApiResponse::ok(cats, "Categories retrieved")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_services(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.service_repo.list_all().await {
        Ok(services) => Json(ApiResponse::ok(services, "Services retrieved")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn place_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PlaceOrderRequest>,
) -> impl IntoResponse {
    let input = PlaceOrderInput {
        user_id: UserId::from_string(payload.user_id),
        service_id: ServiceId::from_string(payload.service_id),
        target: payload.target,
        quantity: payload.quantity,
        custom_comments: payload.custom_comments,
    };

    match state.place_order_uc.execute(input).await {
        Ok(order) => (
            StatusCode::CREATED,
            Json(ApiResponse::ok(order, "Pesanan berhasil dibuat")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn mass_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MassOrderRequest>,
) -> impl IntoResponse {
    let user_id = UserId::from_string(payload.user_id);
    let result = state
        .mass_order_uc
        .execute(user_id, &payload.orders_raw)
        .await;

    Json(ApiResponse::ok(result, "Proses pesanan masal selesai")).into_response()
}

pub async fn list_user_orders(
    State(state): State<Arc<AppState>>,
    Path(user_id_str): Path<String>,
) -> impl IntoResponse {
    let user_id = UserId::from_string(user_id_str);
    match state.order_repo.list_by_user(&user_id).await {
        Ok(orders) => Json(ApiResponse::ok(orders, "Orders retrieved")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn request_deposit(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestDepositRequest>,
) -> impl IntoResponse {
    let input = RequestDepositInput {
        user_id: UserId::from_string(payload.user_id),
        method: payload.method,
        amount: Money::from_idr(payload.amount),
        bonus_percentage: 5.0, // Default 5% promo bonus
    };

    match state.request_deposit_uc.execute(input).await {
        Ok(deposit) => (
            StatusCode::CREATED,
            Json(ApiResponse::ok(deposit, "Tiket deposit berhasil dibuat")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn submit_deposit_proof(
    State(state): State<Arc<AppState>>,
    Path(deposit_id_str): Path<String>,
    Json(payload): Json<SubmitDepositProofRequest>,
) -> impl IntoResponse {
    let deposit_id = DepositId::from_string(deposit_id_str);
    let mut deposit = match state.deposit_repo.find_by_id(&deposit_id).await {
        Ok(Some(d)) => d,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Deposit tidak ditemukan")),
            )
                .into_response()
        }
    };

    match deposit.submit_proof(payload.sender_name, payload.proof_image_url) {
        Ok(_) => {
            let _ = state.deposit_repo.save(&deposit).await;
            Json(ApiResponse::ok(
                deposit,
                "Bukti pembayaran berhasil diunggah, menunggu verifikasi admin",
            ))
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_list_pending_deposits(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.deposit_repo.list_pending().await {
        Ok(deps) => Json(ApiResponse::ok(deps, "Pending deposits retrieved")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_approve_deposit(
    State(state): State<Arc<AppState>>,
    Path(deposit_id_str): Path<String>,
) -> impl IntoResponse {
    let deposit_id = DepositId::from_string(deposit_id_str);
    match state.approve_deposit_uc.execute(&deposit_id).await {
        Ok(deposit) => Json(ApiResponse::ok(
            deposit,
            "Deposit disetujui, saldo telah dikreditkan",
        ))
        .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_reject_deposit(
    State(state): State<Arc<AppState>>,
    Path(deposit_id_str): Path<String>,
    Json(payload): Json<RejectDepositRequest>,
) -> impl IntoResponse {
    let deposit_id = DepositId::from_string(deposit_id_str);
    match state.approve_deposit_uc.reject(&deposit_id, payload.reason).await {
        Ok(deposit) => Json(ApiResponse::ok(deposit, "Deposit ditolak")).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}
