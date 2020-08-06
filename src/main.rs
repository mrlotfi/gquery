mod config;
mod server;
mod collection;

#[tokio::main]
async fn main() {
    server::serve().await;
}
