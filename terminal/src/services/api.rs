use shared::{AuthResponse, ErrorResponse, LoginRequest, SignupRequest};
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "http://127.0.0.1:3001";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectorPrice {
    pub price: f64,
    pub price_raw: String,
    pub timestamp: u64,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectorPricesResponse {
    pub success: bool,
    pub prices: std::collections::HashMap<String, ReflectorPrice>,
    pub timestamp: u64,
    pub oracle_contract: String,
}

pub struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn login(&self, email_or_username: String, password: String) -> Result<AuthResponse, String> {
        let request = LoginRequest {
            email_or_username,
            password,
        };

        let response = self
            .client
            .post(format!("{}/api/auth/login", API_BASE_URL))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn signup(&self, username: String, email: String, password: String) -> Result<AuthResponse, String> {
        let request = SignupRequest {
            username,
            email,
            password,
        };

        let response = self
            .client
            .post(format!("{}/api/auth/signup", API_BASE_URL))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn get_reflector_prices(&self) -> Result<ReflectorPricesResponse, String> {
        let response = self
            .client
            .get(format!("{}/api/market/reflector/prices", API_BASE_URL))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.status().is_success() {
            response
                .json::<ReflectorPricesResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err(format!("Failed to fetch prices: {}", response.status()))
        }
    }
}
