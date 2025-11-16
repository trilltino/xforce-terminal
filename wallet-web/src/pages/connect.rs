//! Connect Wallet Page - Multi-wallet connection for user signup
//! Split-screen layout with left section (header/subheader) and right section (wallet card)

use leptos::prelude::*;
use leptos::logging::log;
use leptos_router::hooks::use_query_map;
use crate::services::wallet::{
    WalletProvider, 
    get_available_wallets, 
    connect_wallet_provider, 
    sign_message_provider,
};
use crate::state::wallet::use_wallet_context;
use crate::utils::url::get_query_param;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use js_sys::Uint8Array;
use bs58;

#[derive(Serialize)]
struct WalletSetupCompleteRequest {
    setup_token: String,
    wallet_address: String,
    signature: String,
    challenge: String,
}

#[derive(Deserialize)]
struct WalletSetupValidateResponse {
    valid: bool,
    username: String,
    challenge: String,
}

#[derive(Deserialize)]
struct WalletSetupCompleteResponse {
    success: bool,
    message: String,
}

#[component]
pub fn ConnectPage() -> impl IntoView {
    let wallet_ctx = use_wallet_context();
    let query = use_query_map();

    let (error, set_error) = signal(None::<String>);
    let (connecting, set_connecting) = signal(false);
    let (setup_complete, set_setup_complete) = signal(false);
    let (username, set_username) = signal(None::<String>);
    let (available_wallets, set_available_wallets) = signal::<Vec<WalletProvider>>(vec![]);

    // Get setup_token from URL query params
    // Try router's query map first, then fall back to direct URL reading
    let setup_token = move || {
        // First try the router's query map
        let router_token = query.with(|params| {
            params.get("token").or_else(|| params.get("setup_token"))
        });
        
        // If not found in router, try reading directly from URL
        router_token.or_else(|| {
            get_query_param("token").or_else(|| get_query_param("setup_token"))
        })
    };

    // Detect available wallets and auto-validate token on mount
    leptos::task::spawn_local(async move {
        let wallets = get_available_wallets();
        let providers: Vec<WalletProvider> = wallets
            .iter()
            .filter_map(|w| {
                match w.provider.as_str() {
                    "phantom" => Some(WalletProvider::Phantom),
                    "solflare" => Some(WalletProvider::Solflare),
                    "backpack" => Some(WalletProvider::Backpack),
                    "sollet" => Some(WalletProvider::Sollet),
                    _ => None,
                }
            })
            .collect();
        set_available_wallets.set(providers);

        // Auto-validate token if present to show username immediately
        // Try router's query map first, then fall back to direct URL reading
        let token = query.with(|params| {
            params.get("token").or_else(|| params.get("setup_token"))
        }).or_else(|| {
            get_query_param("token").or_else(|| get_query_param("setup_token"))
        });
        if let Some(token_val) = token {
            log!("[AUTO-VALIDATE] Found token in URL, attempting auto-validation...");
            log!("[AUTO-VALIDATE] Token: {}...", &token_val[..token_val.len().min(20)]);
            let validate_url = format!("http://localhost:3001/api/wallet/setup/validate?token={}", token_val);
            log!("[AUTO-VALIDATE] URL: {}", validate_url);
            log!("[AUTO-VALIDATE] Sending GET request...");
            match Request::get(&validate_url).send().await {
                Ok(resp) => {
                    log!("[AUTO-VALIDATE] Response received: status={}, ok={}", resp.status(), resp.ok());
                    if resp.ok() {
                        log!("[AUTO-VALIDATE] Response OK, parsing JSON...");
                        match resp.json::<WalletSetupValidateResponse>().await {
                            Ok(validate_data) => {
                                log!("[AUTO-VALIDATE] Auto-validation successful: user={}", validate_data.username);
                                set_username.set(Some(validate_data.username));
                            }
                            Err(e) => {
                                log!("[AUTO-VALIDATE] Failed to parse JSON: {:?}", e);
                            }
                        }
                    } else {
                        log!("[AUTO-VALIDATE] Response not OK: status={}", resp.status());
                    }
                }
                Err(e) => {
                    log!("[AUTO-VALIDATE] Request failed: {:?}", e);
                    log!("[AUTO-VALIDATE] This might indicate backend is not running");
                }
            }
        } else {
            log!("[AUTO-VALIDATE] No token found in URL");
        }
    });

    let connect_wallet_placeholder = move |provider: WalletProvider| {
        // Placeholder - just for screenshot, no actual connection
        log!("Wallet connection clicked: {}", provider.name());
    };

    let connect_wallet = move |provider: WalletProvider| {
        // Placeholder - just for screenshot, no actual connection
        log!("Wallet connection clicked: {}", provider.name());
        // Original connection code disabled for screenshot
        /*
        set_connecting.set(true);
        set_error.set(None);

        let token = setup_token();
        if token.is_none() {
            set_error.set(Some("Missing setup token. Please signup first in the terminal.".to_string()));
            set_connecting.set(false);
            return;
        }
        let token = token.unwrap();
        let provider_clone = provider.clone();

        leptos::task::spawn_local(async move {
            // Step 1: Validate setup token with backend
            log!("[WALLET CONNECT] ========== Starting wallet connection flow ==========");
            log!("[WALLET CONNECT] Provider: {}", provider_clone.name());
            log!("[WALLET CONNECT] Setup token length: {}", token.len());
            log!("[WALLET CONNECT] Setup token: {}...", &token[..token.len().min(20)]);
            
            let validate_url = format!("http://localhost:3001/api/wallet/setup/validate?token={}", token);
            log!("[WALLET CONNECT] Step 1: Validating setup token");
            log!("[WALLET CONNECT] Request URL: {}", validate_url);
            log!("[WALLET CONNECT] Sending GET request to backend...");

            let validate_response = match Request::get(&validate_url).send().await {
                Ok(resp) => {
                    let status_code = resp.status();
                    log!("[WALLET CONNECT] HTTP request sent successfully");
                    log!("[WALLET CONNECT] Response status: {}", status_code);
                    log!("[WALLET CONNECT] Response status code: {}", status_code);
                    log!("[WALLET CONNECT] Response OK: {}", resp.ok());
                    resp
                },
                Err(e) => {
                    log!("[WALLET CONNECT] HTTP REQUEST FAILED");
                    log!("[WALLET CONNECT] Error type: {:?}", e);
                    log!("[WALLET CONNECT] Error details: {:?}", e);
                    log!("[WALLET CONNECT] URL attempted: {}", validate_url);
                    log!("[WALLET CONNECT] This usually means:");
                    log!("[WALLET CONNECT]   1. Backend server is not running on port 3001");
                    log!("[WALLET CONNECT]   2. CORS is blocking the request");
                    log!("[WALLET CONNECT]   3. Network connectivity issue");
                    set_error.set(Some(format!("Failed to connect to backend: {:?}. Is the server running on http://localhost:3001?", e)));
                    set_connecting.set(false);
                    return;
                }
            };

            let status_code = validate_response.status();
            let status_text = status_code.to_string();
            log!("[WALLET CONNECT] Response received:");
            log!("[WALLET CONNECT]   Status Code: {}", status_code);
            log!("[WALLET CONNECT]   Status Text: {}", status_text);
            log!("[WALLET CONNECT]   OK: {}", validate_response.ok());

            if !validate_response.ok() {
                let error_text = validate_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                log!("[WALLET CONNECT] VALIDATION FAILED");
                log!("[WALLET CONNECT] Status: {} ({})", status_code, status_text);
                log!("[WALLET CONNECT] Error response body: {}", error_text);
                log!("[WALLET CONNECT] Possible causes:");
                log!("[WALLET CONNECT]   - 404: Endpoint not found (check backend routes)");
                log!("[WALLET CONNECT]   - 401: Invalid or expired token");
                log!("[WALLET CONNECT]   - 500: Server error (check backend logs)");
                log!("[WALLET CONNECT]   - CORS: Check backend CORS configuration");
                set_error.set(Some(format!("Token validation failed ({}): {}", status_code, error_text)));
                set_connecting.set(false);
                return;
            }
            
            log!("[WALLET CONNECT] Validation response OK, parsing JSON...");

            let validate_data: WalletSetupValidateResponse = match validate_response.json::<WalletSetupValidateResponse>().await {
                Ok(data) => {
                    log!("[WALLET CONNECT] JSON parsed successfully");
                    log!("[WALLET CONNECT] Response data: valid={}, username={}, challenge={}...", 
                         data.valid, data.username, &data.challenge[..data.challenge.len().min(20)]);
                    data
                },
                Err(e) => {
                    log!("[WALLET CONNECT] Failed to parse JSON response");
                    log!("[WALLET CONNECT] Parse error: {:?}", e);
                    log!("[WALLET CONNECT] This usually means the response format is unexpected");
                    set_error.set(Some(format!("Server response error: {:?}", e)));
                    set_connecting.set(false);
                    return;
                }
            };

            log!("[WALLET CONNECT] Token validated successfully");
            log!("[WALLET CONNECT] User: {}", validate_data.username);
            log!("[WALLET CONNECT] Challenge: {}", validate_data.challenge);
            set_username.set(Some(validate_data.username.clone()));
            let challenge = validate_data.challenge;

            // Step 2: Connect to wallet
            let (wallet_address, connected_provider) = match connect_wallet_provider(&provider_clone).await {
                Ok((addr, prov)) => (addr, prov),
                Err(e) => {
                    log!("Failed to connect wallet: {}", e);
                    set_error.set(Some(format!("Failed to connect {} wallet: {}", provider_clone.name(), e)));
                    set_connecting.set(false);
                    return;
                }
            };

            log!("{} connected successfully", connected_provider.name());
            wallet_ctx.set_connected(wallet_address.clone(), connected_provider.clone());

            // Step 3: Sign challenge message
            let message = format!("Connect wallet to XForce Terminal\n\nChallenge: {}", challenge);
            let message_bytes = message.as_bytes();

            let sign_result = match sign_message_provider(&connected_provider, message_bytes, "utf8").await {
                Ok(result) => result,
                Err(e) => {
                    log!("Failed to sign message: {}", e);
                    set_error.set(Some("Failed to sign message with wallet".to_string()));
                    set_connecting.set(false);
                    return;
                }
            };

            // Extract signature from response
            let signature = match js_sys::Reflect::get(&sign_result, &JsValue::from_str("signature")) {
                Ok(sig_val) => {
                    // Convert Uint8Array to base58
                    let uint8_array = Uint8Array::from(sig_val);
                    let bytes: Vec<u8> = uint8_array.to_vec();
                    bs58::encode(bytes).into_string()
                }
                Err(e) => {
                    log!("Failed to extract signature: {:?}", e);
                    set_error.set(Some("Failed to extract signature".to_string()));
                    set_connecting.set(false);
                    return;
                }
            };

            // Step 4: Send to backend to complete setup
            let complete_req = WalletSetupCompleteRequest {
                setup_token: token,
                wallet_address,
                signature,
                challenge,
            };

            let complete_url = "http://localhost:3001/api/wallet/setup/complete".to_string();
            let complete_response = match Request::post(&complete_url)
                .json(&complete_req)
                .unwrap()
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    log!("Failed to complete setup: {:?}", e);
                    set_error.set(Some("Failed to complete wallet setup".to_string()));
                    set_connecting.set(false);
                    return;
                }
            };

            if !complete_response.ok() {
                let error_text = complete_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                log!("Setup completion failed: {}", error_text);
                set_error.set(Some(format!("Setup failed: {}", error_text)));
                set_connecting.set(false);
                return;
            }

            let _complete_data: WalletSetupCompleteResponse = match complete_response.json().await {
                Ok(data) => data,
                Err(e) => {
                    log!("Failed to parse completion response: {:?}", e);
                    set_error.set(Some("Server response error".to_string()));
                    set_connecting.set(false);
                    return;
                }
            };

            log!("Wallet setup completed successfully!");
            set_setup_complete.set(true);
            set_connecting.set(false);
        });
        */
    };

    view! {
        <div class="content-wrapper">
            <div class="left-section">
                <h1 class="main-header">
                    <span class="xf-red">"XF"</span>
                    <span class="terminal-white">"Terminal"</span>
                </h1>
                <p class="main-subheader">"Access SolanaDeFi Anywhere"</p>
                <p class="signup-text">"connect wallet to complete sign up"</p>
                <div class="disclaimer">
                    <p>
                        "The XFORCETERMINAL service and data products are owned and distributed by XFSolutions LTD. XFSolutions provides global marketing and operational support for these products. XFSolutions believe the information herein came from reliable sources, but do not guarantee their accuracy. NO information here constitutes a solicitation of the Purchase of any Cryptocurrency or Asset."
                    </p>
                </div>
            </div>
            <div class="right-section">
                <div class="container">
                    <div class="card">
                        <h1 style="color: #ffffff; font-size: 32px; margin-bottom: 12px; font-weight: 700;">
                            "Connect Wallet"
                        </h1>
                        <p class="subtitle">
                            {move || if let Some(uname) = username.get() {
                                format!("Welcome, {}!", uname)
                            } else {
                                "Select a wallet to connect".to_string()
                            }}
                        </p>

                        {move || if setup_complete.get() {
                            view! {
                                <div>
                                    <div class="success">
                                        <p style="text-align: center; font-weight: bold; font-size: 1.2em; margin-bottom: 12px;">
                                            "Wallet Connected Successfully"
                                        </p>
                                        <div class="wallet-address">
                                            {wallet_ctx.address()}
                                        </div>
                                    </div>
                                    <div class="info">
                                        <p style="text-align: center;">
                                            "You can now close this window and return to the terminal."
                                        </p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div>
                                    {move || error.get().map(|err| view! {
                                        <div class="error">
                                            <p style="text-align: center;">
                                                {err}
                                            </p>
                                        </div>
                                    })}

                                    {view! {
                                        <div>
                                            <div class="info">
                                                <p style="text-align: center; margin-bottom: 8px; font-weight: 600;">
                                                    "Connect your wallet"
                                                </p>
                                                <p style="text-align: center; font-size: 0.9em;">
                                                    "Select a wallet from the options below"
                                                </p>
                                            </div>

                                            <div style="display: flex; flex-direction: column; gap: 12px;">
                                                {move || {
                                                    let wallets = available_wallets.get();
                                                    // Always show common wallets for screenshot, even if not detected
                                                    let mut all_wallets = vec![
                                                        WalletProvider::Phantom,
                                                        WalletProvider::Solflare,
                                                        WalletProvider::Backpack,
                                                    ];
                                                    
                                                    // Add any detected wallets that aren't already in the list
                                                    for wallet in wallets.iter() {
                                                        if !all_wallets.contains(wallet) {
                                                            all_wallets.push(wallet.clone());
                                                        }
                                                    }
                                                    
                                                    let wallets_clone = all_wallets.clone();
                                                    
                                                    view! {
                                                        <div>
                                                            {wallets_clone.into_iter().map(move |wallet_provider| {
                                                                let provider_name = wallet_provider.name().to_string();
                                                                let provider_lower = provider_name.to_lowercase();
                                                                let image_path = format!("/assets/wallets/{}.webp", provider_lower);
                                                                let provider_clone = wallet_provider.clone();
                                                                
                                                                view! {
                                                                    <button
                                                                        class="wallet-button"
                                                                        on:click=move |_| connect_wallet_placeholder(provider_clone.clone())
                                                                    >
                                                                        <img 
                                                                            src=image_path.clone()
                                                                            alt=provider_name.clone()
                                                                            style="width: 32px; height: 32px; object-fit: contain; margin-right: 12px;"
                                                                        />
                                                                        <span style="font-weight: 600; flex: 1; text-align: left;">{provider_name}</span>
                                                                        <span style="font-size: 0.9em; opacity: 0.9;">"→"</span>
                                                                    </button>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
