use std::ops::Add;

use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{
    app::component::{Component, ComponentRender, InputBox},
    core::{Action, Lambda},
};

pub struct SearchProps {
    lambdas: Vec<Lambda>,
    filtered_list: Vec<(Vec<usize>, Lambda)>,
    highlighted_index: usize,
}

pub struct SearchPage {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    props: SearchProps,
    input_mode: InputMode,
    input_box: InputBox,
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
}

fn fuzzy_sort_lambdas(input: Vec<Lambda>, query: &str) -> Vec<(Vec<usize>, Lambda)> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

    let mut lambdas = input
        .iter()
        .map(|l| l.name.to_string())
        .flat_map(|lambda| {
            matcher
                .fuzzy_indices(&lambda, query.replace(" ", "").as_str())
                .map(|(rank, indicies)| (rank, indicies, lambda))
        })
        .collect::<Vec<_>>();

    lambdas.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    lambdas
        .into_iter()
        .map(|(_, indicies, lambda)| {
            (
                indicies,
                input.iter().find(|l| l.name == lambda).unwrap().clone(),
            )
        })
        .collect()
}

impl Component for SearchPage {
    fn new(
        state: &crate::core::State,
        action_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Self
    where
        Self: Sized,
    {
        let props = if let crate::core::State::Search(search_state) = state {
            SearchProps {
                lambdas: search_state.lambdas.clone(),
                filtered_list: fuzzy_sort_lambdas(search_state.lambdas.clone(), ""),
                highlighted_index: 0,
            }
        } else {
            SearchProps {
                lambdas: vec![],
                filtered_list: vec![],
                highlighted_index: 0,
            }
        };

        Self {
            action_tx: action_tx.clone(),
            input_mode: InputMode::Normal,
            props,
            input_box: InputBox::new(state, action_tx),
        }
    }

    fn name(&self) -> &str {
        "Search"
    }

    fn move_with_state(self, state: &crate::core::State) -> Self
    where
        Self: Sized,
    {
        let props = if let crate::core::State::Search(search_state) = state {
            SearchProps {
                lambdas: search_state.lambdas.clone(),
                filtered_list: fuzzy_sort_lambdas(
                    search_state.lambdas.clone(),
                    self.input_box.text(),
                ),
                highlighted_index: self.props.highlighted_index,
            }
        } else {
            SearchProps {
                lambdas: vec![],
                filtered_list: vec![],
                highlighted_index: 0,
            }
        };

        Self {
            action_tx: self.action_tx,
            input_mode: InputMode::Normal,
            props,
            input_box: self.input_box,
        }
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == crossterm::event::KeyCode::Esc {
            self.input_mode = InputMode::Normal;
            return;
        }

        let contains_control = key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL);

        if key.code == crossterm::event::KeyCode::Enter {
            let lambda = self.props.filtered_list.get(self.props.highlighted_index);
            if let Some(l) = lambda {
                self.action_tx
                    .send(Action::PerformSearch {
                        lambda: l.1.clone(),
                    })
                    .unwrap();
            }
            return;
        }

        match key.code {
            crossterm::event::KeyCode::Char('n') if contains_control => {
                self.props.highlighted_index = self
                    .props
                    .highlighted_index
                    .add(1)
                    .min(self.props.filtered_list.len() - 1);
            }
            crossterm::event::KeyCode::Char('p') if contains_control => {
                self.props.highlighted_index =
                    self.props.highlighted_index.saturating_sub(1).max(0);
            }
            _ => {}
        }

        if self.input_mode == InputMode::Normal {
            match key.code {
                crossterm::event::KeyCode::Char('i') => {
                    self.input_mode = InputMode::Insert;
                }
                crossterm::event::KeyCode::Char('q') => {
                    self.action_tx.send(Action::Quit).unwrap();
                }
                _ => {}
            }
        } else {
            self.input_box.handle_key_event(key);
        }

        self.props.filtered_list = fuzzy_sort_lambdas(
            self.props.lambdas.clone(),
            self.input_box.text().to_string().as_str(),
        );
        self.props.highlighted_index = self
            .props
            .highlighted_index
            .min(self.props.filtered_list.len().saturating_sub(1).max(0))
            .max(0);
    }
}

impl ComponentRender<()> for SearchPage {
    fn render(&self, frame: &mut Frame, _: ()) {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .direction(ratatui::layout::Direction::Vertical)
            .split(frame.size());

        let help_text = if self.input_mode == InputMode::Insert {
            Text::styled(
                "help: [esc] normal mode, [ctrl+n] next, [ctrl+p] previous",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            )
        } else {
            Text::styled(
                "help: [q] quit, [i] insert mode, [enter] perform search, [ctrl+n] next, [ctrl+p] previous",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            )
        };

        frame.render_widget(help_text, chunks[2]);

        let max_runtime_len = self
            .props
            .lambdas
            .iter()
            .map(|l| l.runtime.to_string().len())
            .max()
            .unwrap_or(0);

        let max_memory_len = self
            .props
            .lambdas
            .iter()
            .map(|l| l.memory.to_string().len())
            .max()
            .unwrap_or(0);

        let rows: Vec<Row> = self
            .props
            .filtered_list
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, (indicies, lambda))| {
                let mut colorised: Vec<Span<'_>> = Vec::new();

                if index == self.props.highlighted_index {
                    colorised.push(">> ".light_yellow());
                } else {
                    colorised.push("   ".white());
                }

                for (i, c) in lambda.name.chars().enumerate() {
                    if indicies.contains(&i) {
                        colorised.push(c.to_string().light_yellow());
                    } else {
                        colorised.push(c.to_string().white());
                    }
                }

                let line = if index == self.props.highlighted_index {
                    Line::from(colorised).style(Style::default().bg(Color::DarkGray))
                } else {
                    Line::from(colorised)
                };

                Row::new(vec![
                    line,
                    lambda.runtime.to_string().into(),
                    lambda.memory.to_string().into(),
                ])
            })
            .collect::<Vec<_>>();

        let widths = [
            Constraint::Min(1),
            Constraint::Length(max_runtime_len.max(7).try_into().unwrap()),
            Constraint::Length(max_memory_len.max(6).try_into().unwrap()),
        ];

        let table = Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec!["name", "runtime", "memory"])
                    .underlined()
                    .bold(),
            )
            .block(Block::default().title("Results").borders(Borders::ALL))
            .highlight_style(ratatui::style::Style::default().fg(Color::Yellow))
            .highlight_symbol(">>");

        frame.render_widget(table, chunks[0]);

        let mode_text = if self.input_mode == InputMode::Insert {
            Text::styled(
                " INSERT ",
                Style::default().fg(Color::Black).bg(Color::Yellow),
            )
        } else {
            Text::styled(
                " NORMAL ",
                Style::default().fg(Color::Black).bg(Color::LightBlue),
            )
        };

        let input_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(8),
                    Constraint::Length(8),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .direction(ratatui::layout::Direction::Horizontal)
            .split(chunks[1]);

        frame.render_widget(mode_text, input_chunks[0]);

        frame.render_widget(
            Text::styled(
                " search: ",
                Style::default().fg(Color::White).bg(Color::Black),
            ),
            input_chunks[1],
        );

        let input_box_props = crate::app::component::RenderProps {
            title: "Search".to_string(),
            area: input_chunks[2],
            border_color: Color::White,
            show_cursor: self.input_mode == InputMode::Insert,
        };

        self.input_box.render(frame, input_box_props);
    }
}
