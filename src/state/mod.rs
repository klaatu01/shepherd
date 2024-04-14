use crate::{
    aws::AWS,
    core::{Action, DashboardState, SearchState, SearchingState, State},
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
                    self.state_tx
                        .send(State::Search(SearchState {
                            lambdas: self.aws.lambda_functions().await,
                        }))
                        .unwrap();
                }
                Action::PerformSearch { lambda } => {
                    self.state_tx
                        .send(State::Searching(SearchingState {
                            lambda: lambda.clone(),
                        }))
                        .unwrap();
                    let metrics = self.aws.metrics(&lambda).await;
                    let event_source_mappings = self.aws.event_source_mappings(&lambda).await;
                    self.state_tx
                        .send(State::Dashboard(DashboardState {
                            lambda,
                            metrics,
                            event_source_mappings,
                        }))
                        .unwrap();
                }
            }
        }
    }
}
