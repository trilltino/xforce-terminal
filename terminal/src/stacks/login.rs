use gtk::glib;
use gtk::prelude::*;
use gtk::{Box, Button, Entry, Label, Orientation, Spinner, Stack};
use std::cell::RefCell;
use std::rc::Rc;

use crate::services::api::ApiClient;
use crate::utils::runtime::TOKIO_RT;

pub struct LoginForm {
    container: Box,
}

impl LoginForm {
    pub fn new(auth_stack: &Stack, main_stack: Rc<RefCell<Option<Stack>>>) -> Self {
        let page = Box::new(Orientation::Vertical, 20);
        page.set_margin_start(40);
        page.set_margin_end(40);
        page.set_margin_top(40);
        page.set_margin_bottom(40);
        page.set_width_request(400);

        let title = Label::new(Some("DeFi Trading Terminal"));
        title.add_css_class("title-1");
        page.append(&title);

        let subtitle = Label::new(Some("Login to your account"));
        subtitle.add_css_class("dim-label");
        page.append(&subtitle);

        let spacer = Box::new(Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        page.append(&spacer);

        let email_label = Label::new(Some("Email or Username"));
        email_label.set_halign(gtk::Align::Start);
        page.append(&email_label);

        let email_entry = Entry::new();
        email_entry.set_placeholder_text(Some("Enter your email or username"));
        email_entry.set_margin_bottom(10);
        page.append(&email_entry);

        let password_label = Label::new(Some("Password"));
        password_label.set_halign(gtk::Align::Start);
        page.append(&password_label);

        let password_entry = Entry::new();
        password_entry.set_placeholder_text(Some("Enter your password"));
        password_entry.set_visibility(false);
        password_entry.set_margin_bottom(10);
        page.append(&password_entry);

        let error_label = Label::new(None);
        error_label.add_css_class("error");
        error_label.set_visible(false);
        error_label.set_margin_bottom(10);
        page.append(&error_label);

        let button_container = Box::new(Orientation::Horizontal, 10);
        button_container.set_halign(gtk::Align::Fill);

        let spinner = Spinner::new();
        spinner.set_visible(false);
        button_container.append(&spinner);

        let login_button = Button::with_label("Login");
        login_button.add_css_class("suggested-action");
        login_button.add_css_class("pill");
        login_button.set_hexpand(true);
        button_container.append(&login_button);

        page.append(&button_container);

        let signup_box = Box::new(Orientation::Horizontal, 5);
        signup_box.set_halign(gtk::Align::Center);
        signup_box.set_margin_top(20);

        let signup_text = Label::new(Some("Don't have an account?"));
        signup_box.append(&signup_text);

        let signup_link = Button::with_label("Sign Up");
        signup_link.add_css_class("link");
        signup_box.append(&signup_link);

        page.append(&signup_box);

        let stack_clone = auth_stack.clone();
        signup_link.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("signup");
        });

        login_button.connect_clicked(glib::clone!(
            @weak email_entry,
            @weak password_entry,
            @weak error_label,
            @weak login_button,
            @weak spinner,
            @strong main_stack
            => move |_| {
                let email = email_entry.text().to_string();
                let password = password_entry.text().to_string();

                error_label.set_visible(false);

                if email.is_empty() {
                    error_label.set_text("Email or username is required");
                    error_label.set_visible(true);
                    return;
                }

                if password.is_empty() {
                    error_label.set_text("Password is required");
                    error_label.set_visible(true);
                    return;
                }

                println!("🔐 Login attempt: {}", email);

                let (tx, rx) = async_channel::bounded(1);

                let api = ApiClient::new();
                TOKIO_RT.spawn(async move {
                    let result = api.login(email, password).await;
                    let _ = tx.send(result).await;
                });

                glib::spawn_future_local(glib::clone!(
                    @weak error_label,
                    @weak login_button,
                    @weak spinner,
                    @strong main_stack
                    => async move {
                        if let Ok(result) = rx.recv().await {
                            match result {
                                Ok(response) => {
                                    println!("✅ Login successful!");
                                    println!("   Token: {}", response.token);
                                    println!("   User: {}", response.user.username);

                                    if let Some(ref stack) = *main_stack.borrow() {
                                        stack.set_visible_child_name("terminal");
                                    }

                                    spinner.stop();
                                    spinner.set_visible(false);
                                    login_button.set_sensitive(true);
                                }
                                Err(err) => {
                                    println!("❌ Login failed: {}", err);
                                    error_label.set_text(&err);
                                    error_label.set_visible(true);

                                    login_button.set_sensitive(true);
                                    spinner.stop();
                                    spinner.set_visible(false);
                                }
                            }
                        }
                    }
                ));
            }
        ));

        Self { container: page }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
