use crate::ui::{budget, home, register, rsu};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Budget,
    Register,
    Rsu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    view: View,
    exit_requested: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            view: View::Home,
            exit_requested: false,
        }
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn should_exit(&self) -> bool {
        self.exit_requested
    }

    pub const fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    pub const fn set_view(&mut self, view: View) {
        self.view = view;
    }

    #[must_use]
    pub fn render_frame(&self) -> String {
        match self.view {
            View::Home => home::render(),
            View::Budget => budget::render(),
            View::Register => register::render(),
            View::Rsu => rsu::render(),
        }
    }
}
