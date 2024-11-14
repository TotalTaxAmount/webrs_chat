pub mod api;
pub mod handlers;

use core::{fmt, str};
use std::{
  collections::HashMap,
  fmt::{Display, Error},
  sync::Arc,
};

use log::{error, trace};
use serde_json::{to_string, Value};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::Mutex};
use uid::Id;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReqTypes {
  GET,
  POST,
  OPTIONS,
}

#[derive(Debug, Clone)]
pub struct ResError<'r> {
  code: u16,
  description: &'r str,
}

impl<'r> fmt::Display for ResError<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Req error! {} {}", self.code, self.description)
  }
}

impl ResError<'_> {
  pub fn get_code(&self) -> u16 {
    self.code
  }

  pub fn get_description(&self) -> &str {
    self.description
  }
}
#[derive(Debug, Clone)]
pub struct Request<'a> {
  req_type: ReqTypes,
  content_type: &'a str,
  endpoint: &'a str,
  data: Vec<u8>,
  headers: HashMap<&'a str, &'a str>,
  id: Id<Self>,
}

#[derive(Debug, Clone)]
pub struct Response<'a> {
  code: u16,
  content_type: String,
  data: Vec<u8>,
  headers: HashMap<&'a str, &'a str>,
  id: Id<Self>,
}

impl<'a> Response<'a> {
  pub fn new(code: u16, content_type: &'a str) -> Self {
    Self {
      code,
      content_type: content_type.to_string(),
      data: Vec::new(),
      headers: HashMap::new(),
      id: Id::new(),
    }
  }

  pub fn set_data(&mut self, data: Vec<u8>) {
    self.data = data;
  }

  pub fn get_data(&self) -> Vec<u8> {
    self.data.clone()
  }

  pub fn set_data_as_slice(&mut self, data: &[u8]) {
    self.data = data.to_vec();
  }

  pub fn add_header(&mut self, k: &'a str, v: &'a str) {
    self.headers.insert(k, v);
  }

  pub fn set_code(&mut self, code: u16) {
    self.code = code;
  }

  pub fn set_content_type(&mut self, content_type: String) {
    self.content_type = content_type;
  }

  pub fn get_code(&self) -> u16 {
    self.code
  }

  pub fn get_content_type(&self) -> String {
    self.content_type.clone()
  }

  pub fn get_headers(&self) -> HashMap<&'a str, &'a str> {
    self.headers.clone()
  }

  pub fn basic(code: u16, description: &str) -> Self {
    let http = format!(
      "
      <html>
        <body>
          <h1>{} {}</h1>
        <body>
      </html>
    ",
      code, description
    );

    let mut res = Self::new(code, "text/html");
    res.set_data(http.as_bytes().to_vec());

    res
  }

  pub fn from_json(code: u16, json: Value) -> Result<Self, serde_json::Error> {
    let mut res = Self::new(code, "application/json");
    let json_string = match to_string(&json) {
      Ok(s) => s,
      Err(e) => {
        error!("Failed to stringify json: {}", e);
        return Err(e);
      }
    };

    res.set_data(json_string.into_bytes());

    Ok(res)
  }
}

impl Display for Request<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{:?} {} HTTP/1.1", self.get_type(), self.get_endpoint())?;
    for h in self.headers.clone() {
      writeln!(f, "{}: {}", h.0, h.1)?;
    }
    writeln!(f)?;
    write!(
      f,
      "{}",
      String::from_utf8(self.get_data()).unwrap_or("[Not utf8]".to_string())
    )?;

    return writeln!(f);
  }
}

impl<'a> Request<'a> {
  // TODO: make Request::parse return a result and include error codes
  pub fn parse(request: &'a [u8]) -> Result<Self, ResError> {
    let header_body_split = b"\r\n\r\n";
    let split_index = request
      .windows(header_body_split.len())
      .position(|w| w == header_body_split);

    let (header_bytes, body_bytes) = match split_index {
      Some(i) => (&request[..i], &request[i + header_body_split.len()..]),
      None => {
        error!("Invalid request");
        return Err(ResError {
          code: 400,
          description: "Bad Request",
        });
      }
    };
    let header_str: &str = str::from_utf8(&header_bytes).unwrap();
    let parts: Vec<&str> = header_str.split('\n').collect();

    if parts.is_empty() {
      error!("Invalid request");
      return Err(ResError {
        code: 400,
        description: "Bad Request",
      });
    }

    let base: Vec<&str> = parts[0].split(' ').collect();
    if base.len() < 2 {
      error!("Invalid request length");
      trace!("Request string: {}", header_str);
      return Err(ResError {
        code: 400,
        description: "Bad Request",
      });
    }

    let headers: HashMap<&str, &str> = parts[1..]
      .into_iter()
      .filter_map(|f| {
        let mut s = f.split(": ");
        if let (Some(key), Some(value)) = (s.next(), s.next()) {
          Some((key.trim(), value.trim()))
        } else {
          None
        }
      })
      .collect();

    Ok(Self {
      req_type: match base[0] {
        "GET" => ReqTypes::GET,
        "POST" => ReqTypes::POST,
        "OPTIONS" => ReqTypes::OPTIONS,
        _ => {
          error!("Unknown http method: {}", base[0]);
          return Err(ResError {
            code: 501,
            description: "Not Implemented",
          });
        }
      },
      endpoint: base[1],
      headers: headers.clone(),
      id: Id::new(),
      content_type: headers.get("Content-Type").or(Some(&"none")).unwrap(),
      data: body_bytes.to_vec(),
    })
  }

  pub fn get_type(&self) -> ReqTypes {
    self.req_type
  }

  pub fn get_content_type(&self) -> &str {
    &self.content_type
  }

  pub fn get_endpoint(&self) -> &str {
    &self.endpoint
  }

  pub fn get_data(&self) -> Vec<u8> {
    self.data.clone()
  }

  pub fn get_headers(&self) -> HashMap<&'a str, &'a str> {
    self.headers.clone()
  }

  pub fn get_id(&self) -> usize {
    <Id<Request<'_>> as Clone>::clone(&self.id).get()
  }
}

pub async fn respond(stream: Arc<Mutex<WriteHalf<'_>>>, mut res: Response<'_>) {
  let mut stream = stream.lock().await;
  let mut data = format!("HTTP/1.1 {} OK\r\n", res.code).as_bytes().to_vec();

  if !res.headers.contains_key("Content-Type") {
    res
      .headers
      .insert("Content-Type", res.content_type.as_str());
  }

  if !res.headers.contains_key("Content-Length") {
    let dl = res.data.len().to_string();
    res
      .headers
      .insert("Content-Length", Box::leak(dl.into_boxed_str()));
  }

  for (k, v) in res.headers {
    let h = format!("{}: {}\r\n", k, v);
    data.extend_from_slice(&h.as_bytes());
  }

  data.extend_from_slice(&b"\r\n".to_vec());
  data.extend_from_slice(&res.data);

  // println!("Res: {:?}", str::from_utf8(&mut data.clone()).unwrap());

  let _ = stream.write_all(&data).await;
  let _ = stream.flush().await;
}
