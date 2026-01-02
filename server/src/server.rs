use crate::room::Room;
use anyhow::{Result, bail};
use futures::{SinkExt, StreamExt, stream::SplitStream};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_util::codec::{Framed, LinesCodec};
use utils::message::Message;

pub struct ServerChat {
    room: Room,
}

impl ServerChat {
    pub fn new() -> Self {
        Self { room: Room::new() }
    }

    pub async fn new_connection(&self, stream: TcpStream) -> Result<()> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let framed = Framed::new(stream, LinesCodec::new());

        let (mut writer, mut reader) = framed.split();

        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(message) = receiver.recv().await {
                if writer.send(message).await.is_err() {
                    break;
                }
            }
        });

        let auth_username = self.authenticate_user(&mut reader, sender).await?;

        while let Some(Ok(line)) = reader.next().await {
            match Message::from(line) {
                Message::MSG(username, msg) => {
                    self.room
                        .broadcast_message(
                            Message::MSG(username.clone(), msg).to_string(),
                            &username,
                        )
                        .await;
                }
                Message::LEAVE(username) => {
                    self.room.remove_user(&username).await;
                    self.room
                        .broadcast_message(Message::LEAVE(username.clone()).to_string(), &username)
                        .await;
                }
                _ => {
                    tracing::error!("Invalid message");
                }
            }
        }

        self.room.remove_user(&auth_username).await;
        self.room
            .broadcast_message(
                Message::LEAVE(auth_username.clone()).to_string(),
                &auth_username,
            )
            .await;
        Ok(())
    }

    async fn authenticate_user(
        &self,
        reader: &mut SplitStream<Framed<TcpStream, LinesCodec>>,
        sender: UnboundedSender<String>,
    ) -> Result<String> {
        if let Some(Ok(line)) = reader.next().await {
            match Message::from(line) {
                Message::AUTH(username) => {
                    match self.room.add_user(username.clone(), sender.clone()).await {
                        Err(_) => {
                            let _ = sender.send(Message::ALREADYTAKEN.to_string());
                            bail!("Username already taken")
                        }
                        _ => {
                            self.room
                                .broadcast_message(
                                    Message::JOIN(username.clone()).to_string(),
                                    &username,
                                )
                                .await;
                            Ok(username)
                        }
                    }
                }
                _ => {
                    let _ = sender.send(Message::UNAUTHENTICATED.to_string());
                    bail!("Not able to authenticate user!")
                }
            }
        } else {
            let _ = sender.send(Message::UNAUTHENTICATED.to_string());
            bail!("Not able to authenticate user!")
        }
    }

    pub async fn close(&self) {
        self.room
            .broadcast_message("ClOSE".to_string(), &"".to_string())
            .await;
    }
}
