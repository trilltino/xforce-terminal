//! Navigation Bar Component - Space Theme

use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav>
            <div style="max-width: 1200px; margin: 0 auto; padding: 0 24px; display: flex; justify-content: space-between; align-items: center;">
                <A href="/">
                    <span style="font-size: 20px; font-weight: 700; color: #ffffff; text-decoration: none; letter-spacing: -0.01em;">
                        "XForce Terminal"
                    </span>
                </A>
                <div style="display: flex; gap: 8px; align-items: center;">
                    <A href="/" exact=true>
                        <span class="nav-link" style="color: #ffffff; text-decoration: none; font-weight: 500; padding: 8px 16px; border-radius: 4px; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);">
                            "Home"
                        </span>
                    </A>
                    <A href="/about">
                        <span class="nav-link" style="color: #ffffff; text-decoration: none; font-weight: 500; padding: 8px 16px; border-radius: 4px; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);">
                            "About"
                        </span>
                    </A>
                </div>
            </div>
        </nav>
    }
}
