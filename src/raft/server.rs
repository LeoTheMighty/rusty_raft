use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task;
use tonic::transport::Server;
use crate::raft::protobufs::raft_service_server::RaftServiceServer;
use crate::raft::rusty_raft::RustyRaft;

impl RustyRaft {
    async fn run_server(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = self.server_address.clone().parse()?;
        self.log(format!("Server listening on {}", addr));

        let mut shutdown_rx = self.shutdown_tx.lock().await.as_ref().unwrap().subscribe();

        let server = Server::builder()
            .add_service(RaftServiceServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                shutdown_rx.changed().await.ok();
            });

        self.log("Server started...".to_string());

        self.clone().set_follower_role().await;

        server.await?;

        self.log("Server shut down gracefully.".to_string());
        Ok(())
    }

    pub async fn start_server(self: Arc<Self>) {
        let raft_clone = self.clone();
        let (shutdown_tx, _shutdown_rx) = watch::channel(());
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        task::spawn(async move {
            if let Err(e) = raft_clone.run_server().await {
                eprintln!("Server error: {}", e);
            }
        });
    }

    pub async fn stop_server(self: Arc<Self>) {
        self.log("Shutting down server...".to_string());

        if let Some(shutdown_tx) = self.shutdown_tx.lock().await.take() {
            let _ = shutdown_tx.send(());
        }
    }
}
