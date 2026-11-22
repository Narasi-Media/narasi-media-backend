use buzzer_media_backend::domain::entity::{Deposit, Service, User};
use buzzer_media_backend::domain::errors::DomainError;
use buzzer_media_backend::domain::value_objects::{
    CategoryId, DepositMethod, DepositStatus, Money, ServiceId, ServiceType, UserId,
};

#[test]
fn test_money_calculations() {
    let m1 = Money::from_idr(10_000);
    let m2 = Money::from_idr(5_000);

    assert_eq!((m1 + m2).amount(), 15_000);
    assert_eq!((m1 - m2).amount(), 5_000);
    assert_eq!(m1.format_idr(), "Rp 10.000");

    // Calculate order price: 1.500 qty at Rp 15.000 / 1.000
    // Price = 1.5 * 15.000 = 22.500
    let price_per_k = Money::from_idr(15_000);
    let order_price = Money::calculate_order_price(1_500, price_per_k);
    assert_eq!(order_price.amount(), 22_500);

    // Calculate partial refund:
    // Ordered 1.000 for Rp 20.000. Remains 400.
    // Refund = (400 / 1000) * 20.000 = Rp 8.000
    let refund = Money::calculate_partial_refund(400, 1_000, Money::from_idr(20_000));
    assert_eq!(refund.amount(), 8_000);
}

#[test]
fn test_user_balance_debit_and_credit() {
    let mut user = User::new(
        "tester".to_string(),
        "Tester User".to_string(),
        "test@example.com".to_string(),
        "08123".to_string(),
        "hash".to_string(),
    );

    assert_eq!(user.balance.amount(), 0);

    // Credit Rp 100.000
    let credit_log = user.credit(Money::from_idr(100_000), "Topup".to_string());
    assert_eq!(user.balance.amount(), 100_000);
    assert_eq!(credit_log.initial_balance.amount(), 0);
    assert_eq!(credit_log.final_balance.amount(), 100_000);

    // Debit Rp 35.000
    let debit_log = user
        .debit(Money::from_idr(35_000), "Order #1".to_string())
        .expect("Debit should succeed");
    assert_eq!(user.balance.amount(), 65_000);
    assert_eq!(debit_log.final_balance.amount(), 65_000);

    // Debit more than balance -> should fail
    let err = user
        .debit(Money::from_idr(70_000), "Overcharge".to_string())
        .unwrap_err();
    assert_eq!(
        err,
        DomainError::InsufficientBalance {
            current: 65_000,
            required: 70_000,
        }
    );
}

#[test]
fn test_service_validation() {
    let service = Service {
        id: ServiceId::from_string("SRV-1"),
        category_id: CategoryId::from_string("CAT-1"),
        provider_id: None,
        provider_service_id: None,
        name: "Test Service".to_string(),
        service_type: ServiceType::Default,
        price_original: Money::from_idr(8_000),
        price_user: Money::from_idr(10_000),
        min_order: 100,
        max_order: 10_000,
        refill_supported: true,
        refill_days: 30,
        description: "Desc".to_string(),
        is_active: true,
    };

    // Valid quantity
    let price = service.calculate_price(500).unwrap();
    assert_eq!(price.amount(), 5_000);

    // Below minimum
    let err_min = service.calculate_price(50).unwrap_err();
    assert_eq!(
        err_min,
        DomainError::InvalidOrderQuantity {
            quantity: 50,
            min: 100,
            max: 10_000,
        }
    );

    // Above maximum
    let err_max = service.calculate_price(20_000).unwrap_err();
    assert_eq!(
        err_max,
        DomainError::InvalidOrderQuantity {
            quantity: 20_000,
            min: 100,
            max: 10_000,
        }
    );
}

#[test]
fn test_deposit_manual_qris_lifecycle() {
    let user_id = UserId::new();
    let requested = Money::from_idr(50_000);
    let unique_code = 387;
    let bonus_percentage = 10.0; // 10% bonus

    let mut deposit = Deposit::create_manual(
        user_id,
        DepositMethod::ManualQris,
        requested,
        unique_code,
        bonus_percentage,
    )
    .expect("Deposit creation should succeed");

    assert_eq!(deposit.status, DepositStatus::Pending);
    assert_eq!(deposit.total_to_pay.amount(), 50_387);
    assert_eq!(deposit.balance_credited.amount(), 55_000); // 50.000 + 10%

    // Submit payment proof
    deposit
        .submit_proof("Budi Santoso".to_string(), "/uploads/receipt.jpg".to_string())
        .expect("Submit proof should succeed");
    assert_eq!(deposit.status, DepositStatus::WaitingVerification);
    assert_eq!(deposit.sender_name.as_deref(), Some("Budi Santoso"));

    // Approve
    let credited = deposit.approve().expect("Approval should succeed");
    assert_eq!(deposit.status, DepositStatus::Success);
    assert_eq!(credited.amount(), 55_000);
}
