use axum::{routing::{get, post}, Router};
use backend::{database::create_pool, handlers, oracle_cache::OracleCache, stellar::HorizonClient, Config, DbPool};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    db: DbPool,
    config: Config,
    horizon: Arc<HorizonClient>,
    oracle_cache: Arc<OracleCache>,
}

// Implement FromRef to allow extracting individual components from AppState
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

impl axum::extract::FromRef<AppState> for Arc<HorizonClient> {
    fn from_ref(state: &AppState) -> Self {
        state.horizon.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<OracleCache> {
    fn from_ref(state: &AppState) -> Self {
        state.oracle_cache.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("═══════════════════════════════════════════════════════════");
    info!("🚀 GTK4 TRADING TERMINAL BACKEND STARTING");
    info!("═══════════════════════════════════════════════════════════");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load and validate configuration
    info!("📋 Loading configuration...");
    let config = Config::from_env().map_err(|e| anyhow::anyhow!(e))?;
    config.validate().map_err(|e| anyhow::anyhow!(e))?;

    // Create database pool
    info!("🗄️  Connecting to database...");
    let pool = create_pool().await?;

    // Run migrations
    info!("⚙️  Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("✅ Migrations complete");

    // Create Horizon client for Stellar blockchain data
    info!("🌟 Connecting to Stellar Horizon...");
    let horizon = Arc::new(HorizonClient::new(false)); // mainnet
    info!("✅ Horizon client ready");

    // Create Reflector Oracle cache with background refresh
    info!("🔮 Initializing Reflector Oracle cache...");
    let oracle_cache = OracleCache::new();
    oracle_cache.clone().start_background_refresh(10); // Refresh every 10 seconds
    let oracle_cache = Arc::new(oracle_cache);
    info!("✅ Oracle cache initialized (refreshes every 10s)");

    // Create unified app state
    let state = AppState {
        db: pool,
        config,
        horizon,
        oracle_cache,
    };

    // Configure CORS
    use axum::http::{HeaderValue, Method};

    const ALLOWED_ORIGINS: &[&str] = &["http://localhost:3000", "http://127.0.0.1:3000"];

    let origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    // Build router
    let app = Router::new()
        // Auth endpoints
        .route("/api/auth/signup", post(handlers::signup))
        .route("/api/auth/login", post(handlers::login))

        // Market data endpoints (real Stellar blockchain data!)
        .route("/api/market/xlm/history", get(handlers::get_xlm_price_history))
        .route("/api/market/xlm/price", get(handlers::get_xlm_current_price))
        .route("/api/market/xlm/orderbook", get(handlers::get_xlm_orderbook))

        // Soroban endpoints (Reflector Oracle price feeds)
        .route("/api/soroban/call-function", post(handlers::call_contract_function))
        .route("/api/market/reflector/prices", get(handlers::get_reflector_prices))

        // Health check
        .route("/health", get(|| async { "OK" }))

        .with_state(state)
        .layer(cors);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;

    info!("═══════════════════════════════════════════════════════════");
    info!("✅ SERVER READY ON http://127.0.0.1:3001");
    info!("═══════════════════════════════════════════════════════════");
    info!("");
    info!("📡 API ENDPOINTS:");
    info!("");
    info!("🔐 AUTH:");
    info!("   • POST /api/auth/signup - Create new account");
    info!("   • POST /api/auth/login  - Login to account");
    info!("");
    info!("📊 MARKET DATA (Live Stellar Blockchain):");
    info!("   • GET  /api/market/xlm/history      - XLM price history (30 points)");
    info!("   • GET  /api/market/xlm/price        - Current XLM price");
    info!("   • GET  /api/market/xlm/orderbook    - XLM/USDC orderbook");
    info!("   • GET  /api/market/reflector/prices - All Reflector Oracle prices");
    info!("");
    info!("🔮 SOROBAN (Reflector Oracle):");
    info!("   • POST /api/soroban/call-function  - Call contract function");
    info!("");
    info!("💚 HEALTH:");
    info!("   • GET  /health - Health check");
    info!("");
    info!("═══════════════════════════════════════════════════════════");

    axum::serve(listener, app).await?;
    Ok(())
}
