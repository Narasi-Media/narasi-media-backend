pub mod adapters;
pub mod application;
pub mod domain;

use crate::adapters::in_memory::{
    InMemoryDepositRepository, InMemoryOrderRepository, InMemoryServiceRepository,
    InMemoryTicketRepository, InMemoryUserRepository, MockSmmProviderGateway,
};
use crate::adapters::supabase::{
    SupabaseClient, SupabaseDepositRepository, SupabaseOrderRepository,
    SupabaseServiceRepository, SupabaseUserRepository,
};
use crate::application::ports::{
    DepositRepository, OrderRepository, ServiceRepository, SmmProviderGateway, TicketRepository,
    UserRepository,
};
use crate::application::use_cases::{
    ApproveDepositUseCase, ClaimRefillUseCase, MassOrderUseCase, PlaceOrderUseCase,
    RequestDepositUseCase, SyncOrderUseCase,
};
use crate::domain::entity::{Category, Order, Service, User};
use crate::domain::value_objects::{CategoryId, Money, ServiceId, ServiceType};
use std::sync::Arc;

pub struct AppState {
    pub user_repo: Arc<dyn UserRepository>,
    pub order_repo: Arc<dyn OrderRepository>,
    pub service_repo: Arc<dyn ServiceRepository>,
    pub deposit_repo: Arc<dyn DepositRepository>,
    pub ticket_repo: Arc<dyn TicketRepository>,
    pub provider_gateway: Arc<dyn SmmProviderGateway>,

    // Use cases
    pub place_order_uc: Arc<PlaceOrderUseCase>,
    pub mass_order_uc: Arc<MassOrderUseCase>,
    pub request_deposit_uc: Arc<RequestDepositUseCase>,
    pub approve_deposit_uc: Arc<ApproveDepositUseCase>,
    pub sync_order_uc: Arc<SyncOrderUseCase>,
    pub claim_refill_uc: Arc<ClaimRefillUseCase>,
}

impl AppState {
    pub fn new() -> Self {
        let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
        let order_repo: Arc<dyn OrderRepository> = Arc::new(InMemoryOrderRepository::new());
        let service_repo: Arc<dyn ServiceRepository> = Arc::new(InMemoryServiceRepository::new());
        let deposit_repo: Arc<dyn DepositRepository> = Arc::new(InMemoryDepositRepository::new());
        let ticket_repo: Arc<dyn TicketRepository> = Arc::new(InMemoryTicketRepository::new());
        let provider_gateway: Arc<dyn SmmProviderGateway> = Arc::new(MockSmmProviderGateway);

        let place_order_uc = Arc::new(PlaceOrderUseCase::new(
            user_repo.clone(),
            order_repo.clone(),
            service_repo.clone(),
            Some(provider_gateway.clone()),
        ));

        let mass_order_uc = Arc::new(MassOrderUseCase::new(place_order_uc.clone()));

        let request_deposit_uc = Arc::new(RequestDepositUseCase::new(deposit_repo.clone()));

        let approve_deposit_uc = Arc::new(ApproveDepositUseCase::new(
            deposit_repo.clone(),
            user_repo.clone(),
        ));

        let sync_order_uc = Arc::new(SyncOrderUseCase::new(
            order_repo.clone(),
            user_repo.clone(),
            provider_gateway.clone(),
        ));

        let claim_refill_uc = Arc::new(ClaimRefillUseCase::new(
            order_repo.clone(),
            service_repo.clone(),
            provider_gateway.clone(),
        ));

        Self {
            user_repo,
            order_repo,
            service_repo,
            deposit_repo,
            ticket_repo,
            provider_gateway,
            place_order_uc,
            mass_order_uc,
            request_deposit_uc,
            approve_deposit_uc,
            sync_order_uc,
            claim_refill_uc,
        }
    }

    pub fn with_supabase(client: SupabaseClient) -> Self {
        let client_arc = Arc::new(client);
        let user_repo: Arc<dyn UserRepository> = Arc::new(SupabaseUserRepository::new(client_arc.clone()));
        let order_repo: Arc<dyn OrderRepository> = Arc::new(SupabaseOrderRepository::new(client_arc.clone()));
        let service_repo: Arc<dyn ServiceRepository> = Arc::new(SupabaseServiceRepository::new(client_arc.clone()));
        let deposit_repo: Arc<dyn DepositRepository> = Arc::new(SupabaseDepositRepository::new(client_arc.clone()));
        let ticket_repo: Arc<dyn TicketRepository> = Arc::new(InMemoryTicketRepository::new());
        let provider_gateway: Arc<dyn SmmProviderGateway> = Arc::new(MockSmmProviderGateway);

        let place_order_uc = Arc::new(PlaceOrderUseCase::new(
            user_repo.clone(),
            order_repo.clone(),
            service_repo.clone(),
            Some(provider_gateway.clone()),
        ));

        let mass_order_uc = Arc::new(MassOrderUseCase::new(place_order_uc.clone()));
        let request_deposit_uc = Arc::new(RequestDepositUseCase::new(deposit_repo.clone()));
        let approve_deposit_uc = Arc::new(ApproveDepositUseCase::new(
            deposit_repo.clone(),
            user_repo.clone(),
        ));
        let sync_order_uc = Arc::new(SyncOrderUseCase::new(
            order_repo.clone(),
            user_repo.clone(),
            provider_gateway.clone(),
        ));
        let claim_refill_uc = Arc::new(ClaimRefillUseCase::new(
            order_repo.clone(),
            service_repo.clone(),
            provider_gateway.clone(),
        ));

        Self {
            user_repo,
            order_repo,
            service_repo,
            deposit_repo,
            ticket_repo,
            provider_gateway,
            place_order_uc,
            mass_order_uc,
            request_deposit_uc,
            approve_deposit_uc,
            sync_order_uc,
            claim_refill_uc,
        }
    }

    pub fn from_env() -> Self {
        if let Some(supabase_client) = SupabaseClient::from_env() {
            println!("⚡ [Adapter] Terhubung ke Supabase Cloud REST Database!");
            Self::with_supabase(supabase_client)
        } else {
            println!("💾 [Adapter] Menggunakan In-Memory DDD Repository (Lokal/Dev mode)");
            Self::new()
        }
    }

    pub async fn seed_demo_data(&self) {
        // Create demo user 'regiami'
        let mut demo_user = User::new(
            "regiami".to_string(),
            "Regi Ami".to_string(),
            "regi@narasi.media".to_string(),
            "081234567890".to_string(),
            "hashed_pass".to_string(),
        );
        demo_user.credit(
            Money::from_idr(500_000),
            "Saldo Awal Akun".to_string(),
        );
        let _ = self.user_repo.save(&demo_user).await;

        // Categories
        let cat_ig = Category {
            id: CategoryId::from_string("cat-ig"),
            name: "Instagram Followers & Likes".to_string(),
            icon: "instagram".to_string(),
            sort_order: 1,
        };
        let cat_tiktok = Category {
            id: CategoryId::from_string("cat-tiktok"),
            name: "TikTok Followers, Likes & Views".to_string(),
            icon: "tiktok".to_string(),
            sort_order: 2,
        };
        let cat_yt = Category {
            id: CategoryId::from_string("cat-yt"),
            name: "YouTube Subscribers & Views".to_string(),
            icon: "youtube".to_string(),
            sort_order: 3,
        };
        let cat_tw = Category {
            id: CategoryId::from_string("cat-tw"),
            name: "Twitter / X Followers & Retweet".to_string(),
            icon: "twitter".to_string(),
            sort_order: 4,
        };

        let _ = self.service_repo.save_category(&cat_ig).await;
        let _ = self.service_repo.save_category(&cat_tiktok).await;
        let _ = self.service_repo.save_category(&cat_yt).await;
        let _ = self.service_repo.save_category(&cat_tw).await;

        // Services
        let services = vec![
            Service {
                id: ServiceId::from_string("SRV-101"),
                category_id: cat_ig.id.clone(),
                provider_id: Some("prov-1".to_string()),
                provider_service_id: Some("101".to_string()),
                name: "Instagram Followers REAL Indonesia [Maks 50K] [Garansi 30 Hari]".to_string(),
                service_type: ServiceType::Default,
                price_original: Money::from_idr(12_000),
                price_user: Money::from_idr(15_500),
                min_order: 100,
                max_order: 50_000,
                refill_supported: true,
                refill_days: 30,
                description: "Akun real aktif Indonesia. Kecepatan 1.000 - 5.000 / hari. Garansi tombol Refill 30 hari jika drop > 10%.".to_string(),
                is_active: true,
            },
            Service {
                id: ServiceId::from_string("SRV-102"),
                category_id: cat_ig.id.clone(),
                provider_id: Some("prov-1".to_string()),
                provider_service_id: Some("102".to_string()),
                name: "Instagram Likes Super Instant [Maks 100K] [No Drop]".to_string(),
                service_type: ServiceType::Default,
                price_original: Money::from_idr(3_000),
                price_user: Money::from_idr(4_500),
                min_order: 50,
                max_order: 100_000,
                refill_supported: false,
                refill_days: 0,
                description: "Proses instan 5 - 10 menit setelah submit. Masukkan link postingan valid.".to_string(),
                is_active: true,
            },
            Service {
                id: ServiceId::from_string("SRV-201"),
                category_id: cat_tiktok.id.clone(),
                provider_id: Some("prov-1".to_string()),
                provider_service_id: Some("201".to_string()),
                name: "TikTok Followers Organik HQ [Garansi 30 Hari]".to_string(),
                service_type: ServiceType::Default,
                price_original: Money::from_idr(18_000),
                price_user: Money::from_idr(22_000),
                min_order: 100,
                max_order: 25_000,
                refill_supported: true,
                refill_days: 30,
                description: "Followers TikTok kualitas tinggi dengan foto profil dan video.".to_string(),
                is_active: true,
            },
            Service {
                id: ServiceId::from_string("SRV-301"),
                category_id: cat_yt.id.clone(),
                provider_id: Some("prov-1".to_string()),
                provider_service_id: Some("301".to_string()),
                name: "YouTube Subscribers Non-Drop [Garansi Seumur Hidup]".to_string(),
                service_type: ServiceType::Default,
                price_original: Money::from_idr(75_000),
                price_user: Money::from_idr(95_000),
                min_order: 50,
                max_order: 10_000,
                refill_supported: true,
                refill_days: 365,
                description: "Subscribers aman untuk monetisasi YouTube. Kecepatan 100 - 300 / hari.".to_string(),
                is_active: true,
            },
        ];

        for srv in services {
            let _ = self.service_repo.save(&srv).await;
        }

        // Seed Demo Orders
        if let Ok(mut order1) = Order::create(
            demo_user.id.clone(),
            ServiceId::from_string("SRV-101"),
            "https://instagram.com/fashion_indo".to_string(),
            1_000,
            Money::from_idr(15_500),
            None,
        ) {
            order1.set_processing("SMM-98214".to_string());
            let _ = self.order_repo.save(&order1).await;
        }

        if let Ok(mut order2) = Order::create(
            demo_user.id.clone(),
            ServiceId::from_string("SRV-201"),
            "https://tiktok.com/@selebtok_id".to_string(),
            500,
            Money::from_idr(22_000),
            None,
        ) {
            order2.set_processing("SMM-98215".to_string());
            order2.update_status(
                crate::domain::value_objects::OrderStatus::Success,
                1_200,
                0,
                None,
            );
            let _ = self.order_repo.save(&order2).await;
        }

        if let Ok(order3) = Order::create(
            demo_user.id.clone(),
            ServiceId::from_string("SRV-102"),
            "https://instagram.com/p/DF1289x".to_string(),
            2_000,
            Money::from_idr(4_500),
            None,
        ) {
            let _ = self.order_repo.save(&order3).await;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
