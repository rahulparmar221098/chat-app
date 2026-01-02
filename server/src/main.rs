use clap::Parser;
use server::server::ServerChat;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    let server = Arc::new(ServerChat::new());
    tracing::info!("Server running on 127.0.0.1:{}", args.port);

    while let Ok((stream, _addr)) = listener.accept().await {
        let server_clone = Arc::clone(&server);

        tokio::spawn(async move {
            let _ = server_clone.new_connection(stream).await;
        });
    }

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    /// Port to listen on
    #[arg(short, long)]
    port: u16,
}
