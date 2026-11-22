use buzzer_media_backend::adapters::supabase::SupabaseClient;
use buzzer_media_backend::domain::entity::{Deposit, Order, Service, User};
use buzzer_media_backend::domain::value_objects::{
    CategoryId, DepositMethod, DepositStatus, Money, OrderStatus, ServiceId, UserId,
};

#[test]
fn test_supabase_client_initialization() {
    let client = SupabaseClient::new(
        "https://testproject.supabase.co/".to_string(),
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.dummy_key".to_string(),
    );
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(
        client.rest_url("users"),
        "https://testproject.supabase.co/rest/v1/users"
    );
    assert_eq!(
        client.rest_url("orders"),
        "https://testproject.supabase.co/rest/v1/orders"
    );
    assert_eq!(
        client.rest_url("services"),
        "https://testproject.supabase.co/rest/v1/services"
    );
}

#[test]
fn test_user_entity_mapping() {
    let mut user = User::new(
        "testreseller".to_string(),
        "Test Reseller Indonesia".to_string(),
        "test@buzzer.media".to_string(),
        "081234567890".to_string(),
        "secure_hash".to_string(),
    );
    user.credit(Money::from_idr(1_000_000), "Deposit awal".to_string());

    assert_eq!(user.balance.amount(), 1_000_000);
    assert!(user.api_key.is_some());
}

#[test]
fn test_service_entity_mapping() {
    let srv = Service {
        id: ServiceId::from_string("SRV-999"),
        category_id: CategoryId::from_string("cat-ig"),
        provider_id: Some("prov-1".to_string()),
        provider_service_id: Some("101".to_string()),
        name: "Instagram Likes High Quality".to_string(),
        service_type: buzzer_media_backend::domain::value_objects::ServiceType::Default,
        price_original: Money::from_idr(5000),
        price_user: Money::from_idr(8500),
        min_order: 100,
        max_order: 50000,
        refill_supported: true,
        refill_days: 30,
        description: "Layanan likes instagram aktif".to_string(),
        is_active: true,
    };

    let calculated = srv.calculate_price(1000).unwrap();
    assert_eq!(calculated.amount(), 8500);
}

#[test]
fn test_order_entity_mapping() {
    let user_id = UserId::new();
    let service_id = ServiceId::from_string("SRV-101");
    let price_per_k = Money::from_idr(15000);

    let order = Order::create(
        user_id.clone(),
        service_id,
        "https://instagram.com/p/viral".to_string(),
        2000,
        price_per_k,
        None,
    )
    .unwrap();

    assert_eq!(order.total_price.amount(), 30000);
    assert_eq!(order.status, OrderStatus::Pending);
}

#[test]
fn test_deposit_entity_mapping() {
    let user_id = UserId::new();
    let deposit = Deposit::create_manual(
        user_id,
        DepositMethod::ManualQris,
        Money::from_idr(100000),
        215,
        0.0, // 0% bonus
    )
    .unwrap();

    assert_eq!(deposit.total_to_pay.amount(), 100215);
    assert_eq!(deposit.balance_credited.amount(), 100000);
    assert_eq!(deposit.status, DepositStatus::Pending);
}
