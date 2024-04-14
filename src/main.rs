pub mod app;
pub mod aws;
pub mod core;
pub mod state;
pub mod termination;

#[tokio::main]
async fn main() {
    let aws = aws::AWS::new().await;

    let (terminator_tx, terminator_rx) = termination::create_termination();
    let (state_tx, state_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = state::StateManager::new(aws, state_tx, action_rx);
    let app = app::App::new(action_tx, state_rx, terminator_rx);

    tokio::join!(app.run(), state_manager.run());
}
