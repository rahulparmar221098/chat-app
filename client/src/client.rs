use std::process::exit;

use futures::{SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_util::codec::{Framed, LinesCodec};
use utils::message::Message;

type MessageType = String;

pub struct ClientChat {
    sender: UnboundedSender<MessageType>,
}

impl ClientChat {
    pub async fn connect(addr: &String, username: &String) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let framed = Framed::new(stream, LinesCodec::new());
        let (mut writer, mut reader) = framed.split();

        let (sender, mut receiver) = mpsc::unbounded_channel();

        // Writer task (outgoing messages)
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                if writer.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Reader task (incoming messages)
        tokio::spawn(async move {
            while let Some(Ok(line)) = reader.next().await {
                match Message::from(line) {
                    Message::JOIN(username) => {
                        eprintln!("{} joined", username);
                    }
                    Message::MSG(username, msg) => {
                        eprintln!("{} : {}", username, msg);
                    }
                    Message::LEAVE(username) => {
                        eprintln!("{} left", username);
                    }
                    Message::ALREADYTAKEN => {
                        eprintln!("Username is not available");
                        exit(0);
                    }
                    Message::UNAUTHENTICATED => {
                        eprintln!("UNAUTHENTICATED");
                        exit(0);
                    }
                    _ => {}
                }

                // println!("{}", line);
            }
        });

        let _ = sender.send(Message::AUTH(username.clone()).to_string());
        Ok(Self { sender })
    }

    pub fn send(&self, message: String) {
        let _ = self.sender.send(message);
    }
}
