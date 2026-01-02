use anyhow::{Result, bail};
use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc::UnboundedSender};

pub struct Room {
    clients: Mutex<HashMap<String, UnboundedSender<String>>>,
}

impl Room {
    pub fn new() -> Self {
        Room {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_user(&self, username: String, sender: UnboundedSender<String>) -> Result<()> {
        let is_user_exists = self.clients.lock().await.contains_key(&username);
        if is_user_exists {
            bail!("Username not available!")
        } else {
            self.clients.lock().await.insert(username, sender);
            Ok(())
        }
    }

    pub async fn send(&self, username: &String, message: String) {
        if let Some(sender) = self.clients.lock().await.get(username) {
            let _ = sender.send(message);
        } else {
            eprintln!("User with this name {} does not exists", username)
        }
    }

    pub async fn broadcast_message(&self, message: String, username: &String) {
        let mut clients = vec![];
        self.clients.lock().await.iter().for_each(|(key, sender)| {
            if *key != *username {
                clients.push(sender.clone());
            }
        });

        clients.iter().for_each(|sender| {
            let _ = sender.send(message.clone());
        });
    }

    pub async fn remove_user(&self, username: &String) {
        self.clients.lock().await.remove(username);
    }
}

#[cfg(test)]
mod tests {

    use super::Room;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn add_user_success() {
        let room = Room::new();
        let (sender, _) = mpsc::unbounded_channel();
        let result = room.add_user("alice".to_string(), sender).await;
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn user_already_exist() {
        let room = Room::new();
        let (sender, _) = mpsc::unbounded_channel();
        let _result = room.add_user("alice".to_string(), sender.clone()).await;
        let result2 = room.add_user("alice".to_string(), sender).await;
        assert!(result2.is_err())
    }

    #[tokio::test]
    async fn remove_user() {
        let room = Room::new();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        room.add_user("alice".to_string(), tx).await.unwrap();
        room.remove_user(&"alice".to_string()).await;

        let clients = room.clients.lock().await;
        assert!(!clients.contains_key("alice"));
    }
    #[tokio::test]
    async fn send_to_non_existing_user() {
        let room = Room::new();

        // Should not panic
        room.send(&"ghost".to_string(), "msg".to_string()).await;
    }

    #[tokio::test]
    async fn broadcast_message() {
        let room = Room::new();

        let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();

        room.add_user("alice".to_string(), tx1).await.unwrap();
        room.add_user("bob".to_string(), tx2).await.unwrap();

        room.broadcast_message("hi".to_string(), &"alice".to_string())
            .await;

        assert_eq!(rx2.recv().await.unwrap(), "hi");
        assert!(rx1.try_recv().is_err());
    }
}
