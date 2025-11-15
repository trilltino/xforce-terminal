//! Transaction signing page
//!
//! Allows users to sign Solana transactions using their connected wallet.
//! Transaction data is passed via URL query parameters (base64 encoded).

use leptos::prelude::*;
use leptos::logging::log;
use leptos_router::hooks::use_query_map;
use crate::services::wallet::{
    WalletProvider,
    WalletState,
};
use crate::state::wallet::use_wallet_context;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SubmitTransactionRequest {
    #[serde(rename = "signedTransaction")]
    signed_transaction: String,
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    #[serde(rename = "inputAmount")]
    input_amount: i64,
    #[serde(rename = "outputAmount")]
    output_amount: i64,
}

#[derive(Deserialize)]
struct SubmitTransactionResponse {
    signature: String,
    status: String,
}

use crate::services::wallet::signTransactionWithProvider;

#[component]
pub fn TransactionSignPage() -> impl IntoView {
    let wallet_ctx = use_wallet_context();
    let query = use_query_map();
    
    let (error, set_error) = signal(None::<String>);
    let (signing, set_signing) = signal(false);
    let (signed, set_signed) = signal(false);
    let (tx_signature, set_tx_signature) = signal(None::<String>);
    
    // Get transaction data from query params
    let transaction_base64 = move || {
        query.with(|params| params.get("tx"))
    };
    
    let input_mint = move || {
        query.with(|params| params.get("inputMint").unwrap_or_else(|| "".to_string()))
    };
    
    let output_mint = move || {
        query.with(|params| params.get("outputMint").unwrap_or_else(|| "".to_string()))
    };
    
    let input_amount = move || {
        query.with(|params| {
            params.get("inputAmount")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };
    
    let output_amount = move || {
        query.with(|params| {
            params.get("outputAmount")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };
    
    let sign_transaction = move || {
        let tx_data = transaction_base64();
        if tx_data.is_none() {
            set_error.set(Some("No transaction data provided".to_string()));
            return;
        }
        
        let wallet_state = wallet_ctx.wallet.get();
        let provider = match wallet_state {
            WalletState::Connected { provider, .. } => provider,
            _ => {
                set_error.set(Some("No wallet connected. Please connect a wallet first.".to_string()));
                return;
            }
        };
        
        set_signing.set(true);
        set_error.set(None);
        
        let tx_data = tx_data.unwrap();
        let provider_str = match provider {
            WalletProvider::Phantom => "phantom",
            WalletProvider::Solflare => "solflare",
            WalletProvider::Backpack => "backpack",
            WalletProvider::Sollet => "sollet",
            WalletProvider::Ledger => "ledger",
        };
        
        let input_mint_val = input_mint();
        let output_mint_val = output_mint();
        let input_amount_val = input_amount();
        let output_amount_val = output_amount();
        
        leptos::task::spawn_local(async move {
            // Sign the transaction
            let signed_tx_base64 = match signTransactionWithProvider(provider_str, &tx_data).await {
                Ok(tx_js) => {
                    // Extract string from JsValue
                    if let Some(tx_str) = tx_js.as_string() {
                        tx_str
                    } else {
                        set_error.set(Some("Failed to get signed transaction: invalid response type".to_string()));
                        set_signing.set(false);
                        return;
                    }
                },
                Err(e) => {
                    let error_msg = if let Some(err_str) = e.as_string() {
                        err_str
                    } else {
                        format!("Sign error: {:?}", e)
                    };
                    log!("Failed to sign transaction: {}", error_msg);
                    set_error.set(Some(format!("Failed to sign transaction: {}", error_msg)));
                    set_signing.set(false);
                    return;
                }
            };
            
            log!("Transaction signed successfully");
            
            // Submit to backend
            let submit_req = SubmitTransactionRequest {
                signed_transaction: signed_tx_base64.clone(),
                input_mint: input_mint_val,
                output_mint: output_mint_val,
                input_amount: input_amount_val,
                output_amount: output_amount_val,
            };
            
            let submit_url = "http://localhost:3001/api/transactions/submit".to_string();
            let submit_response = match Request::post(&submit_url)
                .json(&submit_req)
                .unwrap()
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    log!("Failed to submit transaction: {:?}", e);
                    set_error.set(Some("Failed to submit transaction to backend".to_string()));
                    set_signing.set(false);
                    return;
                }
            };
            
            if !submit_response.ok() {
                let error_text = submit_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                log!("Transaction submission failed: {}", error_text);
                set_error.set(Some(format!("Submission failed: {}", error_text)));
                set_signing.set(false);
                return;
            }
            
            let submit_data: SubmitTransactionResponse = match submit_response.json().await {
                Ok(data) => data,
                Err(e) => {
                    log!("Failed to parse submission response: {:?}", e);
                    set_error.set(Some("Server response error".to_string()));
                    set_signing.set(false);
                    return;
                }
            };
            
            log!("Transaction submitted successfully: {}", submit_data.signature);
            set_tx_signature.set(Some(submit_data.signature));
            set_signed.set(true);
            set_signing.set(false);
        });
    };
    
    view! {
        <div class="wallet-overlay">
            <div class="wallet-setup-card" style="max-width: 600px;">
                <h1>"Sign Transaction"</h1>
                
                {move || {
                    let tx_data = transaction_base64();
                    if tx_data.is_none() {
                        return view! {
                            <div class="error">
                                <p>"No transaction data provided"</p>
                                <p style="font-size: 0.9em; margin-top: 8px;">
                                    "Please provide transaction data via URL parameters."
                                </p>
                            </div>
                        }.into_any();
                    }
                    
                    let wallet_state = wallet_ctx.wallet.get();
                    if !wallet_state.is_connected() {
                        return view! {
                            <div class="error">
                                <p>"No wallet connected"</p>
                                <p style="font-size: 0.9em; margin-top: 8px;">
                                    "Please connect a wallet to sign transactions."
                                </p>
                            </div>
                        }.into_any();
                    }
                    
                    if signed.get() {
                        return view! {
                            <div>
                                <div class="success">
                                    <p style="text-align: center; font-weight: bold; font-size: 1.2em; margin-bottom: 12px;">
                                        "Transaction Signed and Submitted"
                                    </p>
                                    {move || tx_signature.get().map(|sig| view! {
                                        <div style="word-break: break-all; font-family: monospace; font-size: 0.9em; margin-top: 12px;">
                                            <p style="color: #888888; margin-bottom: 4px;">"Signature:"</p>
                                            <p style="color: #00ff88;">{sig}</p>
                                        </div>
                                    })}
                                    <p style="text-align: center; margin-top: 16px; font-size: 0.9em;">
                                        "You can close this window."
                                    </p>
                                </div>
                            </div>
                        }.into_any();
                    }
                    
                    view! {
                        <div>
                            {move || error.get().map(|err| view! {
                                <div class="error">
                                    <p>{err}</p>
                                </div>
                            })}
                            
                            <div class="info" style="margin: 16px 0;">
                                <p style="margin-bottom: 8px;">"Review Transaction Details"</p>
                                {move || {
                                    let input = input_mint();
                                    let output = output_mint();
                                    if !input.is_empty() && !output.is_empty() {
                                        view! {
                                            <div style="font-size: 0.9em; margin-top: 8px;">
                                                <p>"Input: " <span style="font-family: monospace;">{input}</span></p>
                                                <p>"Output: " <span style="font-family: monospace;">{output}</span></p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }
                                }}
                            </div>
                            
                            <button
                                class="btn-secondary"
                                style="width: 100%; padding: 16px; font-size: 1em; margin-top: 16px;"
                                on:click=move |_| sign_transaction()
                                disabled=signing.get()
                            >
                                {move || if signing.get() {
                                    "Signing..."
                                } else {
                                    "Sign and Submit Transaction"
                                }}
                            </button>
                            
                            {move || if signing.get() {
                                view! {
                                    <div style="text-align: center; margin-top: 16px;">
                                        <div class="spinner"></div>
                                        <p style="color: #888888; font-size: 0.9em; margin-top: 12px;">
                                            "Please approve the transaction in your wallet..."
                                        </p>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
