mod handlers;

use core::str;
use std::{collections::HashMap, io::Write, net::TcpStream};

use log::error;
use uid::Id;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReqTypes {
    GET,
    POST
}
#[derive(Debug, Clone)]
pub struct Request<'a> {
  pub req_type: ReqTypes,
  pub endpoint: &'a str,
  pub headers: HashMap<&'a str, &'a str>,
  id: Id<Self>
}

#[derive(Debug, Clone)]
pub struct Response<'a> {
  code: u16,
  content_type: &'a str,
  data: Vec<u8>,
  headers: HashMap<&'a str, &'a str>,
}

impl<'a> Response<'a> { 
  pub fn new(code: u16, content_type: &'a str) -> Self {
    Self {
      code,
      content_type,
      data: Vec::new(),
      headers: HashMap::new(),
    }
  }

  pub fn set_data(&mut self, data: Vec<u8>) {
    self.data = data;
  }

  pub fn add_header(&mut self, k: &'a str, v: &'a str) {
    self.headers.insert(k, v);
  }

  pub fn set_code(&mut self, code: u16) {
    self.code = code;
  }

  pub fn set_content_type(&mut self, content_type: &'a str) {
    self.content_type = content_type;
  }

  pub fn get_code(&self) -> u16 {
    self.code
  }

  pub fn get_content_type(&self) -> &'a str {
    self.content_type
  }

  pub fn get_headers(&self) -> HashMap<&'a str, &'a str> {
    self.headers.clone()
  }
}

impl<'a> Request<'a> {
    // TODO: make Request::parse return a result and include error codes
    pub fn parse(request: &'a [u8]) -> Option<Self> {
      let req_string: &str = str::from_utf8(&request).unwrap();
      let parts: Vec<&str> = req_string.split('\n').collect();
          
      if parts.is_empty() {
        error!("Invalid request");
        return None;
      }

      let base: Vec<&str> = parts[0].split(' ').collect();
      if base.len() < 2 {
        error!("Invalid request len");
        return None;
      }

      Some(Self {
        req_type: match base[0] {
            "GET" => ReqTypes::GET,
            "POST" => ReqTypes::POST,
            _ => {
              error!("Unknown http method: {}", base[0]);
              return None;
            }
        },
        endpoint: base[1],
        headers: parts[1..]
          .into_iter()
          .filter_map(|f| {
            let mut s = f.split(": ");
            if let (Some(key), Some(value)) = (s.next(), s.next()) {
              Some((key.trim(), value.trim()))
            } else {
              None
            }
          }).collect(),
        id: Id::new()
      })      
    }

    pub fn get_headers(&self) -> HashMap<&'a str, &'a str> {
      self.headers.clone()
    } 

    pub fn get_id(&self) -> usize {
      <Id<Request<'_>> as Clone>::clone(&self.id).get()
    }
}

pub fn respond(mut stream: TcpStream, mut res: Response) {
  let mut data = format!(
    "HTTP/1.1 {} OK\r\n",
    res.code
  ).as_bytes().to_vec();

  if !res.headers.contains_key("Content-Type") {
    res.headers.insert("Content-Type", res.content_type);
  }

  if !res.headers.contains_key("Content-Length") {
    let dl = res.data.len().to_string();
    res.headers.insert("Content-Length", Box::leak(dl.into_boxed_str()));
  }

  for (k, v) in res.headers {
      let h = format!("{}: {}\r\n", k, v);
      data.extend_from_slice(&h.as_bytes());
  }

  data.extend_from_slice(&b"\r\n".to_vec());
  data.extend_from_slice(&res.data);

  // println!("Data: {:?}", data);

  // println!("Res: {:?}", str::from_utf8(&mut data.clone()).unwrap());

  let _ = stream.write_all(&data);
  let _ = stream.flush();
}

