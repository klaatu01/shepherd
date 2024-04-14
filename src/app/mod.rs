mod component;
mod pages;
use std::{
    io::{self, Stdout},
    panic,
    time::Duration,
};

use crate::core::{Action, State};
use anyhow::Context;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use pages::AppRouter;
use ratatui::{backend::CrosstermBackend, Terminal};

use self::component::{Component, ComponentRender};

const RENDERING_TICK_RATE: Duration = Duration::from_millis(250);

pub struct App {
    router: AppRouter,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    state_rx: tokio::sync::mpsc::UnboundedReceiver<State>,
    terminator_rx: tokio::sync::mpsc::Receiver<crate::termination::Interrupted>,
}

impl App {
    pub fn new(
        action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
        state_rx: tokio::sync::mpsc::UnboundedReceiver<State>,
        terminator_rx: tokio::sync::mpsc::Receiver<crate::termination::Interrupted>,
    ) -> Self {
        Self {
            router: AppRouter::new(&State::Splash, &action_tx),
            action_tx,
            state_rx,
            terminator_rx,
        }
    }

    fn setup_terminal(&self) -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
        let mut stdout = io::stdout();

        enable_raw_mode()?;

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        Ok(Terminal::new(CrosstermBackend::new(stdout))?)
    }

    fn restore_terminal(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        disable_raw_mode()?;

        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        Ok(terminal.show_cursor()?)
    }

    pub async fn run(mut self) {
        let mut terminal = self.setup_terminal().unwrap();
        let mut ticker = tokio::time::interval(RENDERING_TICK_RATE);
        let mut crossterm_events = EventStream::new();

        if let Err(err) = terminal
            .draw(|frame| self.router.render(frame, ()))
            .context("could not render to the terminal")
        {
            panic!("Error: {:?}", err);
        }

        loop {
            tokio::select! {
                _ = ticker.tick() => (),
                _ = self.terminator_rx.recv() => {
                    self.action_tx.send(Action::Quit).unwrap();
                    break;
                }
                maybe_event = crossterm_events.next() => match maybe_event {
                    Some(Ok(Event::Key(key)))  => {
                        self.router.handle_key_event(key);
                    },
                    None => {
                        self.action_tx.send(Action::Quit).unwrap();
                        break;
                    }
                    _ => (),
                },
                Some(state) = self.state_rx.recv() => {
                    if let State::Quit = state {
                        break;
                    }
                    self.router = self.router.move_with_state(&state);
                }
            }

            if let Err(err) = terminal
                .draw(|frame| self.router.render(frame, ()))
                .context("could not render to the terminal")
            {
                panic!("Error: {:?}", err);
            }
        }

        self.restore_terminal(&mut terminal).unwrap();
    }
}
