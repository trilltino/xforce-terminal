//! # Server Setup
//!
//! Server initialization, route registration, and HTTP server startup.
//!
//! This module provides the main server setup function that creates the Axum router,
//! registers all routes, applies middleware, and starts the HTTP server.

// region: --- Imports
use axum::{routing::{get, post}, Router};
use lib_core::{Config, DbPool, create_pool};
use lib_solana::{SolanaState, Network, ContractRegistry, PluginLoader, PriceStreamServer};
use lib_solana::contracts::{
    BatchSwapRouterPlugin, PluginConfig, Cluster, CommitmentLevel, ContractPlugin,
};
use lib_solana::contracts::batch_swap::routes::{
    handle_batch_swap_app_state,
    handle_execute_swap_app_state,
    handle_health_app_state,
    handle_metadata_app_state,
};
use crate::chat::{ChatAppState, handle_braid_subscription, handle_braid_put, handle_typing_event};
use crate::handlers;
use crate::middleware::{stamp_req, log_requests};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
// endregion: --- Imports

// region: --- AppState
/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Config,
    pub solana: Arc<SolanaState>,
    pub contract_registry: Arc<ContractRegistry>,
    pub batch_swap_plugin: Arc<BatchSwapRouterPlugin>,
    pub price_stream: Arc<PriceStreamServer>,
}

impl axum::extract::FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl axum::extract::FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<SolanaState> {
    fn from_ref(state: &AppState) -> Self {
        state.solana.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<ContractRegistry> {
    fn from_ref(state: &AppState) -> Self {
        state.contract_registry.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<BatchSwapRouterPlugin> {
    fn from_ref(state: &AppState) -> Self {
        state.batch_swap_plugin.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<PriceStreamServer> {
    fn from_ref(state: &AppState) -> Self {
        state.price_stream.clone()
    }
}
// endregion: --- AppState

// region: --- Server Configuration
/// Server configuration
pub struct ServerConfig {
    /// Bind address (e.g., "127.0.0.1:3001")
    pub bind_address: String,
    /// Allowed CORS origins
    pub allowed_origins: Vec<String>,
    /// Database migrations path
    pub migrations_path: &'static str,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:3001".to_string(),
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://localhost:3002".to_string(),
                "http://127.0.0.1:3002".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:8080".to_string(),
            ],
            migrations_path: "./migrations",
        }
    }
}
// endregion: --- Server Configuration

// region: --- Server Setup
/// Initialize and start the HTTP server
///
/// # Arguments
///
/// * `config` - Server configuration
///
/// # Returns
///
/// Returns `Ok(())` if the server starts successfully, or an error if initialization fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Configuration loading fails
/// - Database connection fails
/// - Database migrations fail
/// - Solana client initialization fails
/// - Contract plugin initialization fails
/// - Server binding fails
pub async fn start_server(config: ServerConfig) -> anyhow::Result<()> {
    // Configure tracing subscriber with detailed formatting
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    
    let filter = match log_level.as_str() {
        "trace" => tracing_subscriber::EnvFilter::new("trace"),
        "debug" => tracing_subscriber::EnvFilter::new("debug"),
        "info" => tracing_subscriber::EnvFilter::new("info"),
        "warn" => tracing_subscriber::EnvFilter::new("warn"),
        "error" => tracing_subscriber::EnvFilter::new("error"),
        _ => tracing_subscriber::EnvFilter::new("info"),
    };
    
    // Configure subscriber with detailed formatting
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true) // Show module paths
        .with_thread_ids(true) // Show thread IDs
        .with_thread_names(true) // Show thread names
        .with_line_number(true) // Show line numbers
        .with_file(true) // Show file names
        .with_max_level(tracing::Level::TRACE)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
    
    info!(" SOLANA TRADING TERMINAL BACKEND STARTING");
    info!(" Log level: {}", log_level);

    dotenvy::dotenv().ok();

    info!("Loading configuration...");
    let app_config = Config::from_env().map_err(|e| anyhow::anyhow!(e))?;
    app_config.validate().map_err(|e| anyhow::anyhow!(e))?;

    info!("Database URL: {}", app_config.database_url);
    
    // Ensure data directory exists for SQLite database
    if app_config.database_url.starts_with("sqlite:") {
        let db_path = app_config.database_url.strip_prefix("sqlite:").unwrap();
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
                info!("Created database directory: {:?}", parent);
            }
        }
        info!("Database file will be at: {}", db_path);
    }
    
    info!("Connecting to database...");
    let pool = create_pool().await?;

    info!(" Running database migrations from: {}", config.migrations_path);
    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(config.migrations_path)).await?;
    migrator.run(&pool).await?;
    info!(" Migrations complete");

    info!(" Connecting to Solana...");
    let helius_key = std::env::var("HELIUS_API_KEY").ok();
    
    // Determine network from environment (case-insensitive, defaults to mainnet)
    let network = std::env::var("SOLANA_NETWORK")
        .ok()
        .as_deref()
        .map(str::to_lowercase)
        .filter(|s| s == "devnet")
        .map(|_| Network::Devnet)
        .unwrap_or_else(|| {
            info!("MAINNET MODE - Using Solana Mainnet");
            Network::Mainnet
        });
    
    if matches!(network, Network::Devnet) {
        info!("DEVNET MODE - Using Solana Devnet");
    }
    let solana = Arc::new(SolanaState::new(network.clone(), helius_key.clone()).await?);
    info!(" Solana initialized");

    // Initialize contract registry and plugins
    info!(" Initializing contract plugins...");
    let contract_registry = Arc::new(ContractRegistry::new());
    let _plugin_loader = PluginLoader::new(Arc::clone(&contract_registry));
    
    // Get RPC URL for plugin configuration
    let rpc_url = match network {
        Network::Mainnet => {
            helius_key.as_ref()
                .map(|key| format!("https://mainnet.helius-rpc.com/?api-key={}", key))
                .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string())
        }
        Network::Devnet => "https://api.devnet.solana.com".to_string(),
    };
    
    // Create and initialize batch swap router plugin
    let mut batch_swap_plugin = BatchSwapRouterPlugin::new();
    let plugin_config = PluginConfig {
        program_id: ContractPlugin::program_id(&batch_swap_plugin),
        cluster: match network {
            Network::Mainnet => Cluster::Mainnet,
            Network::Devnet => Cluster::Devnet,
        },
        rpc_url: rpc_url.clone(),
        commitment: CommitmentLevel::Confirmed,
        enabled: true,
    };
    
    ContractPlugin::initialize(&mut batch_swap_plugin, plugin_config).await
        .map_err(|e| anyhow::anyhow!("Failed to initialize batch swap plugin: {}", e))?;
    
    // Create Arc for plugin (we'll use this for both registry and routes)
    let batch_swap_plugin_arc = Arc::new(batch_swap_plugin);
    
    // Register plugin in registry (convert Arc<BatchSwapRouterPlugin> to Arc<dyn ContractPlugin>)
    let plugin_trait: Arc<dyn lib_solana::contracts::ContractPlugin> = batch_swap_plugin_arc.clone() as Arc<dyn lib_solana::contracts::ContractPlugin>;
    contract_registry.register(plugin_trait).await
        .map_err(|e| anyhow::anyhow!("Failed to register batch swap plugin: {}", e))?;
    info!(" Batch swap router plugin registered");

    tokio::spawn({
        let cache = solana.price_cache.clone();
        async move {
            cache.start_background_refresh().await;
        }
    });
    info!(" Background price refresh started (10s interval)");

    // Initialize price stream server
    info!(" Initializing price stream server...");
    let price_stream = Arc::new(PriceStreamServer::new(
        Arc::clone(&solana.jupiter),
        500, // 500ms update interval for sub-second updates
    ));
    
    // Start the price stream server in background
    // Note: Even if start() fails, we still add price_stream to AppState
    // so WebSocket connections can be established (they just won't receive updates until it starts)
    let price_stream_clone: Arc<PriceStreamServer> = Arc::clone(&price_stream);
    tokio::spawn(async move {
        info!(" Starting price stream server background task...");
        match price_stream_clone.start().await {
            Ok(_) => {
                info!(" Price stream server started successfully");
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "CRITICAL: Failed to start price stream server: {}. WebSocket connections will work but won't receive price updates until server starts.",
                    e
                );
                // Don't panic - allow WebSocket connections even if stream isn't running
                // The stream will retry loading tokens in the background
            }
        }
    });
    info!(" Price stream server initialized (background task started)");

    // Create chat app state
    let chat_config = app_config.clone();
    let chat_db = pool.clone();
    let chat_state = Arc::new(ChatAppState::new(chat_db, chat_config));

    let state = AppState {
        db: pool,
        config: app_config,
        solana: Arc::clone(&solana),
        contract_registry: Arc::clone(&contract_registry),
        batch_swap_plugin: Arc::clone(&batch_swap_plugin_arc),
        price_stream: Arc::clone(&price_stream),
    };

    // Create router
    let app = create_router(state, chat_state, config.allowed_origins.clone());

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;

    info!(" SERVER READY: http://{}", config.bind_address);
    log_server_info();

    // Use into_make_service_with_connect_info to enable ConnectInfo extraction
    // This is required for WebSocket handlers that need client connection info
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    Ok(())
}

/// Create the main application router with all routes
fn create_router(
    state: AppState,
    chat_state: Arc<ChatAppState>,
    allowed_origins: Vec<String>,
) -> Router {
    use axum::http::{HeaderValue, Method};

    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("subscribe"),
            axum::http::header::HeaderName::from_static("parents"),
            axum::http::header::HeaderName::from_static("version"),
        ]);

    // Create main router with AppState
    // Note: Contract routes are added directly here to avoid state type conflicts when nesting/merging
    info!("[ROUTE SETUP] Registering HTTP routes...");
    let app = Router::new()
        .route("/api/auth/signup", post(handlers::auth::signup))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/wallet-setup/validate", get(handlers::wallet_auth::validate_wallet_setup))
        .route("/api/auth/wallet-setup/complete", post(handlers::wallet_auth::complete_wallet_setup))
        // Also support the frontend's expected path
        .route("/api/wallet/setup/validate", get(handlers::wallet_auth::validate_wallet_setup))
        .route("/api/wallet/setup/complete", post(handlers::wallet_auth::complete_wallet_setup))
        .route("/api/auth/wallet-login", post(handlers::wallet_auth::wallet_login))
        .route("/api/market/prices", get(handlers::market::get_prices))
        .route("/api/market/tokens", get(handlers::market::get_token_list))
        .route("/api/market/candles", get(handlers::market::get_candles))
        .route("/api/ws/prices", get(handlers::websocket::price_stream_websocket))
        .route("/api/wallet/balance", get(handlers::wallet::get_wallet_balance))
        .route("/api/wallet/info", get(handlers::wallet::get_wallet_info))
        .route("/api/wallet/tokens", get(handlers::wallet::get_token_balances))
        .route("/api/transactions", get(handlers::transaction::get_transaction_history))
        .route("/api/transactions/submit", post(handlers::swap::submit_transaction))
        .route("/api/transaction/submit", post(handlers::transaction::submit_transaction))
        .route("/api/staking/info", get(handlers::staking::get_staking_info))
        .route("/api/swap/quote", get(handlers::swap::get_swap_quote))
        .route("/api/swap/execute", post(handlers::swap::execute_swap))
        // Friend management routes
        .route("/api/friends/request", post(handlers::friends::send_friend_request))
        .route("/api/friends/accept/{id}", post(handlers::friends::accept_friend_request))
        .route("/api/friends/reject/{id}", post(handlers::friends::reject_friend_request))
        .route("/api/friends/block/{user_id}", post(handlers::friends::block_user))
        .route("/api/friends", get(handlers::friends::get_friends))
        .route("/api/friends/search", get(handlers::friends::search_users))
        // Contract routes - added directly to avoid state type conflicts
        .route("/api/contracts/contracts", get(handlers::contracts::list_contracts_handler))
        .route("/api/contracts/contracts/{name}", get(handlers::contracts::get_contract_handler))
        .route("/api/contracts/contracts/{name}/health", get(handlers::contracts::health_check_handler))
        .route("/api/contracts/contracts/{name}/metadata", get(handlers::contracts::get_metadata_handler))
        // Batch swap routes - handlers extract Arc<BatchSwapRouterPlugin> from AppState via FromRef
        .route("/api/contracts/batch-swap-router/batch-swap", post(handle_batch_swap_app_state))
        .route("/api/contracts/batch-swap-router/execute-swap", post(handle_execute_swap_app_state))
        .route("/api/contracts/batch-swap-router/health", get(handle_health_app_state))
        .route("/api/contracts/batch-swap-router/metadata", get(handle_metadata_app_state))
        .route("/health", get(|| async { "OK" }))
        .fallback(|| async {
            info!("[404 HANDLER] Unmatched route - returning 404");
            (axum::http::StatusCode::NOT_FOUND, "Route not found")
        })
        .merge(
            Router::new()
                .route("/api/chat/{conversation_id}", get(handle_braid_subscription).put(handle_braid_put))
                .route("/api/chat/{conversation_id}/typing", post(handle_typing_event))
                .with_state(chat_state)
        )
        .with_state(state)
        // Request stamping (adds request ID) - must be first
        .layer(axum::middleware::from_fn(stamp_req))
        // Comprehensive request/response logging
        .layer(axum::middleware::from_fn(log_requests))
        // Tower HTTP trace layer for spans
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let request_id = request
                        .extensions()
                        .get::<crate::middleware::mw_req_stamp::RequestStamp>()
                        .map(|s| s.id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|_request: &axum::http::Request<_>, span: &tracing::Span| {
                    let _enter = span.enter();
                    // Detailed logging is handled by log_requests middleware
                })
                .on_response(|_response: &axum::http::Response<_>, _latency: std::time::Duration, span: &tracing::Span| {
                    let _enter = span.enter();
                    // Detailed logging is handled by log_requests middleware
                })
                .on_failure(|error: tower_http::classify::ServerErrorsFailureClass, latency: std::time::Duration, span: &tracing::Span| {
                    let _enter = span.enter();
                    tracing::error!(
                        error = ?error,
                        latency_ms = latency.as_millis(),
                        "[HTTP FAILURE] Error: {:?}, Latency: {}ms",
                        error,
                        latency.as_millis()
                    );
                })
        )
        .layer(cors);

    app
}

/// Log server information
fn log_server_info() {
    info!("SOLANA MARKET DATA:");
    info!("   • GET  /api/market/prices?symbols=SOL,USDC,JUP");
    info!("   • GET  /api/market/tokens");
    info!(" WALLET:");
    info!("   • GET  /api/wallet/balance?address={{pubkey}}");
    info!("   • GET  /api/wallet/info?address={{pubkey}}");
    info!(" TRANSACTIONS:");
    info!("   • GET  /api/transactions?address={{pubkey}}&limit=10");
    info!(" STAKING:");
    info!("   • GET  /api/staking/info?address={{pubkey}}");
    info!(" SWAP/TRADING:");
    info!("   • GET  /api/swap/quote?inputMint={{mint}}&outputMint={{mint}}&amount={{lamports}}&slippageBps=50");
    info!(" AUTH:");
    info!("   • POST /api/auth/signup");
    info!("   • POST /api/auth/login");
    info!("   • GET  /api/auth/wallet-setup/validate?token={{setup_token}}");
    info!("   • GET  /api/wallet/setup/validate?token={{setup_token}} (alternative path)");
    info!("   • POST /api/auth/wallet-setup/complete");
    info!("   • POST /api/auth/wallet-login");
    info!(" HEALTH:");
    info!("   • GET  /health");
}
// endregion: --- Server Setup

