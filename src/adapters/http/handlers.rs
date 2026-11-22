use crate::adapters::http::dto::{
    AdjustBalanceRequest, ApiResponse, BatchMarkupRequest, CreateCategoryRequest, CreateServiceRequest,
    MassOrderRequest, PlaceOrderRequest, RejectDepositRequest, RequestDepositRequest,
    SubmitDepositProofRequest, UpdateCategoryRequest, UpdateOrderStatusRequest, UpdatePasswordRequest,
    UpdateProfileRequest, UpdateServiceRequest, UserProfileDto,
};
use crate::application::use_cases::{PlaceOrderInput, RequestDepositInput};
use crate::domain::entity::{Category, Service};
use crate::domain::value_objects::{
    CategoryId, DepositId, Money, OrderId, OrderStatus, ServiceId, ServiceType, UserId,
};
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

pub async fn admin_create_service(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateServiceRequest>,
) -> impl IntoResponse {
    let service_id = match payload.id {
        Some(id) if !id.trim().is_empty() => ServiceId::from_string(id),
        _ => {
            let num = {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                rng.gen_range(100..=999)
            };
            ServiceId::from_string(format!("SRV-{}", num))
        }
    };

    let service = Service {
        id: service_id,
        category_id: CategoryId::from_string(payload.category_id),
        provider_id: payload.provider_id,
        provider_service_id: payload.provider_service_id,
        name: payload.name,
        service_type: payload.service_type.unwrap_or(ServiceType::Default),
        price_original: Money::from_idr(payload.price_original),
        price_user: Money::from_idr(payload.price_user),
        min_order: payload.min_order,
        max_order: payload.max_order,
        refill_supported: payload.refill_supported,
        refill_days: payload.refill_days,
        description: payload.description,
        is_active: payload.is_active.unwrap_or(true),
    };

    match state.service_repo.save(&service).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ApiResponse::ok(service, "Layanan baru berhasil ditambahkan")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_update_service(
    State(state): State<Arc<AppState>>,
    Path(service_id_str): Path<String>,
    Json(payload): Json<UpdateServiceRequest>,
) -> impl IntoResponse {
    let service_id = ServiceId::from_string(service_id_str);
    let existing = match state.service_repo.find_by_id(&service_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Layanan tidak ditemukan")),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    };

    let updated = Service {
        id: service_id,
        category_id: CategoryId::from_string(payload.category_id),
        provider_id: payload.provider_id.or(existing.provider_id),
        provider_service_id: payload.provider_service_id.or(existing.provider_service_id),
        name: payload.name,
        service_type: payload.service_type.unwrap_or(existing.service_type),
        price_original: Money::from_idr(payload.price_original),
        price_user: Money::from_idr(payload.price_user),
        min_order: payload.min_order,
        max_order: payload.max_order,
        refill_supported: payload.refill_supported,
        refill_days: payload.refill_days,
        description: payload.description,
        is_active: payload.is_active,
    };

    match state.service_repo.save(&updated).await {
        Ok(_) => Json(ApiResponse::ok(updated, "Layanan berhasil diperbarui")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_delete_service(
    State(state): State<Arc<AppState>>,
    Path(service_id_str): Path<String>,
) -> impl IntoResponse {
    let service_id = ServiceId::from_string(service_id_str);
    match state.service_repo.delete(&service_id).await {
        Ok(_) => Json(ApiResponse::ok((), "Layanan berhasil dihapus")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_toggle_service(
    State(state): State<Arc<AppState>>,
    Path(service_id_str): Path<String>,
) -> impl IntoResponse {
    let service_id = ServiceId::from_string(service_id_str);
    let mut service = match state.service_repo.find_by_id(&service_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Layanan tidak ditemukan")),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    };

    let new_state = service.toggle_active();
    match state.service_repo.save(&service).await {
        Ok(_) => Json(ApiResponse::ok(
            service,
            format!(
                "Status layanan kini {}",
                if new_state { "Aktif (Tampil)" } else { "Nonaktif (Sembunyi)" }
            ),
        ))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_batch_markup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BatchMarkupRequest>,
) -> impl IntoResponse {
    let services = match state.service_repo.list_all().await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    };

    let mut updated_count = 0;
    for mut srv in services {
        let matches_category = match &payload.category_id {
            Some(cat_id) if !cat_id.trim().is_empty() && cat_id != "all" => {
                srv.category_id.as_str() == cat_id.as_str()
            }
            _ => true,
        };

        if matches_category {
            srv.apply_markup_percentage(payload.percentage);
            if let Ok(_) = state.service_repo.save(&srv).await {
                updated_count += 1;
            }
        }
    }

    Json(ApiResponse::ok(
        updated_count,
        format!(
            "Berhasil memperbarui harga {} layanan dengan markup {:.1}%",
            updated_count, payload.percentage
        ),
    ))
    .into_response()
}

pub async fn admin_create_category(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCategoryRequest>,
) -> impl IntoResponse {
    let category_id = match payload.id {
        Some(id) if !id.trim().is_empty() => CategoryId::from_string(id),
        _ => {
            let slug = payload
                .name
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            CategoryId::from_string(format!("cat-{}", slug))
        }
    };

    let category = Category {
        id: category_id,
        name: payload.name,
        icon: payload.icon.unwrap_or_else(|| "tag".to_string()),
        sort_order: payload.sort_order.unwrap_or(99),
    };

    match state.service_repo.save_category(&category).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ApiResponse::ok(category, "Kategori berhasil dibuat")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_update_category(
    State(state): State<Arc<AppState>>,
    Path(category_id_str): Path<String>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> impl IntoResponse {
    let category_id = CategoryId::from_string(category_id_str);
    let category = Category {
        id: category_id,
        name: payload.name,
        icon: payload.icon,
        sort_order: payload.sort_order,
    };

    match state.service_repo.save_category(&category).await {
        Ok(_) => Json(ApiResponse::ok(category, "Kategori berhasil diperbarui")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_delete_category(
    State(state): State<Arc<AppState>>,
    Path(category_id_str): Path<String>,
) -> impl IntoResponse {
    let category_id = CategoryId::from_string(category_id_str);
    match state.service_repo.delete_category(&category_id).await {
        Ok(_) => Json(ApiResponse::ok((), "Kategori berhasil dihapus")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_list_all_orders(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.order_repo.list_all().await {
        Ok(orders) => Json(ApiResponse::ok(orders, "All orders retrieved")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn admin_update_order_status(
    State(state): State<Arc<AppState>>,
    Path(order_id_str): Path<String>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> impl IntoResponse {
    let order_id = OrderId::from_string(order_id_str);
    let mut order = match state.order_repo.find_by_id(&order_id).await {
        Ok(Some(o)) => o,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Order tidak ditemukan")),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    };

    let old_status = order.status;
    let remains = payload.remains.unwrap_or(order.remains);
    let start_count = payload.start_count.unwrap_or(order.start_count);
    order.update_status(payload.status, start_count, remains, payload.error_message);

    // If order was canceled or partial from an active state, calculate and refund to user
    if (order.status == OrderStatus::Canceled || order.status == OrderStatus::Partial)
        && old_status != OrderStatus::Canceled
        && old_status != OrderStatus::Partial
    {
        let refund_amount = order.calculate_partial_refund();
        if refund_amount.amount() > 0 {
            if let Ok(Some(mut user)) = state.user_repo.find_by_id(&order.user_id).await {
                let note = format!("Refund pesanan #{} ({:?})", order.id, order.status);
                let log = user.credit(refund_amount, note);
                let _ = state.user_repo.save(&user).await;
                let _ = state.user_repo.save_balance_log(&log).await;
            }
        }
    }

    match state.order_repo.save(&order).await {
        Ok(_) => Json(ApiResponse::ok(order, "Status pesanan berhasil diperbarui")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let user = match state.user_repo.find_by_username(&username).await {
        Ok(Some(u)) => u,
        _ => {
            let user_id = UserId::from_string(username.clone());
            match state.user_repo.find_by_id(&user_id).await {
                Ok(Some(u)) => u,
                _ => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(ApiResponse::<()>::error("Pengguna tidak ditemukan")),
                    )
                        .into_response()
                }
            }
        }
    };

    let orders = state
        .order_repo
        .list_by_user(&user.id)
        .await
        .unwrap_or_default();
    let deposits = state
        .deposit_repo
        .list_by_user(&user.id)
        .await
        .unwrap_or_default();

    let total_spent = orders.iter().map(|o| o.total_price.amount()).sum::<i64>();
    let total_deposit = deposits
        .iter()
        .filter(|d| d.status == crate::domain::value_objects::DepositStatus::Success)
        .map(|d| d.balance_credited.amount())
        .sum::<i64>();

    let tier = if total_spent > 2_500_000 {
        "VIP Platinum Partner"
    } else if total_spent > 500_000 {
        "VIP Gold Partner"
    } else {
        "Member Reguler"
    };

    let profile = UserProfileDto {
        id: user.id.to_string(),
        username: user.username.clone(),
        full_name: user.full_name.clone(),
        email: user.email.clone(),
        whatsapp_number: user.whatsapp_number.clone(),
        telegram_username: user.telegram_username.clone(),
        role: format!("{:?}", user.role),
        tier: tier.to_string(),
        balance: user.balance.amount(),
        balance_formatted: user.balance.format_idr(),
        total_orders: orders.len(),
        total_spent,
        total_deposit,
        api_key: user
            .api_key
            .unwrap_or_else(|| "bm_live_9f82bc7190d8a43e7b1a20c3ef".to_string()),
        webhook_url: user.webhook_url.clone(),
        two_factor_enabled: user.two_factor_enabled,
        registered_at: user.created_at.format("%d %b %Y").to_string(),
    };

    Json(ApiResponse::ok(profile, "Profil berhasil dimuat")).into_response()
}

pub async fn update_user_profile(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    let mut user = match state.user_repo.find_by_username(&username).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Pengguna tidak ditemukan")),
            )
                .into_response()
        }
    };

    if let Some(name) = payload.full_name {
        user.full_name = name;
    }
    if let Some(email) = payload.email {
        user.email = email;
    }
    if let Some(wa) = payload.whatsapp_number {
        user.whatsapp_number = wa;
    }
    if let Some(tg) = payload.telegram_username {
        user.telegram_username = Some(tg);
    }
    if let Some(webhook) = payload.webhook_url {
        user.webhook_url = Some(webhook);
    }
    if let Some(tfa) = payload.two_factor_enabled {
        user.two_factor_enabled = tfa;
    }
    user.updated_at = chrono::Utc::now();

    match state.user_repo.save(&user).await {
        Ok(_) => Json(ApiResponse::ok(user, "Profil berhasil diperbarui")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn regenerate_api_key(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let mut user = match state.user_repo.find_by_username(&username).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Pengguna tidak ditemukan")),
            )
                .into_response()
        }
    };

    let new_key = format!("bm_live_{}", uuid::Uuid::new_v4().simple());
    user.api_key = Some(new_key.clone());
    user.updated_at = chrono::Utc::now();

    match state.user_repo.save(&user).await {
        Ok(_) => Json(ApiResponse::ok(
            new_key,
            "API Key baru berhasil di-generate",
        ))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn update_password(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> impl IntoResponse {
    let mut user = match state.user_repo.find_by_username(&username).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Pengguna tidak ditemukan")),
            )
                .into_response()
        }
    };

    if payload.new_password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Kata sandi baru minimal 6 karakter",
            )),
        )
            .into_response();
    }

    user.password_hash = format!("hashed_{}", payload.new_password);
    user.updated_at = chrono::Utc::now();

    match state.user_repo.save(&user).await {
        Ok(_) => Json(ApiResponse::ok((), "Kata sandi berhasil diperbarui")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

pub async fn adjust_user_balance(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Json(payload): Json<AdjustBalanceRequest>,
) -> impl IntoResponse {
    let mut user = match state.user_repo.find_by_username(&username).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Pengguna tidak ditemukan")),
            )
                .into_response()
        }
    };

    user.balance = Money::from_idr(payload.amount);
    user.updated_at = chrono::Utc::now();

    match state.user_repo.save(&user).await {
        Ok(_) => Json(ApiResponse::ok(
            serde_json::json!({
                "balance": user.balance.amount(),
                "balance_formatted": user.balance.format_idr(),
            }),
            "Saldo pengguna berhasil disesuaikan",
        ))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}
