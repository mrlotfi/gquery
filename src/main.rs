mod collection;
mod config;
mod server;

#[tokio::main]
async fn main() {
    server::serve().await;
}
