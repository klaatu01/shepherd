use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled},
    text::Text,
    Frame,
};

use crate::{
    app::component::{Component, ComponentRender},
    core::Action,
};

pub struct SplashProps;

pub struct SplashPage {
    props: SplashProps,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
}

impl Component for SplashPage {
    fn new(
        state: &crate::core::State,
        action_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            props: SplashProps,
            action_tx: action_tx.clone(),
        }
    }

    fn name(&self) -> &str {
        "Splash"
    }

    fn move_with_state(self, state: &crate::core::State) -> Self
    where
        Self: Sized,
    {
        Self {
            props: SplashProps,
            action_tx: self.action_tx,
        }
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char('q') => {
                self.action_tx.send(Action::Quit).unwrap();
            }
            crossterm::event::KeyCode::Char('s') => {
                self.action_tx.send(Action::Search).unwrap();
            }
            _ => {}
        }
    }
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl ComponentRender<()> for SplashPage {
    fn render(&self, frame: &mut Frame, _: ()) {
        let sheep_logo = r#"
        ,ww 
  wWWWWWWW_) 
  `WWWWWW' 
   II  II 

"#;

        let sheep_logo = Text::styled(sheep_logo, Style::default().fg(Color::White)).centered();

        let rect = centered_rect(frame.size(), 50, 100);

        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Max(7),
                    Constraint::Max(1),
                    Constraint::Max(1),
                    Constraint::Max(1),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .direction(Direction::Vertical)
            .split(rect);

        frame.render_widget(sheep_logo, chunks[1]);

        let title = Text::styled("SHEPHERD", Style::default().fg(Color::White)).centered();

        let q_to_quit = Text::styled("[q] to quit", Style::default().fg(Color::White)).centered();

        let s_to_search =
            Text::styled("[s] to search", Style::default().fg(Color::White)).centered();

        frame.render_widget(title, chunks[2]);
        frame.render_widget(q_to_quit, chunks[3]);
        frame.render_widget(s_to_search, chunks[4]);
    }
}
