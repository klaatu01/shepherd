#[cfg(unix)]
use tokio::signal::unix::signal;

#[derive(Debug, Clone)]
pub enum Interrupted {
    OsSigInt,
    UserInt,
}

#[cfg(unix)]
async fn terminate_by_unix_signal(terminator: tokio::sync::mpsc::Sender<Interrupted>) {
    let mut interrupt_signal = signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("failed to create interrupt signal stream");

    interrupt_signal.recv().await;

    terminator
        .send(Interrupted::OsSigInt)
        .await
        .expect("failed to send interrupt signal");
}

// create a broadcast channel for retrieving the application kill signal
pub fn create_termination() -> (
    tokio::sync::mpsc::Sender<Interrupted>,
    tokio::sync::mpsc::Receiver<Interrupted>,
) {
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    #[cfg(unix)]
    tokio::spawn(terminate_by_unix_signal(tx.clone()));

    (tx, rx)
}
