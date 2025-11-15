//! # Backend Service
//!
//! Thin entry point that delegates to lib-web for server setup.

use lib_web::{start_server, ServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    let config = ServerConfig {
        bind_address: "127.0.0.1:3001".to_string(),
        migrations_path: "migrations",
        ..Default::default()
    };

    start_server(config).await
}

