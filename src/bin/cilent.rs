use bytes::Bytes;
use mini_redis::{client, Result};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(32);
    
    let sender_clone = sender.clone();

    let t1 = tokio::spawn(async move {
        let cmd = Command::Set { key: "hello".to_string(), value: "world".into() };
        sender.send(cmd).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let cmd = Command::Get { key: "hello".to_string() };
        sender_clone.send(cmd).await.unwrap();
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        use Command::*;
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                Set {key, value} => {
                    client.set(&key, value).await.unwrap();
                },
                Get {key} => {
                    client.get(&key).await.unwrap();
                }
            }
        }
    });
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

enum Command {
    Set {
        key: String,
        value: Bytes
    },
    Get {
        key: String
    }
}