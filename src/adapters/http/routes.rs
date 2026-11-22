use crate::adapters::http::handlers::*;
use crate::AppState;
use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/api/dashboard/:user_id", get(get_dashboard_stats))
        .route("/api/categories", get(list_categories))
        .route("/api/services", get(list_services))
        .route("/api/orders", post(place_order))
        .route("/api/orders/mass", post(mass_order))
        .route("/api/orders/user/:user_id", get(list_user_orders))
        .route("/api/deposits", post(request_deposit))
        .route(
            "/api/deposits/:deposit_id/proof",
            post(submit_deposit_proof),
        )
        // Admin Deposit routes
        .route("/api/admin/deposits/pending", get(admin_list_pending_deposits))
        .route(
            "/api/admin/deposits/:deposit_id/approve",
            post(admin_approve_deposit),
        )
        .route(
            "/api/admin/deposits/:deposit_id/reject",
            post(admin_reject_deposit),
        )
        // Admin Product & Service CRUD
        .route("/api/admin/services", post(admin_create_service))
        .route(
            "/api/admin/services/:id",
            put(admin_update_service).delete(admin_delete_service),
        )
        .route(
            "/api/admin/services/:id/toggle",
            post(admin_toggle_service),
        )
        .route("/api/admin/services/batch-markup", post(admin_batch_markup))
        .route("/api/admin/categories", post(admin_create_category))
        .route(
            "/api/admin/categories/:id",
            put(admin_update_category).delete(admin_delete_category),
        )
        .layer(cors)
        .with_state(state)
}
