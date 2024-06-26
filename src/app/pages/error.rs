use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use crate::{
    app::component::{Component, ComponentRender},
    core::{Action, ErrorState, State},
};

pub struct ErrorProps {
    error_message: String,
}

pub struct ErrorPage {
    props: ErrorProps,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
}

impl Component for ErrorPage {
    fn new(
        state: &crate::core::State,
        action_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            props: if let State::Error(ErrorState { error_message }) = state {
                ErrorProps {
                    error_message: error_message.clone(),
                }
            } else {
                ErrorProps {
                    error_message: "".to_string(),
                }
            },
            action_tx: action_tx.clone(),
        }
    }

    fn name(&self) -> &str {
        "Error"
    }

    fn move_with_state(self, state: &crate::core::State) -> Self
    where
        Self: Sized,
    {
        Self {
            props: if let State::Error(ErrorState { error_message }) = state {
                ErrorProps {
                    error_message: error_message.clone(),
                }
            } else {
                ErrorProps {
                    error_message: "".to_string(),
                }
            },
            action_tx: self.action_tx,
        }
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char('q') => {
                self.action_tx.send(Action::Quit).unwrap();
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

fn split_text_into_lines(text: &str, max_line_length: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > max_line_length {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
            }
            current_line.clear();
        }

        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

impl ComponentRender<()> for ErrorPage {
    fn render(&self, frame: &mut Frame, _: ()) {
        let sheep_logo = r#"
        ,ww 
  wWWWWWWWx) 
  `WWWWWW' 
   II  II 

"#;

        let sheep_logo = Text::styled(sheep_logo, Style::default().fg(Color::Red)).centered();

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

        let title = Text::styled("UH OH", Style::default().fg(Color::Red)).centered();

        let error_message = Text::styled(
            "shepherd enountered an unrecoverable error",
            Style::default().fg(Color::Red),
        )
        .centered();

        let q_to_quit = Text::styled("[q] to quit", Style::default().fg(Color::Red)).centered();

        let mut text = Text::default();
        text.extend(
            split_text_into_lines(self.props.error_message.as_str(), 100)
                .into_iter()
                .map(|m| m.red()),
        );

        frame.render_widget(title, chunks[2]);
        frame.render_widget(error_message, chunks[3]);
        frame.render_widget(q_to_quit, chunks[4]);
        frame.render_widget(Paragraph::new(text).centered(), chunks[5]);
    }
}
