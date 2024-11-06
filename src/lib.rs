use core::str;
use std::{collections::HashMap, io::Write, net::TcpStream};


#[derive(Debug, PartialEq, Eq)]
pub enum ReqTypes {
    GET,
    POST
}
#[derive(Debug)]
pub struct Request<'a> {
  pub req_type: ReqTypes,
  pub endpoint: &'a str,
  pub headers: HashMap<&'a str, &'a str>
}

#[derive(Debug)]
pub struct Response<'a> {
  pub code: u16,
  pub content_type: &'a str,
  pub data: Vec<u8>,
  pub headers: HashMap<&'a str, &'a str>
}

impl<'a> Response<'a> { }

impl<'a> Request<'a> {
    // TODO: make Request::parse return a result and include error codes
    pub fn parse(request: &'a [u8]) -> Option<Self> {
      let req_string: &str = str::from_utf8(&request).unwrap();
      let parts: Vec<&str> = req_string.split('\n').collect();
          
      if parts.is_empty() {
        println!("[ERROR] Invalid request");
        return None;
      }

      let base: Vec<&str> = parts[0].split(' ').collect();
      if base.len() < 2 {
        println!("[ERROR] Invalid request line");
        return None;
      }

      Some(Self {
        req_type: match base[0] {
            "GET" => ReqTypes::GET,
            "POST" => ReqTypes::POST,
            _ => {
              println!("[ERROR] Unknown http method: {}", base[0]);
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
          }).collect()
      })      
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

