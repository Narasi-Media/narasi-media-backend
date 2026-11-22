use buzzer_media_backend::application::use_cases::{
    PlaceOrderInput, RequestDepositInput,
};
use buzzer_media_backend::domain::entity::{Category, Service, User};
use buzzer_media_backend::domain::value_objects::{
    CategoryId, DepositMethod, DepositStatus, Money, OrderStatus, ServiceId, ServiceType,
};
use buzzer_media_backend::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_place_order_use_case_success() {
    let state = Arc::new(AppState::new());

    // Setup User with balance Rp 50.000
    let mut user = User::new(
        "alice".to_string(),
        "Alice Smith".to_string(),
        "alice@mail.com".to_string(),
        "0811".to_string(),
        "pass".to_string(),
    );
    let initial_log = user.credit(Money::from_idr(50_000), "Initial".to_string());
    state.user_repo.save(&user).await.unwrap();
    state.user_repo.save_balance_log(&initial_log).await.unwrap();

    // Setup Service Rp 10.000 / 1.000
    let service = Service {
        id: ServiceId::from_string("SRV-TEST"),
        category_id: CategoryId::from_string("CAT-1"),
        provider_id: None,
        provider_service_id: None,
        name: "Test IG Followers".to_string(),
        service_type: ServiceType::Default,
        price_original: Money::from_idr(8_000),
        price_user: Money::from_idr(10_000),
        min_order: 100,
        max_order: 10_000,
        refill_supported: true,
        refill_days: 30,
        description: "Test".to_string(),
        is_active: true,
    };
    state.service_repo.save(&service).await.unwrap();

    // Place Order for 1.000 quantity -> total price Rp 10.000
    let input = PlaceOrderInput {
        user_id: user.id.clone(),
        service_id: service.id.clone(),
        target: "https://instagram.com/myprofile".to_string(),
        quantity: 1_000,
        custom_comments: None,
    };

    let order = state.place_order_uc.execute(input).await.unwrap();
    assert_eq!(order.total_price.amount(), 10_000);
    assert_eq!(order.status, OrderStatus::Processing);

    // Verify User balance deducted to Rp 40.000
    let updated_user = state.user_repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.balance.amount(), 40_000);

    // Verify balance logs
    let logs = state.user_repo.list_balance_logs(&user.id).await.unwrap();
    assert_eq!(logs.len(), 2); // Initial credit + Order debit
}

#[tokio::test]
async fn test_mass_order_use_case() {
    let state = Arc::new(AppState::new());

    // Setup User with balance Rp 100.000
    let mut user = User::new(
        "bob".to_string(),
        "Bob".to_string(),
        "bob@mail.com".to_string(),
        "0812".to_string(),
        "pass".to_string(),
    );
    user.credit(Money::from_idr(100_000), "Initial".to_string());
    state.user_repo.save(&user).await.unwrap();

    let service = Service {
        id: ServiceId::from_string("SRV-MASS"),
        category_id: CategoryId::from_string("CAT-1"),
        provider_id: None,
        provider_service_id: None,
        name: "Mass Service".to_string(),
        service_type: ServiceType::Default,
        price_original: Money::from_idr(5_000),
        price_user: Money::from_idr(10_000),
        min_order: 100,
        max_order: 5_000,
        refill_supported: false,
        refill_days: 0,
        description: "Desc".to_string(),
        is_active: true,
    };
    state.service_repo.save(&service).await.unwrap();

    let raw_orders = r#"
        SRV-MASS|https://instagram.com/user1|500
        INVALID_FORMAT_LINE
        SRV-MASS|https://instagram.com/user2|1000
        SRV-NONEXISTENT|https://instagram.com/user3|500
    "#;

    let result = state.mass_order_uc.execute(user.id, raw_orders).await;
    assert_eq!(result.total_lines, 4);
    assert_eq!(result.successful_orders, 2);
    assert_eq!(result.failed_orders, 2);
}

#[tokio::test]
async fn test_manual_deposit_request_and_approval() {
    let state = Arc::new(AppState::new());

    let user = User::new(
        "charlie".to_string(),
        "Charlie".to_string(),
        "charlie@mail.com".to_string(),
        "0813".to_string(),
        "pass".to_string(),
    );
    state.user_repo.save(&user).await.unwrap();

    // Request deposit Rp 50.000 via Manual QRIS
    let input = RequestDepositInput {
        user_id: user.id.clone(),
        method: DepositMethod::ManualQris,
        amount: Money::from_idr(50_000),
        bonus_percentage: 10.0, // 10% promo
    };

    let deposit = state.request_deposit_uc.execute(input).await.unwrap();
    assert_eq!(deposit.status, DepositStatus::Pending);
    assert!(deposit.unique_code >= 100 && deposit.unique_code <= 999);
    assert_eq!(deposit.balance_credited.amount(), 55_000);

    // Admin approves deposit
    let approved = state.approve_deposit_uc.execute(&deposit.id).await.unwrap();
    assert_eq!(approved.status, DepositStatus::Success);

    // Verify user balance credited with Rp 55.000
    let updated_user = state.user_repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.balance.amount(), 55_000);
}

#[tokio::test]
async fn test_service_repository_crud() {
    let state = Arc::new(AppState::new());

    // 1. Create Category
    let cat = Category {
        id: CategoryId::from_string("cat-threads"),
        name: "Threads Likes & Reposts".to_string(),
        icon: "threads".to_string(),
        sort_order: 10,
    };
    state.service_repo.save_category(&cat).await.unwrap();

    let found_cat = state.service_repo.find_category_by_id(&cat.id).await.unwrap();
    assert!(found_cat.is_some());
    assert_eq!(found_cat.unwrap().name, "Threads Likes & Reposts");

    // 2. Create Service
    let srv_id = ServiceId::from_string("SRV-THR-1");
    let service = Service {
        id: srv_id.clone(),
        category_id: cat.id.clone(),
        provider_id: Some("prov-test".to_string()),
        provider_service_id: Some("99".to_string()),
        name: "Threads Repost Super Instant".to_string(),
        service_type: ServiceType::Default,
        price_original: Money::from_idr(5_000),
        price_user: Money::from_idr(7_500),
        min_order: 50,
        max_order: 10_000,
        refill_supported: false,
        refill_days: 0,
        description: "Instant repost".to_string(),
        is_active: true,
    };
    state.service_repo.save(&service).await.unwrap();

    // 3. Read Service
    let retrieved = state.service_repo.find_by_id(&srv_id).await.unwrap().unwrap();
    assert_eq!(retrieved.name, "Threads Repost Super Instant");
    assert_eq!(retrieved.price_user.amount(), 7_500);

    // 4. Update Service
    let mut updated = retrieved.clone();
    updated.name = "Threads Repost High Quality".to_string();
    updated.price_user = Money::from_idr(8_000);
    state.service_repo.save(&updated).await.unwrap();

    let after_update = state.service_repo.find_by_id(&srv_id).await.unwrap().unwrap();
    assert_eq!(after_update.name, "Threads Repost High Quality");
    assert_eq!(after_update.price_user.amount(), 8_000);

    // 5. Delete Service
    state.service_repo.delete(&srv_id).await.unwrap();
    let after_delete = state.service_repo.find_by_id(&srv_id).await.unwrap();
    assert!(after_delete.is_none());

    // 6. Delete Category
    state.service_repo.delete_category(&cat.id).await.unwrap();
    let after_cat_delete = state.service_repo.find_category_by_id(&cat.id).await.unwrap();
    assert!(after_cat_delete.is_none());
}
