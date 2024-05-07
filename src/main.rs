use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    println!("Tokio environment is running!");

    let task = async {
        time::sleep(Duration::from_secs(2)).await;
        println!("Task completed after a delay.");
    };

    // Execute the task
    task.await;
}
