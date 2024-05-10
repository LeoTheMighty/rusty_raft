use tokio::sync::{Mutex, mpsc};
use tokio::time::{self, Duration};
use std::sync::Arc;
use std::future::Future;
use rand::Rng;

const TIMEOUT_MIN: u64 = 1000;
const TIMEOUT_MAX: u64 = 3000;

const HEARTBEAT_TIMEOUT: u64 = 500;
const ELECTION_TIMEOUT: u64 = 1000;

pub struct TimeoutHandler {
    task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    cancel_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

impl Default for TimeoutHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeoutHandler {
    pub fn new() -> Self {
        Self {
            task: Arc::new(Mutex::new(None)),
            cancel_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_election_timeout() -> Duration {
        Duration::from_millis(ELECTION_TIMEOUT)
    }

    pub fn get_heartbeat_timeout() -> Duration {
        Duration::from_millis(HEARTBEAT_TIMEOUT)
    }

    pub fn get_random_timeout() -> Duration {
        Duration::from_millis(
            rand::thread_rng().gen_range(TIMEOUT_MIN..=TIMEOUT_MAX),
        )
    }

    // pub async fn cancel_timeout(&self) {
    //     self.cancel_tx.lock().await.take().unwrap().send(()).await.unwrap();
    // }

    pub async fn set_timeout<F>(&self, duration: Duration, fut: F)
        where
            F: Future<Output = ()> + Send + 'static,
    {
        let mut task_guard = self.task.lock().await;
        let mut cancel_tx_guard = self.cancel_tx.lock().await;

        // Cancel the old task if it exists
        if let Some(task) = task_guard.take() {
            task.abort();
        }

        // Create a new channel for canceling the new task
        let (cancel_tx, mut cancel_rx) = mpsc::channel(1);

        // Spawn a new task
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = time::sleep(duration) => {
                    fut.await;
                }
                _ = cancel_rx.recv() => {}
            }
        });

        // Store the new task and cancel sender
        *task_guard = Some(handle);
        *cancel_tx_guard = Some(cancel_tx);
    }
}

impl Clone for TimeoutHandler {
    fn clone(&self) -> Self {
        Self {
            task: Arc::clone(&self.task),
            cancel_tx: Arc::clone(&self.cancel_tx),
        }
    }
}
