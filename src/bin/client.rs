use std::println;

use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(32);
    
    let sender_clone = sender.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set { key: "hello".to_string(), value: "world".into(), resp: resp_tx };
        sender.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("GOT: {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "hello".to_string(), resp: resp_tx };
        sender_clone.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("GOT: {:?}", res);
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        use Command::*;
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                Set {key, value, resp} => {
                    let res = client.set(&key, value).await;

                    let _ = resp.send(res);
                },
                Get {key, resp} => {
                    let res = client.get(&key).await;

                    let _ = resp.send(res);
                }
            }
        }
    });
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}


type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
enum Command {
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    }
}