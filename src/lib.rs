use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};
use std::io::{Error, ErrorKind};


#[allow(unused_imports)]
use futures::sink::SinkExt;


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

pub struct NetStringCodec {}

impl NetStringCodec {
    pub fn extract_frame(&self, src: &mut BytesMut) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some((lhs, rhs)) = extract_frameborders(src) {
            let data = src.split_to(rhs + 1); // <- modify src
            check(lhs, rhs, src)?;
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
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Could not parse data"))
        }
    }
}


impl Encoder<String> for NetStringCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        println!("Encoding {:?}", item);
        let data = format!("{}:{},", item.len(), item);
        println!("Encoded data: {:?}", data);
        dst.put(data.as_bytes());
        Ok(())
    }
}

