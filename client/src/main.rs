use clap::Parser;
use client::client::ClientChat;
use tokio::io::{self, AsyncBufReadExt};
use utils::message::Message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let server_addr = &format!("{}:{}", args.host, args.port);
    let client = ClientChat::connect(server_addr, &args.username).await?;

    // Terminal interaction.
    println!("Enter command (send <MSG> or leave): ");
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let command = Command::from_input(&line);

        match command {
            Command::Send(msg) => {
                client.send(Message::MSG(args.username.clone(), msg).to_string());
            }
            Command::Leave => {
                client.send(Message::LEAVE(args.username.clone()).to_string());
                break;
                // exit(0);
            }
            Command::Invalid => {
                // Handle invalid command
                println!(
                    "Invalid command. Use 'send <MSG>' to send a message or 'leave' to disconnect."
                );
            }
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'o', long)]
    host: String,
    #[arg(short, long)]
    port: String,
    #[arg(short, long)]
    username: String,
}

#[derive(Debug)]
enum Command {
    Send(String),
    Leave,
    Invalid,
}

impl Command {
    fn from_input(input: &str) -> Command {
        let trimmed = input.trim();
        if trimmed.starts_with("send ") {
            let msg = trimmed[5..].to_string();
            Command::Send(msg)
        } else if trimmed == "leave" {
            Command::Leave
        } else {
            Command::Invalid
        }
    }
}
