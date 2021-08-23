use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed};
use futures::StreamExt;

use tokio_netstring::NetStringCodec;
use futures::sink::SinkExt;


async fn server() {
    if let Ok(tcp_listener) = TcpListener::bind("127.0.0.1:7979").await {
        println!("listening with {:?}", tcp_listener);
        while let Ok((tcp_stream, sock_addr)) = tcp_listener.accept().await {
            println!("Accepted inbound from {}", sock_addr);
            tokio::spawn(async move {
                let (_, mut reader) = Framed::new(tcp_stream, NetStringCodec {}).split();
                while let Some(Ok(line)) = reader.next().await {
                    println!("Received:{:?}", line);
                }
            });
        }
        eprintln!("TCP connection was closed!");
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let serv_handle = tokio::spawn(server());

    let stream = TcpStream::connect("127.0.0.1:7979").await?;
    let mut framed_stream = Framed::new(stream, NetStringCodec {});
    // Write some data.
    framed_stream.send("hello".to_string()).await?;

    let _ = serv_handle.await;

    Ok(())
}
