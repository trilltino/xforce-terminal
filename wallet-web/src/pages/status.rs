//! Wallet Status Page - Show connected wallet info

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::state::wallet::use_wallet_context;

#[component]
pub fn StatusPage() -> impl IntoView {
    let wallet_ctx = use_wallet_context();
    let navigate = use_navigate();

    let on_disconnect = move |_| {
        wallet_ctx.disconnect();
        navigate("/", Default::default());
    };

    view! {
        <div class="app-container" style="display: flex; justify-content: center; align-items: center; min-height: calc(100vh - 60px); background: #000000;">
            <div class="card" style="width: 100%; max-width: 500px; padding: var(--spacing-xl); background: #000000; border: 1px solid #333333;">
                <h1 class="card-title" style="text-align: center; margin-bottom: var(--spacing-md);">
                    "Wallet Status"
                </h1>

                {move || {
                    let is_connected = wallet_ctx.is_connected();
                    let address = wallet_ctx.address();
                    let disconnect = on_disconnect.clone();

                    if is_connected {
                        view! {
                            <div>
                                <div style="background: var(--bg-card); padding: var(--spacing-lg); border-radius: var(--border-radius); border: 1px solid var(--border-color); margin-bottom: var(--spacing-lg);">
                                    <p style="color: var(--text-secondary); margin-bottom: var(--spacing-sm);">
                                        "Status"
                                    </p>
                                    <p style="color: var(--price-up); font-weight: bold; margin-bottom: var(--spacing-lg);">
                                        "Connected"
                                    </p>

                                    <p style="color: var(--text-secondary); margin-bottom: var(--spacing-sm);">
                                        "Wallet Address"
                                    </p>
                                    <p style="font-family: monospace; color: var(--text-primary); word-break: break-all; font-size: 0.9em;">
                                        {address}
                                    </p>
                                </div>

                                <button
                                    class="btn"
                                    style="width: 100%; background: var(--bg-error);"
                                    on:click=disconnect
                                >
                                    "Disconnect Wallet"
                                </button>

                                <p style="text-align: center; color: var(--text-secondary); margin-top: var(--spacing-lg); font-size: 0.9em;">
                                    "Return to terminal to continue trading"
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <p style="text-align: center; color: var(--text-secondary); margin-bottom: var(--spacing-lg);">
                                    "No wallet connected"
                                </p>
                                <a href="/" class="btn" style="width: 100%; display: block; text-align: center; text-decoration: none;">
                                    "Connect Wallet"
                                </a>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
