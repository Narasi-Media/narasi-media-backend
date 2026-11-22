use buzzer_media_backend::adapters::http::routes::create_router;
use buzzer_media_backend::AppState;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Buzzer Media Backend Server (VPS / Standalone mode)...");

    let state = Arc::new(AppState::new());
    state.seed_demo_data().await;
    println!("🌱 Demo data seeded (User: 'regiami' with initial balance Rp 500.000)");

    let app = create_router(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("⚡ Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
