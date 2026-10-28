use std::{collections::HashMap, sync::{Arc, Mutex}};
use bytes::Bytes;
use mini_redis::Frame;
use my_redis::Connection;
use tokio::net::{TcpListener, TcpStream};
use my_redis::Result;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();

        println!("Accept!");
        tokio::spawn(async move {
            if let Err(e) = process(socket, db).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}

async fn process(socket: TcpStream, db: Db)-> Result<()> {
    use mini_redis::Command::{self, Set, Get};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await? {
        let response = match Command::from_frame(frame)? {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            },  
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            },
            cmd => Frame::Error(format!("ERR unimplemented command: {cmd:?}")),
        };
        connection.write_frame(&response).await?;
    }
    Ok(())
}