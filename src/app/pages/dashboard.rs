use crate::core::{DashboardState, EventSourceMapping, Lambda, Metric, State};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols,
    text::Text,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table},
    Frame,
};

use crate::{
    app::component::{Component, ComponentRender},
    core::Action,
};

pub struct DashboardProps {
    lambda: Option<Lambda>,
    event_source_mappings: Vec<EventSourceMapping>,
    data: Option<Vec<Metric>>,
}

pub struct DashboardPage {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    props: DashboardProps,
}

impl Component for DashboardPage {
    fn new(
        state: &crate::core::State,
        action_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            action_tx: action_tx.clone(),
            props: DashboardProps {
                data: None,
                lambda: None,
                event_source_mappings: vec![],
            },
        }
    }

    fn name(&self) -> &str {
        "Dashboard"
    }

    fn move_with_state(self, state: &crate::core::State) -> Self
    where
        Self: Sized,
    {
        Self {
            action_tx: self.action_tx,
            props: if let State::Dashboard(dashboard) = state {
                DashboardProps {
                    data: Some(dashboard.metrics.clone()),
                    lambda: Some(dashboard.lambda.clone()),
                    event_source_mappings: dashboard.event_source_mappings.clone(),
                }
            } else {
                DashboardProps {
                    data: None,
                    lambda: None,
                    event_source_mappings: vec![],
                }
            },
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

impl ComponentRender<()> for DashboardPage {
    fn render(&self, frame: &mut Frame, _: ()) {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(if self.props.event_source_mappings.is_empty() {
                        0
                    } else {
                        self.props.event_source_mappings.len() as u16 + 3
                    }),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .direction(ratatui::layout::Direction::Vertical)
            .split(frame.size());

        let lambda = self.props.lambda.as_ref().unwrap();

        let lambda_detail_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(lambda.runtime.len() as u16 + 2),
                    Constraint::Length(lambda.memory.to_string().len() as u16 + 2),
                ]
                .as_ref(),
            )
            .direction(ratatui::layout::Direction::Horizontal)
            .split(chunks[0]);

        let lambda_name = Text::styled(
            format!("{}", lambda.name),
            Style::default().fg(Color::White).bold().bg(Color::DarkGray),
        );

        frame.render_widget(lambda_name, lambda_detail_chunks[0]);

        let lambda_runtime = Text::styled(
            format!(" {} ", lambda.runtime),
            Style::default().fg(Color::White).bold().bg(Color::DarkGray),
        );

        frame.render_widget(lambda_runtime, lambda_detail_chunks[1]);

        let lambda_memory = Text::styled(
            format!(" {} ", lambda.memory),
            Style::default().fg(Color::White).bold().bg(Color::DarkGray),
        );

        frame.render_widget(lambda_memory, lambda_detail_chunks[2]);

        let rows = self
            .props
            .event_source_mappings
            .iter()
            .map(|event_source| {
                Row::new(vec![
                    format!("{} ", event_source.type_name()),
                    format!("{} ", event_source.name()),
                    format!(
                        "{} ",
                        event_source
                            .batch_size()
                            .map(|x| x.to_string())
                            .unwrap_or("".to_string())
                    ),
                    format!(
                        "{} ",
                        event_source
                            .minimum_batching_window_in_seconds()
                            .map(|x| x.to_string())
                            .unwrap_or("".to_string())
                    ),
                    format!(
                        "{} ",
                        match event_source.state() {
                            crate::core::EventSourceMappingState::Disabled => Text::styled(
                                "DISABLED",
                                Style::default().bg(Color::Red).fg(Color::Black)
                            )
                            .on_red(),
                            crate::core::EventSourceMappingState::Enabled => Text::styled(
                                "ENABLED",
                                Style::default().bg(Color::Green).fg(Color::Black)
                            )
                            .on_green(),
                        }
                    ),
                ])
            })
            .collect::<Vec<_>>();

        let max_type_name_length = self
            .props
            .event_source_mappings
            .iter()
            .map(|event_source| event_source.type_name().len())
            .max()
            .unwrap_or(0);

        let widths = vec![
            Constraint::Length(max_type_name_length as u16 + 2),
            Constraint::Min(1),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec!["Type", "Name", "Batch Size", "Batch Window", "State"])
                    .underlined()
                    .bold(),
            )
            .block(
                Block::default()
                    .title("Event Source Mappings")
                    .borders(Borders::ALL),
            );

        frame.render_widget(table, chunks[1]);

        let help_text = Text::styled(
            "help: [q] quit, [s] to search",
            Style::default().fg(Color::White).bg(Color::DarkGray),
        );

        frame.render_widget(help_text, chunks[3]);

        // favor width, count of 6 charts = 2x3, count of 5 = 2x2 + 1, count of 4 = 2x2, count of 3 = 1x3, count of 2 = 1x2, count of 1 = 1x1
        let (x, y) = match &self.props.data {
            Some(data) => {
                let count = data.len();
                match count {
                    1 => (1, 1),
                    2 => (1, 2),
                    3 => (1, 3),
                    4 => (2, 2),
                    5 => (2, 2),
                    _ => (2, 3),
                }
            }
            None => (1, 1),
        };

        let vertical_chart_chunks = Layout::default()
            .constraints(
                (0..x)
                    .map(|_| Constraint::Percentage(100 / x as u16))
                    .collect::<Vec<_>>(),
            )
            .direction(ratatui::layout::Direction::Horizontal)
            .split(chunks[2]);

        let horizontal_chart_chunks: Vec<_> = vertical_chart_chunks
            .iter()
            .map(|chunk| {
                Layout::default()
                    .constraints(
                        (0..y)
                            .map(|_| Constraint::Percentage(100 / y as u16))
                            .collect::<Vec<_>>(),
                    )
                    .direction(ratatui::layout::Direction::Vertical)
                    .split(*chunk)
            })
            .collect();

        if let Some(data) = &self.props.data {
            data.iter().enumerate().for_each(|(index, data)| {
                let dataset: Vec<_> = data
                    .timestamps
                    .iter()
                    .zip(data.values.iter().clone())
                    .map(|(timestamp, value)| (*timestamp as f64, *value as f64))
                    .clone()
                    .collect();

                let (min_x, max_x) = dataset
                    .iter()
                    .fold((f64::MAX, f64::MIN), |(min, max), (x, _)| {
                        (min.min(*x), max.max(*x))
                    });

                let (min_y, max_y) = dataset
                    .iter()
                    .fold((f64::MAX, f64::MIN), |(min, max), (_, y)| {
                        (min.min(*y), max.max(*y))
                    });

                let line_style = match data.name.as_str() {
                    "errors" => Style::default().red(),
                    _ => Style::default().green(),
                };

                let datasets = vec![Dataset::default()
                    .name(data.name.clone())
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(line_style)
                    .data(&dataset)];

                // Create the X axis and define its properties
                let x_axis = Axis::default()
                    .title("Time".white())
                    .style(Style::default().gray())
                    .bounds([min_x, max_x])
                    .labels(vec!["A".into(), "B".into(), "C".into()]);

                // Create the Y axis and define its properties
                let y_axis = Axis::default()
                    .title("Count".white())
                    .style(Style::default().gray())
                    .bounds([0.0, max_y])
                    .labels(vec![
                        min_y.to_string().into(),
                        (min_y + (max_y - min_y) / 2.0).to_string().into(),
                        max_y.to_string().into(),
                    ]);

                // Create the chart and link all the parts together
                let chart = Chart::new(datasets)
                    .block(
                        Block::default()
                            .title(data.name.to_string())
                            .borders(Borders::ALL),
                    )
                    .x_axis(x_axis)
                    .y_axis(y_axis);

                let chunk = horizontal_chart_chunks[index / y][index % y];

                frame.render_widget(chart, chunk);
            });
        }
    }
}
