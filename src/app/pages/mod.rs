use ratatui::Frame;

use self::{dashboard::DashboardPage, error::ErrorPage, search::SearchPage, splash::SplashPage};

use super::component::{Component, ComponentRender};

mod dashboard;
mod error;
mod search;
mod splash;

pub enum Page {
    Splash,
    Search,
    Dashboard,
    Error,
}

pub struct AppRouter {
    pub current: Page,
    pub splash: SplashPage,
    pub search: SearchPage,
    pub dashboard: DashboardPage,
    pub error: ErrorPage,
}

impl AppRouter {
    fn get_current_page_mut(&mut self) -> &mut dyn Component {
        match self.current {
            Page::Splash => &mut self.splash,
            Page::Search => &mut self.search,
            Page::Dashboard => &mut self.dashboard,
            Page::Error => &mut self.error,
        }
    }

    fn get_current_page(&self) -> &dyn Component {
        match self.current {
            Page::Splash => &self.splash,
            Page::Search => &self.search,
            Page::Dashboard => &self.dashboard,
            Page::Error => &self.error,
        }
    }
}

impl Component for AppRouter {
    fn new(
        state: &crate::core::State,
        action_tx: &tokio::sync::mpsc::UnboundedSender<crate::core::Action>,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            current: Page::Splash,
            splash: SplashPage::new(state, action_tx),
            search: SearchPage::new(state, action_tx),
            dashboard: DashboardPage::new(state, action_tx),
            error: ErrorPage::new(state, action_tx),
        }
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        self.get_current_page_mut().handle_key_event(key);
    }

    fn name(&self) -> &str {
        self.get_current_page().name()
    }

    fn move_with_state(self, state: &crate::core::State) -> Self
    where
        Self: Sized,
    {
        Self {
            current: match state {
                crate::core::State::Search(_) => Page::Search,
                crate::core::State::Searching(_) => Page::Search,
                crate::core::State::Dashboard(_) => Page::Dashboard,
                crate::core::State::Error(_) => Page::Error,
                _ => Page::Splash,
            },
            splash: self.splash.move_with_state(state),
            search: self.search.move_with_state(state),
            dashboard: self.dashboard.move_with_state(state),
            error: self.error.move_with_state(state),
        }
    }
}

impl ComponentRender<()> for AppRouter {
    fn render(&self, frame: &mut Frame, props: ()) {
        match &self.current {
            Page::Splash => self.splash.render(frame, props),
            Page::Search => self.search.render(frame, props),
            Page::Dashboard => self.dashboard.render(frame, props),
            Page::Error => self.error.render(frame, props),
        }
    }
}
