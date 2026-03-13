use rust_serv::{Config, Server};
use std::env;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> rust_serv::error::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Load configuration
    let config = if args.len() > 1 {
        // Load from file
        Config::default() // TODO: Load from file
    } else {
        Config::default()
    };

    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.log_level))
        .with(fmt::layer())
        .init();

    // Create and run server
    let server = Server::new(config);
    server.run().await?;

    Ok(())
}
