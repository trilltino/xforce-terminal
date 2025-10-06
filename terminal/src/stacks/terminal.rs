use gtk::prelude::*;
use gtk::{Box, Orientation, Window};

use crate::ui::widgets::header::Header;
use crate::ui::widgets::price_panel::PricePanel;

pub struct TerminalStack {
    container: Box,
}

impl TerminalStack {
    pub fn new(window: &Window) -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        // Header bar
        let header = Header::new(window);
        container.append(header.widget());

        // Main trading area (full width price feed)
        let main_area = Box::new(Orientation::Horizontal, 0);
        main_area.set_hexpand(true);
        main_area.set_vexpand(true);

        // Full width price panel
        let price_panel = PricePanel::new();
        price_panel.widget().set_hexpand(true);
        main_area.append(price_panel.widget());

        // Start auto-refresh
        price_panel.start_auto_refresh();

        container.append(&main_area);

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
