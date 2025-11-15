//! Wallet setup page (Phantom connection)

use leptos::prelude::*;

#[component]
pub fn WalletSetupPage() -> impl IntoView {
    view! {
        <div class="wallet-overlay">
            <div class="wallet-setup-card">
                <h1>"Wallet Setup"</h1>
                <p>"Connect your Phantom wallet"</p>
            </div>
        </div>
    }
}
