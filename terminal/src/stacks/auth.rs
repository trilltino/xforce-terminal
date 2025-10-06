use gtk::prelude::*;
use gtk::{Box, Orientation, Stack};
use std::cell::RefCell;
use std::rc::Rc;

use super::login::LoginForm;
use super::signup::SignupForm;

pub struct AuthStack {
    container: Box,
}

impl AuthStack {
    pub fn new(main_stack: Stack) -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let auth_stack = Stack::new();
        auth_stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        auth_stack.set_transition_duration(300);

        let main_stack_ref = Rc::new(RefCell::new(Some(main_stack)));

        let login_form = LoginForm::new(&auth_stack, main_stack_ref.clone());
        let signup_form = SignupForm::new(&auth_stack, main_stack_ref.clone());

        auth_stack.add_titled(login_form.widget(), Some("login"), "Login");
        auth_stack.add_titled(signup_form.widget(), Some("signup"), "Sign Up");

        container.append(&auth_stack);

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
