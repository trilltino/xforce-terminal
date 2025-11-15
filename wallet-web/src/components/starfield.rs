//! Starfield Background Component
//! Creates an animated starfield background with twinkling stars

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use gloo_timers::future::TimeoutFuture;

#[component]
pub fn Starfield() -> impl IntoView {
    // Create stars after component mounts
    let create_stars_effect = move || {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        
        // Use spawn_local to run async code
        leptos::task::spawn_local(async move {
            // Wait a bit for DOM to be ready
            TimeoutFuture::new(100).await;
            
            if let Some(starfield_element) = document.get_element_by_id("starfield") {
                if let Some(html_element) = starfield_element.dyn_ref::<HtmlElement>() {
                    create_stars(html_element);
                }
            }
        });
    };

    // Run effect on mount
    create_stars_effect();

    view! {
        <div
            class="starfield"
            id="starfield"
        ></div>
    }
}

fn create_stars(container: &HtmlElement) {
    let document = web_sys::window()
        .and_then(|win| win.document())
        .expect("should have a document");

    let num_stars = 150;

    for _i in 0..num_stars {
        let star = document
            .create_element("div")
            .expect("should create star element");

        star.set_class_name("stars");

        // Random position
        let left = js_sys::Math::random() * 100.0;
        let top = js_sys::Math::random() * 100.0;
        let delay = js_sys::Math::random() * 3.0;
        let size = js_sys::Math::random() * 2.0 + 1.0;

        star.set_attribute("style", &format!(
            "left: {}%; top: {}%; animation-delay: {}s; width: {}px; height: {}px;",
            left, top, delay, size, size
        )).expect("should set style");

        // Create some larger, brighter stars (20% chance)
        if js_sys::Math::random() > 0.8 {
            let large_size = js_sys::Math::random() * 3.0 + 2.0;
            star.set_attribute("style", &format!(
                "left: {}%; top: {}%; animation-delay: {}s; width: {}px; height: {}px; \
                box-shadow: 0 0 10px rgba(255, 255, 255, 1), 0 0 20px rgba(255, 0, 0, 0.5); \
                background: #ff6666;",
                left, top, delay, large_size, large_size
            )).expect("should set style");
        }
        // Create some red-tinted stars (10% chance)
        else if js_sys::Math::random() > 0.9 {
            star.set_attribute("style", &format!(
                "left: {}%; top: {}%; animation-delay: {}s; width: {}px; height: {}px; \
                background: #ff3333; \
                box-shadow: 0 0 8px rgba(255, 0, 0, 0.8), 0 0 16px rgba(255, 0, 0, 0.4);",
                left, top, delay, size, size
            )).expect("should set style");
        }

        container
            .append_child(&star)
            .expect("should append star");
    }
}
