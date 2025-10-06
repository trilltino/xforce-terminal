use gtk::prelude::*;
use gtk::{Box, Button, Label, Orientation, Window};

pub struct Header {
    container: Box,
    price_label: Label,
}

impl Header {
    pub fn new(window: &Window) -> Self {
        let container = Box::new(Orientation::Horizontal, 10);
        container.set_margin_start(20);
        container.set_margin_end(20);
        container.set_margin_top(10);
        container.set_margin_bottom(10);
        container.add_css_class("header");

        let title = Label::new(Some("STELLAR TRADING TERMINAL"));
        title.add_css_class("title");
        container.append(&title);

        container.set_halign(gtk::Align::Start);

        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        container.append(&spacer);

        // Price indicator
        let price_label = Label::new(Some("XLM/USDC: $0.1250"));
        price_label.add_css_class("price-indicator");
        price_label.set_margin_end(20);
        container.append(&price_label);

        // Window controls (far right)
        let controls = Box::new(Orientation::Horizontal, 8);

        // Minimize button (dash)
        let minimize_btn = Button::with_label("—");
        minimize_btn.add_css_class("window-control");
        minimize_btn.add_css_class("minimize-btn");
        let window_clone = window.clone();
        minimize_btn.connect_clicked(move |_| {
            window_clone.minimize();
        });
        controls.append(&minimize_btn);

        // Close button (green X)
        let close_btn = Button::with_label("✕");
        close_btn.add_css_class("window-control");
        close_btn.add_css_class("close-btn");
        let window_clone = window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });
        controls.append(&close_btn);

        container.append(&controls);

        Self {
            container,
            price_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn update_price(&self, symbol: &str, price: f64) {
        self.price_label.set_text(&format!("{}: ${:.4}", symbol, price));
    }
}
