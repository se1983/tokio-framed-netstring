use tokio_util::codec::{Decoder, Encoder, Framed};
use bytes::{BytesMut, BufMut};
use std::io::{Error, ErrorKind};
use tokio::net::{TcpListener};

#[allow(unused_imports)]
use futures::sink::SinkExt;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;


// https://docs.rs/tokio-util/0.6.7/tokio_util/codec/index.html
fn check(left: usize, right: usize, data: &[u8]) -> Result<(), Error> {
    if let Ok(num_string) = String::from_utf8(Vec::from(&data[..left])) {
        if let Ok(exp_size) = num_string.parse::<usize>() {
            if right > left && (right - 1) - left == exp_size {
                return Ok(());
            }
        }
    }
    Err(Error::new(ErrorKind::InvalidData, "Invalid data"))
}


fn extract_frameborders(src: &BytesMut) -> Option<(usize, usize)> {
    let left_idx = src.iter().position(|&b| b == b':');
    let right_idx = src.iter().position(|&b| b == b',');

    match (left_idx, right_idx) {
        (Some(l), Some(r)) => Some((l, r)),
        _ => None
    }
}

struct NetStringCodec {}

impl NetStringCodec {
    pub(crate) fn extract_frame(&self, src: &mut BytesMut) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some((lhs, rhs)) = extract_frameborders(src) {
            check(lhs, rhs, src)?;
            let data = src.split_to(rhs + 1); // <- modify src
            let data = String::from_utf8(Vec::from(&data[lhs + 1..rhs]))?;
            return Ok(Some(data));
        }
        Ok(None)
    }
}

impl Decoder for NetStringCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        println!("Trying to decode {:?}", src);

        match self.extract_frame(src) {
            Ok(frame) => Ok(frame),
            Err(_) => Err(
                Error::new(ErrorKind::InvalidData, "Could not parse data")
            )
        }
    }
}


impl Encoder<String> for NetStringCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = format!("{}:{},", item.len(), item);
        dst.put(data.as_bytes());
        Ok(())
    }
}

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

    let mut stream = TcpStream::connect("127.0.0.1:7979").await?;
    // Write some data.
    stream.write_all(b"5:hello,").await?;
    stream.write_all(b"5:hello,").await?;

    let _ = serv_handle.await;

    Ok(())
}
