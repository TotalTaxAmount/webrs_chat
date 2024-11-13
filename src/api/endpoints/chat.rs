use std::{fs::File, io::{Read, Seek, Write}, path::Path, time::{SystemTime, UNIX_EPOCH}};

use log::warn;
use serde_json::{json, to_string, Value};

use crate::{api::Method, Response};

const CHAT_HISTORY_FILE: &str = "history.json";

pub struct Chat<'a> {
  endpoint: &'a str,
}

impl<'a> Chat<'a> {
  pub fn new(endpoint: &'a str) -> Self {
    Self { endpoint }
  }
}

impl<'a> Method for Chat<'a> {
  fn get_endpoint(&self) -> &str {
    self.endpoint
  }

  fn handle_get<'s, 'r>(&'s self, req: crate::Request<'r>) -> Option<crate::Response<'r>> where 'r: 's  {
    let mut his_file = match File::open(Path::new(CHAT_HISTORY_FILE)) {
      Ok(f) => f,
      Err(e) => {
        warn!("[Request {}] Failed to open file with error '{}'", req.get_id(), e);
        return Some(Response::basic(500, "Internal Server Error"));
      },
    };

    let mut buf: Vec<u8> = Vec::new();
    let _ = his_file.read_to_end(&mut buf);

    let mut res = Response::new(200, "application/json");
    res.set_data(buf);
    Some(res)
  }

  fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>> where 'r: 's {
    if req.get_data().len() == 0 {
      warn!("[Request {}] No data" , req.get_id());
      return Some(Response::basic(400, "Bad Request (No data)"));
    }

    let req_parsed: Value = match serde_json::from_slice(&req.get_data()) {
        Ok(v) => v,
        Err(_) => {
          warn!("[Request {}] Invalid json", req.get_id());
          return Some(Response::basic(400, "Bad Request"));
        },
    };

    let mut his_file = match File::options()
      .read(true)
      .write(true)
      .open(Path::new(CHAT_HISTORY_FILE)) {
        Ok(f) => f,
        Err(e) => {
          warn!("[Request {}] Failed to open file with error '{}'", req.get_id(), e);
          return Some(Response::basic(500, "Internal Server Error"));
        },
    };

    let mut buf: Vec<u8> = Vec::new();
    let _ = his_file.read_to_end(&mut buf);

    let current: Value = match serde_json::from_slice(&buf) {
      Ok(v) => v,
      Err(e) => {
        warn!("[Request {}] Failed to parse current json: '{}'", req.get_id(), e);
        return Some(Response::basic(500, "Internal Server Error"));
      },
    };

    /*
    {
      "messages": [
        {
          "user": "test",
          "content": "hello!",
          "timestamp": 1862816946
        }
      ]
    }
    */
    
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut messages = current["messages"].as_array().unwrap_or(&Vec::<Value>::new()).clone();
    messages.insert(messages.len(), json!({
      "user": req_parsed["user"],
      "content": req_parsed["content"],
      "timestamp": timestamp
    }));

    let new_json = json!({ "messages": messages });
    let final_json = to_string(&new_json).unwrap();
    
    his_file.set_len(0).unwrap();
    his_file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Ensure the cursor is at the start
    his_file.write(final_json.as_bytes()).unwrap();
    his_file.flush().unwrap();

    Some(Response::basic(200, "OK"))
  }
}