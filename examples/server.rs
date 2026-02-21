//! Example HTTP server for rust-sixtysix.

use sixtysix::api::create_router;
use sixtysix::engine::Engine;
use sixtysix::game::SixtySix;
use sixtysix::store::Memory;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");

    let mem = Arc::new(Memory::new());
    let engine = Arc::new(Engine::new(mem));
    engine.register(Arc::new(SixtySix));

    let app = create_router(engine);

    println!("rust-sixtysix server starting | addr={addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
