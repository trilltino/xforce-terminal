//! Navigation Bar Component - Space Theme

use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav>
            <div style="max-width: 1200px; margin: 0 auto; padding: 0 24px; display: flex; justify-content: flex-start; align-items: center;">
                <A href="/" class="nav-link-clean">
                    <span class="nav-title">
                        <span class="xf-red">"XF"</span><span class="terminal-white">"Terminal"</span>
                    </span>
                </A>
            </div>
        </nav>
    }
}
