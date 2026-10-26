use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpStream};

#[tokio::main]
async fn main()-> Result<(), io::Error> {
    let socket = TcpStream::connect("127.0.0.1:6142").await.unwrap();
    let (mut rd, mut wr) = io::split(socket);

    tokio::spawn(async move {
        wr.write_all(b"hello\r\n").await.unwrap();
        wr.write_all(b"world\r\n").await.unwrap();

        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 128];

    loop {
        let n = rd.read(&mut buf).await?;

        if n == 0 {
            break;
        }

        println!("GOT {:?}", &buf[..n]);
    }
    
    Ok(())
}