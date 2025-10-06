use gtk::prelude::*;
use gtk::{Box, Label, Orientation, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::services::api::ApiClient;
use crate::utils::runtime::TOKIO_RT;

pub struct PricePanel {
    container: Box,
    price_list: Box,
    counter_label: Label,
}

impl PricePanel {
    pub fn new() -> Self {
        let panel = Box::new(Orientation::Vertical, 10);
        panel.set_margin_start(20);
        panel.set_margin_top(10);
        panel.add_css_class("panel");

        let header_box = Box::new(Orientation::Horizontal, 10);

        let title = Label::new(Some("REFLECTOR ORACLE PRICES"));
        title.add_css_class("panel-title");
        title.set_halign(gtk::Align::Start);
        title.set_hexpand(true);
        header_box.append(&title);

        // Update indicator
        let update_label = Label::new(Some("● LIVE"));
        update_label.add_css_class("price-value");
        update_label.set_halign(gtk::Align::End);
        header_box.append(&update_label);

        panel.append(&header_box);

        // Add update counter below header
        let counter_label = Label::new(Some("Updates: 0"));
        counter_label.add_css_class("data-label");
        counter_label.set_halign(gtk::Align::End);
        counter_label.set_margin_end(10);
        counter_label.set_margin_bottom(5);
        panel.append(&counter_label);

        // Scrolled window for price list
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_min_content_height(400);

        let price_list = Box::new(Orientation::Vertical, 5);
        price_list.set_margin_start(10);
        price_list.set_margin_end(10);
        price_list.set_margin_top(5);
        price_list.set_margin_bottom(5);

        let loading_label = Label::new(Some("Fetching prices..."));
        loading_label.add_css_class("data-label");
        price_list.append(&loading_label);

        scrolled.set_child(Some(&price_list));
        panel.append(&scrolled);

        Self {
            container: panel,
            price_list,
            counter_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn start_auto_refresh(&self) {
        let price_list_clone = self.price_list.clone();
        let counter_clone = self.counter_label.clone();
        let update_count = Rc::new(RefCell::new(0u32));

        glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
            let price_list_ref = price_list_clone.clone();
            let counter_ref = counter_clone.clone();
            let update_count_ref = update_count.clone();

            glib::spawn_future_local(async move {
                let api = ApiClient::new();

                // Use channel to bridge Tokio and glib
                let (tx, rx) = async_channel::bounded(1);
                TOKIO_RT.spawn(async move {
                    let result = api.get_reflector_prices().await;
                    let _ = tx.send(result).await;
                });

                if let Ok(result) = rx.recv().await {
                    // Increment update counter
                    let mut count = update_count_ref.borrow_mut();
                    *count += 1;
                    counter_ref.set_text(&format!("Updates: {}", *count));
                    drop(count);

                    // Remove old prices
                    while let Some(child) = price_list_ref.first_child() {
                        price_list_ref.remove(&child);
                    }

                    match result {
                        Ok(response) => {
                            // Sort assets by symbol
                            let mut assets: Vec<_> = response.prices.iter().collect();
                            assets.sort_by_key(|(symbol, _)| *symbol);

                            for (symbol, price_data) in assets {
                                let price_row = Box::new(Orientation::Horizontal, 10);
                                price_row.set_margin_bottom(3);

                                let symbol_label = Label::new(Some(symbol.as_str()));
                                symbol_label.add_css_class("price-symbol");
                                symbol_label.set_width_chars(8);
                                symbol_label.set_xalign(0.0);
                                price_row.append(&symbol_label);

                                let price_text = format!("${:.6}", price_data.price);
                                let price_label = Label::new(Some(&price_text));
                                price_label.add_css_class("price-value");
                                price_label.set_hexpand(true);
                                price_label.set_xalign(1.0);
                                price_row.append(&price_label);

                                price_list_ref.append(&price_row);
                            }
                        }
                        Err(err) => {
                            let error_label = Label::new(Some(&format!("Error: {}", err)));
                            error_label.add_css_class("error");
                            price_list_ref.append(&error_label);
                        }
                    }
                }
            });

            glib::ControlFlow::Continue
        });
    }
}
