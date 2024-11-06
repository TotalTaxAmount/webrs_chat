use std::{io::Write, net::TcpStream};

#[derive(Debug)]
pub struct Request {
  req_type: ReqTypes,
  endpoint: String,
  headers: Vec<(String, String)>
}

#[derive(Debug)]
pub enum ReqTypes {
    GET,
    POST
}

impl Request {
    pub fn parse(request: String) -> Option<Self> {
      println!("{}", request);
      None
    }
}

pub fn respond(mut stream: TcpStream, data: &[u8], code: i32, content_type: &str) {
  let mut header = format!(
    "HTTP/1.1 {} OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
    code,
    data.len(),
    content_type
  ).as_bytes().to_vec();

  header.append(&mut data.to_vec());

  let _ = stream.write_all(header.as_slice());
  let _ = stream.flush();
}

