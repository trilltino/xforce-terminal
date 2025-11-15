//! About Page - Space Theme

use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="app-container" style="display: flex; flex-direction: column; min-height: calc(100vh - 60px);">
            <div style="flex: 1; display: flex; justify-content: center; align-items: center; padding: 48px 24px;">
                <div class="about-container">
                    <div class="card">
                        <h1 class="card-title" style="text-align: center; margin-bottom: 32px; color: #ffffff; font-size: 32px; font-weight: 700;">
                            "About XForce Terminal"
                        </h1>
                        
                        <div style="color: #ffffff; line-height: 1.8; font-size: 16px;">
                            <p style="margin-bottom: 24px; color: #cccccc;">
                                "XForce Terminal is a professional-grade DeFi trading terminal designed for the Solana ecosystem. 
                                Built with performance and usability in mind, it provides traders with the tools they need to 
                                execute trades efficiently and manage their portfolios."
                            </p>
                            
                            <h2 style="color: #ffffff; font-size: 24px; margin-top: 32px; margin-bottom: 16px; font-weight: 600;">
                                "Features"
                            </h2>
                            <ul style="margin-left: 24px; margin-bottom: 24px; color: #cccccc;">
                                <li style="margin-bottom: 8px;">"Real-time market data and price feeds"</li>
                                <li style="margin-bottom: 8px;">"Secure wallet integration with Phantom, Solflare, and Backpack"</li>
                                <li style="margin-bottom: 8px;">"Advanced trading capabilities"</li>
                                <li style="margin-bottom: 8px;">"Portfolio management and tracking"</li>
                                <li style="margin-bottom: 8px;">"Transaction signing and management"</li>
                            </ul>
                            
                            <h2 style="color: #ffffff; font-size: 24px; margin-top: 32px; margin-bottom: 16px; font-weight: 600;">
                                "Technology"
                            </h2>
                            <p style="margin-bottom: 24px; color: #cccccc;">
                                "XForce Terminal is built using modern web technologies and the Rust programming language 
                                for maximum performance and security. The terminal interface provides a Bloomberg-inspired 
                                experience optimized for cryptocurrency trading."
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
