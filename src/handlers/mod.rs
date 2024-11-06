use std::{collections::HashMap, io::Read, usize};

use flate2::{bufread::{DeflateEncoder, GzEncoder, ZlibEncoder}, Compression};
use log::{info, trace, warn};
use zstd::Encoder;

use crate::Request;

pub mod get;
pub mod post;

pub fn handle_encoding<'a>(req: Request<'a>, mut buf: Vec<u8>) -> (Vec<u8>, Option<&'a str>) {
  if !req.get_headers().contains_key("Accept-Encoding") {
    info!("Request {} does not support encoding", req.get_id());
    return (buf, None);
  }

  let mut compression_types: Vec<&str> = req.get_headers().get("Accept-Encoding")
    .unwrap()
    .split(", ")
    .collect();

  let mut algorithm = None;

  let order = ["zstd", "br", "gzip"];
  let order_map: HashMap<&str, usize> = order.into_iter().enumerate().map(|(i, s)| (s, i)).collect();
  compression_types.sort_by_key(|a: &&str| order_map.get(a).copied().unwrap_or(usize::MAX));
  trace!("{:?}", compression_types);
  for compression_type in compression_types {
    match compression_type {
      "gzip" => {
        algorithm = Some(compression_type);

        let mut read_buf: Vec<u8> = Vec::new();
        let mut e = GzEncoder::new(buf.as_slice(), Compression::fast());

        let _ = e.read_to_end(&mut read_buf);

        buf = read_buf;
        break;
      }
      "zstd" => {
        algorithm = Some(compression_type);

        buf = zstd::encode_all(buf.as_slice(), 3).unwrap();
        break;
      }
      "br" => {
        let mut read_buf: Vec<u8> = Vec::new();
        let mut e = brotli::CompressorReader::new(buf.as_slice(), 4096, 11, 21);
        
        let _ = e.read_to_end(&mut read_buf);
        
        buf = read_buf;
      }
      _ => {
        warn!("Unsupported compression algorithm {} for request {}", compression_type, req.get_id());
      }
    }
  }

  (buf, algorithm)

}