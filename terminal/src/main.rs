mod services;
mod stacks;
mod ui;
mod utils;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Stack};

use stacks::auth::AuthStack;
use stacks::terminal::TerminalStack;
use ui::styles;

const APP_ID: &str = "com.xforce.terminal";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("DeFi Trading Terminal")
        .default_width(1366)
        .default_height(768)
        .decorated(false)
        .build();

    // Main stack for switching between auth and terminal views
    let main_stack = Stack::new();
    main_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    main_stack.set_transition_duration(500);

    // Create auth stack (login/signup)
    let auth_stack = AuthStack::new(main_stack.clone());
    main_stack.add_named(auth_stack.widget(), Some("login"));

    // Create terminal stack (full trading interface)
    let terminal_stack = TerminalStack::new(window.upcast_ref());
    main_stack.add_named(terminal_stack.widget(), Some("terminal"));

    // Start with auth stack
    main_stack.set_visible_child_name("login");

    window.set_child(Some(&main_stack));

    // Apply CSS theme
    styles::apply_theme();

    window.present();
}
