use tokio_util::codec::{Decoder, Encoder, Framed};
use bytes::{BytesMut, BufMut};
use std::io::{Error, ErrorKind};
use tokio::net::{TcpListener};

#[allow(unused_imports)]
use futures::sink::SinkExt;
// enables next() on Framed
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;


// https://docs.rs/tokio-util/0.6.7/tokio_util/codec/index.html

struct NetStringCodec {}

impl NetStringCodec {


    pub(crate) fn extract_frame(&self, delimiter_pos: usize, src: &mut BytesMut) -> Result<String, Box<dyn std::error::Error>> {

        let length = &src[..delimiter_pos];
        let expected_data_length = String::from_utf8(Vec::from(length)).unwrap().parse::<usize>().unwrap();
        let rhs_pos = delimiter_pos + expected_data_length + 1;
        if (src.len() as usize) < rhs_pos {
            return Err("Incomplete".into());
        }
        let data = src.split_to(rhs_pos); // <- modify src
        Ok(String::from_utf8(Vec::from(&data[delimiter_pos + 1..])).unwrap())
    }
}

impl Decoder for NetStringCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        println!("Trying to decode {:?}", src);
        if let Some(delimiter_idx) = src.iter().position(|&b| b == b':') {
            let result = match self.extract_frame(delimiter_idx, src) {
                Ok(frame) => Some(frame),
                Err(err) if err.to_string() == "Incomplete" => { None }
                _ => return Err(Error::new(
                    ErrorKind::Other, "Something went wrong parsing data")
                )
            };
            return Ok(result);
        }
        Ok(None)
    }
}

impl Encoder<String> for NetStringCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = format!("{}:{}", item.len(), item);
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
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let serv_handle = tokio::spawn({server()});

    let mut stream = TcpStream::connect("127.0.0.1:7979").await?;
    // Write some data.
    stream.write_all(b"5:hello").await?;

    serv_handle.await;

    Ok(())


}
