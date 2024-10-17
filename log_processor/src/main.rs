mod listener;
mod processor;

#[tokio::main]
async fn main() {
    if let Err(e) = listener::syslog().await {
        eprintln!("Error: {:?}", e);
    }
}
