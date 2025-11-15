//! # Authentication Screen
//!
//! Login and signup forms using egui widgets.

use egui;
use crate::app::{AppState, AuthState, AppLike};
use crate::ui::theme::Theme;
use crate::ui::widgets::{branding, forms};

/// Signup form input values
struct SignupFormInputs<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
    confirm_password: &'a str,
}

/// Render authentication screen (login/signup)
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike, cube: &mut crate::ui::cube::RotatingCube) {
    let theme = Theme::default();

    // Split screen: Branding left, Form right
    ui.columns(2, |columns| {
        // Left column - Branding
        columns[0].vertical_centered(|ui| {
            ui.add_space(100.0);
            branding::render_branding_section(ui, "<Enter> to begin");
            branding::render_cube_section(ui, cube);
        });

        // Right column - Form
        columns[1].vertical_centered(|ui| {
            ui.add_space(100.0);

            // Draw form based on auth state
            match &state.auth {
                AuthState::Login {
                    username,
                    password,
                    error,
                    ..
                } => render_login_form(ui, username, password, error.as_deref(), app, &theme),
                AuthState::Signup {
                    username,
                    email,
                    password,
                    confirm_password,
                    error,
                    ..
                } => render_signup_form(ui, &SignupFormInputs { username, email, password, confirm_password }, error.as_deref(), app, &theme),
            }
        });
    });

    // Version info and legal disclaimer footer
    branding::render_footer(ui);
}

/// Render login form
fn render_login_form(
    ui: &mut egui::Ui,
    username: &str,
    password: &str,
    error: Option<&str>,
    app: &mut impl AppLike,
    theme: &Theme,
) {
    forms::render_form_heading(ui, "LOGIN", theme);

    // Create local mutable copies for text inputs
    let mut username_input = username.to_string();
    let mut password_input = password.to_string();
    let mut submit = false;

    // Username field
    let _username_response = forms::render_text_input(
        ui,
        "Username:",
        &mut username_input,
        "Enter username",
        false,
        [250.0, 30.0],
    );

    // Update state if changed
    {
        let mut state = app.state().write();
        if let AuthState::Login { username, .. } = &mut state.auth {
            *username = username_input.clone();
        }
    }

    ui.add_space(10.0);

    // Password field
    let password_response = forms::render_text_input(
        ui,
        "Password:",
        &mut password_input,
        "Enter password",
        true,
        [250.0, 30.0],
    );

    // Check for Enter key press
    if password_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        submit = true;
    }

    // Update state if changed
    {
        let mut state = app.state().write();
        if let AuthState::Login { password, .. } = &mut state.auth {
            *password = password_input.clone();
        }
    }

    ui.add_space(15.0);

    // Error message
    if let Some(err) = error {
        forms::render_error(ui, err, theme);
    }

    // Actions with styled buttons - aligned with text input width (250.0)
    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
        ui.set_width(250.0);
        if forms::render_button(ui, "Login", None, theme, Some(theme.selected), Some(egui::vec2(100.0, 35.0))).clicked() || submit {
            app.handle_login_click(username_input.clone(), password_input.clone());
        }

        ui.add_space(10.0);

        if ui.button("Switch to Signup").clicked() {
            app.handle_switch_to_signup();
        }
    });

    ui.add_space(10.0);
    forms::render_hint(ui, "Press <Enter> to login", theme);
}

/// Render signup form
fn render_signup_form(
    ui: &mut egui::Ui,
    inputs: &SignupFormInputs,
    error: Option<&str>,
    app: &mut impl AppLike,
    theme: &Theme,
) {
    forms::render_form_heading(ui, "SIGN UP", theme);

    // Create local mutable copies for text inputs
    let mut username_input = inputs.username.to_string();
    let mut email_input = inputs.email.to_string();
    let mut password_input = inputs.password.to_string();
    let mut confirm_password_input = inputs.confirm_password.to_string();
    let mut submit = false;

    // Username field
    forms::render_text_input(
        ui,
        "Username:",
        &mut username_input,
        "Choose username",
        false,
        [250.0, 30.0],
    );
    {
        let mut state = app.state().write();
        if let AuthState::Signup { username, .. } = &mut state.auth {
            *username = username_input.clone();
        }
    }
    ui.add_space(10.0);

    // Email field
    forms::render_text_input(
        ui,
        "Email:",
        &mut email_input,
        "your@email.com",
        false,
        [250.0, 30.0],
    );
    {
        let mut state = app.state().write();
        if let AuthState::Signup { email, .. } = &mut state.auth {
            *email = email_input.clone();
        }
    }
    ui.add_space(10.0);

    // Password field
    forms::render_text_input(
        ui,
        "Password:",
        &mut password_input,
        "Enter password",
        true,
        [250.0, 30.0],
    );
    {
        let mut state = app.state().write();
        if let AuthState::Signup { password, .. } = &mut state.auth {
            *password = password_input.clone();
        }
    }
    ui.add_space(10.0);

    // Confirm password field
    let confirm_response = forms::render_text_input(
        ui,
        "Confirm Password:",
        &mut confirm_password_input,
        "Confirm password",
        true,
        [250.0, 30.0],
    );

    // Check for Enter key press
    if confirm_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        submit = true;
    }

    {
        let mut state = app.state().write();
        if let AuthState::Signup { confirm_password, .. } = &mut state.auth {
            *confirm_password = confirm_password_input.clone();
        }
    }
    ui.add_space(15.0);

    // Error message
    if let Some(err) = error {
        forms::render_error(ui, err, theme);
    }

    // Actions with styled buttons - aligned with text input width (250.0)
    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
        ui.set_width(250.0);
        if forms::render_button(ui, "Sign Up", None, theme, Some(theme.selected), Some(egui::vec2(100.0, 35.0))).clicked() || submit {
            app.handle_signup_click(username_input.clone(), email_input.clone(), password_input.clone(), confirm_password_input.clone());
        }

        ui.add_space(10.0);

        if ui.button("Switch to Login").clicked() {
            app.handle_switch_to_login();
        }
    });

    ui.add_space(10.0);
    forms::render_hint(ui, "Press <Enter> to sign up", theme);
}
