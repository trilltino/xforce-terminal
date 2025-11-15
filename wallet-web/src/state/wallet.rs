//! Wallet state management

use leptos::prelude::*;
use crate::services::wallet::{WalletState, WalletProvider};

/// Global wallet context
#[derive(Clone, Copy)]
pub struct WalletContext {
    pub wallet: RwSignal<WalletState>,
}

impl WalletContext {
    pub fn new() -> Self {
        Self {
            wallet: RwSignal::new(WalletState::Disconnected),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.wallet.with(|state| state.is_connected())
    }

    pub fn address(&self) -> Option<String> {
        self.wallet.with(|state| state.address().map(|s| s.to_string()))
    }
    
    pub fn provider(&self) -> Option<WalletProvider> {
        self.wallet.with(|state| state.provider())
    }

    pub fn set_connecting(&self) {
        self.wallet.set(WalletState::Connecting);
    }

    pub fn set_connected(&self, address: String, provider: WalletProvider) {
        self.wallet.set(WalletState::Connected { address, provider });
    }
    
    pub fn set_connected_address_only(&self, address: String) {
        // For backward compatibility, use Phantom as default
        self.wallet.set(WalletState::Connected { 
            address, 
            provider: WalletProvider::Phantom 
        });
    }

    pub fn set_error(&self, error: String) {
        self.wallet.set(WalletState::Error(error));
    }

    pub fn disconnect(&self) {
        self.wallet.set(WalletState::Disconnected);
    }
}

pub fn provide_wallet_context() -> WalletContext {
    let context = WalletContext::new();
    provide_context(context);
    context
}

pub fn use_wallet_context() -> WalletContext {
    expect_context::<WalletContext>()
}
