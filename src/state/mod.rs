use anyhow::Result;

use crate::{
    aws::AWS,
    core::{Action, DashboardState, ErrorState, SearchState, SearchingState, State},
};

pub struct StateManager {
    aws: AWS,
    action_rx: tokio::sync::mpsc::UnboundedReceiver<Action>,
    state_tx: tokio::sync::mpsc::UnboundedSender<State>,
}

impl StateManager {
    pub fn new(
        aws: AWS,
        state_tx: tokio::sync::mpsc::UnboundedSender<State>,
        action_rx: tokio::sync::mpsc::UnboundedReceiver<Action>,
    ) -> Self {
        Self {
            aws,
            action_rx,
            state_tx,
        }
    }

    pub async fn run(mut self) {
        loop {
            let action = self.action_rx.recv().await.unwrap();
            match action {
                Action::Quit => {
                    self.state_tx.send(State::Quit).unwrap();
                    break;
                }
                Action::Search => {
                    let lambdas = self.aws.lambda_functions().await;
                    match lambdas {
                        Ok(lambdas) => {
                            self.state_tx
                                .send(State::Search(SearchState { lambdas }))
                                .unwrap();
                        }
                        Err(e) => {
                            self.state_tx
                                .send(State::Error(ErrorState {
                                    error_message: e.root_cause().to_string(),
                                }))
                                .unwrap();
                        }
                    }
                }
                Action::PerformSearch { lambda } => {
                    self.state_tx
                        .send(State::Searching(SearchingState {
                            lambda: lambda.clone(),
                        }))
                        .unwrap();
                    let metrics = self.aws.metrics(&lambda).await;
                    let event_source_mappings = self.aws.event_source_mappings(&lambda).await;
                    match (metrics, event_source_mappings) {
                        (Ok(metrics), Ok(event_source_mappings)) => {
                            self.state_tx
                                .send(State::Dashboard(DashboardState {
                                    lambda,
                                    metrics,
                                    event_source_mappings,
                                }))
                                .unwrap();
                        }
                        (Err(e), _) => {
                            self.state_tx
                                .send(State::Error(ErrorState {
                                    error_message: e.to_string(),
                                }))
                                .unwrap();
                        }
                        (_, Err(e)) => {
                            self.state_tx
                                .send(State::Error(ErrorState {
                                    error_message: e.to_string(),
                                }))
                                .unwrap();
                        }
                    }
                }
            }
        }
    }
}
