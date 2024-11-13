use std::{
  collections::HashMap,
  fs::File,
  io::{Read, Seek, Write},
  path::Path,
  time::{SystemTime, UNIX_EPOCH},
};

use log::warn;
use serde_json::{json, to_string, Value};

use crate::{api::Method, Request, Response};

const CHAT_HISTORY_FILE: &str = "history.json";
const AUTH_FILE: &str = "auth.json";

pub struct Chat<'a> {
  endpoint: &'a str,
  tokens: HashMap<String, String>,
}

impl<'a> Chat<'a> {
  pub fn new(endpoint: &'a str) -> Self {
    Self {
      endpoint,
      tokens: HashMap::new(),
    }
  }

  fn get_messages(req: Request<'a>) -> Option<Response<'a>> {
    let mut his_file = match File::open(Path::new(CHAT_HISTORY_FILE)) {
      Ok(f) => f,
      Err(e) => {
        warn!(
          "[Request {}] Failed to open file with error '{}'",
          req.get_id(),
          e
        );
        return Some(Response::basic(500, "Internal Server Error"));
      }
    };

    let mut buf: Vec<u8> = Vec::new();
    let _ = his_file.read_to_end(&mut buf);

    let mut res = Response::new(200, "application/json");
    res.set_data(buf);
    Some(res)
  }

  fn new_message(req: Request<'a>) -> Option<Response<'a>> {
    let req_parsed: Value = match serde_json::from_slice(&req.get_data()) {
      Ok(v) => v,
      Err(_) => {
        warn!("[Request {}] Invalid json", req.get_id());
        return Some(Response::basic(400, "Bad Request"));
      }
    };

    let mut his_file = match File::options()
      .read(true)
      .write(true)
      .open(Path::new(CHAT_HISTORY_FILE))
    {
      Ok(f) => f,
      Err(e) => {
        warn!(
          "[Request {}] Failed to open file with error '{}'",
          req.get_id(),
          e
        );
        return Some(Response::basic(500, "Internal Server Error"));
      }
    };

    let mut buf: Vec<u8> = Vec::new();
    let _ = his_file.read_to_end(&mut buf);

    let current: Value = match serde_json::from_slice(&buf) {
      Ok(v) => v,
      Err(e) => {
        warn!(
          "[Request {}] Failed to parse current json: '{}'",
          req.get_id(),
          e
        );
        return Some(Response::basic(500, "Internal Server Error"));
      }
    };

    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();
    let mut messages = current["messages"]
      .as_array()
      .unwrap_or(&Vec::<Value>::new())
      .clone();
    messages.insert(
      messages.len(),
      json!({
        "user": req_parsed["user"],
        "content": req_parsed["content"],
        "timestamp": timestamp
      }),
    );

    let new_json = json!({ "messages": messages });
    let final_json = to_string(&new_json).unwrap();

    his_file.set_len(0).unwrap();
    his_file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Ensure the cursor is at the start
    his_file.write(final_json.as_bytes()).unwrap();
    his_file.flush().unwrap();

    Some(Response::basic(200, "OK"))
  }

  fn handle_auth(req: Request<'a>) -> Option<Response<'a>> {
    let req_json: Value = match serde_json::from_slice(&req.get_data()) {
      Ok(p) => p,
      Err(e) => {
        return None;
      }
    };

    let mut f = match File::options()
      .write(true)
      .read(true)
      .open(Path::new(AUTH_FILE))
    {
      Ok(f) => f,
      Err(e) => todo!(),
    };

    let mut buf: Vec<u8> = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    let parsed: Value = match serde_json::from_slice(&buf) {
      Ok(v) => v,
      Err(_) => todo!(),
    };

    match parsed["type"].as_str() {
      Some(t) if t == "new" => {
        let users = parsed["users"].as_array()?;
        if users.into_iter().all(|f| {
          f.as_array()
            .unwrap_or(&Vec::new())
            .get(0)
            .unwrap_or(&Value::String("".to_string()))
            .as_str()
            == req_json["username"].as_str()
        }) {
          warn!(
            "[Request {}] User '{}' already exists",
            req.get_id(),
            req_json["username"].as_str().unwrap()
          );
          let mut res = Response::new(422, "application/json");
          res.set_data(
            to_string(&json!({
              "error": "User already exists"
            }))
            .unwrap()
            .into_bytes(),
          );
          return Some(res);
        }
        None
      }
      Some(t) if t == "login" => todo!(),
      Some(_) | None => {
        return Some(Response::basic(400, "Bad Request"));
      }
    }
  }
}

impl<'a> Method for Chat<'a> {
  fn get_endpoint(&self) -> &str {
    self.endpoint
  }

  fn handle_get<'s, 'r>(&'s self, req: crate::Request<'r>) -> Option<crate::Response<'r>>
  where
    'r: 's,
  {
    match req.get_endpoint().rsplit("/").next() {
      Some("messages") => return Chat::<'r>::get_messages(req),
      _ => {
        return Some(Response::basic(404, "Not Found"));
      }
    }
  }

  fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>>
  where
    'r: 's,
  {
    if req.get_data().len() == 0 {
      warn!("[Request {}] No data", req.get_id());
      return Some(Response::basic(400, "Bad Request (No data)"));
    }

    match req.get_endpoint().rsplit('/').next() {
      Some("send") => return Chat::<'r>::new_message(req),
      Some("auth") => return Chat::<'r>::handle_auth(req),
      _ => return Some(Response::basic(404, "Not Found")),
    }
  }
}
