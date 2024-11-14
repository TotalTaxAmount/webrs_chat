mod auth;

use std::{
  collections::HashMap,
  fs::File,
  io::{Read, Seek, Write},
  path::Path,
  time::{SystemTime, UNIX_EPOCH},
};

use auth::handle_auth;
use log::{error, info, warn};
use serde_json::{json, to_string, Value};

use crate::{api::Method, Request, Response};

const CHAT_HISTORY_FILE: &str = "history.json";

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

  fn handle_send_message(
    req: Request<'a>,
    tokens: HashMap<String, String>,
  ) -> Option<Response<'a>> {
    let req_parsed: Value = match serde_json::from_slice(&req.get_data()) {
      Ok(v) => v,
      Err(_) => {
        warn!("[Request {}] Invalid json", req.get_id());
        return Some(
          Response::from_json(
            400,
            json!({
              "error": "Invalid request json"
            }),
          )
          .unwrap(),
        );
      }
    };

    let user = match req_parsed["user"].as_str() {
      Some(s) => s,
      None => {
        return Some(
          Response::from_json(
            400,
            json!({
              "error": "No user field"
            }),
          )
          .unwrap(),
        )
      }
    };

    let content = match req_parsed["content"].as_str() {
      Some(s) => s,
      None => {
        return Some(
          Response::from_json(
            400,
            json!({
              "error": "No content field"
            }),
          )
          .unwrap(),
        )
      }
    };

    let token = match req_parsed["token"].as_str() {
      Some(s) => s,
      None => {
        return Some(
          Response::from_json(
            400,
            json!({
              "error": "No token field"
            }),
          )
          .unwrap(),
        )
      }
    };

    if token
      != match tokens.get(user) {
        Some(t) => t.as_str(),
        None => {
          error!("[Request {}] User {} not logged in", req.get_id(), user);
          return Some(Response::from_json(401, json!({
            "error": "User not logged in"
          })).unwrap());
        }
      }
    {
      let mut res = Response::new(401, "application/json");
      res.set_data(
        to_string(&json!({
          "error": "Invalid token"
        }))
        .unwrap()
        .into_bytes(),
      );
      return Some(res);
    }

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
        "user": user,
        "content": content,
        "timestamp": timestamp
      }),
    );

    let new_json = json!({ "messages": messages });
    let final_json = to_string(&new_json).unwrap();

    his_file.set_len(0).unwrap();
    his_file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Ensure the cursor is at the start
    his_file.write(final_json.as_bytes()).unwrap();
    his_file.flush().unwrap();

    Some(Response::from_json(200, json!({
      "success": "message sent"
    })).unwrap())
  }
}

impl<'a> Method for Chat<'a> {
  fn get_endpoint(&self) -> &str {
    self.endpoint
  }

  /// Handle GET method for chat api
  /// 
  /// **/api/chat/messages**:
  /// 
  /// **Example**: GET /api/chat/messages
  /// 
  /// **Responses**:
  /// - *500*: If the server returns an error
  /// - *200*: ```{"messages": [ { "user": "<username>", "content": "<message content>", "timestamp": <unix timestamp mills> }, <...> ]}``` -> List of all the sent messages
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

  /// Handle POST method for chat api
  ///
  /// **/api/chat/auth -> Authentication**:
  ///
  /// **Example**:```{"type": "login", "user": "<username>", "password": "<password>"}```
  ///
  /// **Errors**:
  /// - *400*:
  ///   - ```{"error": "No user given"}``` if there is no "user" field
  ///   - ```{"error": "No auth type given"}``` if there is no "type" field
  /// - *500*: If the server encounters an error
  ///
  /// **Types**:
  /// - *new*: Create a new user, example request: ```{"type": "new", "user": "<username>", "password": <password>}``` possible responses:
  ///   - *422*: ```{"error": "User already exists"}``` -> A user with the same username already exists
  ///   - *200*: ```{"success": "Created new user <username>"}``` -> Successfully created a new user
  /// - *login*: Login to an existing user: ```{"type": "login", "user": "<username>", "password": "<password>"}``` possible responses:
  ///   - *400*: ```{"error": "No password"}``` -> No password provided
  ///   - *401*: ```{"error": "Invalid username or password"}``` -> Username or password is invalid
  ///   - *200*: ```{"token": "<token>"}``` -> The username and password are correct *Note*: tokens are needed to send messages
  /// - *check*: Check if a token is valid ```{"type": "check", "user", "<username": "<token>"}``` possible responses:
  ///   - *401*: ```{"error": "User <username> is not logged in"}``` -> The user does not have a valid token.  
  ///   - *401* {"error": "Token is invalid"} -> The token does not match the users token
  ///   - *200* {"success": "Token is valid"} -> The token is valid for the user
  ///
  /// **/api/chat/send -> Send a message**:
  ///
  /// **Example**: ```{"user": "<username>", "content": "<message content>", "token": "<token>"}```
  ///
  /// **Responses**:
  /// - 400: ```{"error": "Invalid request json"}``` -> The requests json failed to parse
  /// - 400: ```{"error": "No [user/token/content] field"}``` -> There is no [user/token/content] field
  /// - 401: ```{"error": "User not logged in"}``` -> The user does not have a token and needs to login
  /// - 401: ```{"error", "Invalid token"}``` -> The token is invalid
  /// - 500: Internal Server Error
  /// - 200: ```{"success": "message sent"}``` -> The message was sent successfully
  fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>>
  where
    'r: 's,
  {
    if req.get_data().len() == 0 {
      warn!("[Request {}] No data", req.get_id());
      return Some(Response::basic(400, "Bad Request (No data)"));
    }

    match req.get_endpoint().rsplit_once("/") {
      Some(s) if s.1 == "send" => return Chat::<'r>::handle_send_message(req, self.tokens.clone()),
      Some(s) if s.1 == "auth" => handle_auth(&mut self.tokens, req),
      s => {
        error!("[Request {}] '{:?}' is unrecognized", req.get_id(), s);
        return Some(Response::basic(400, "Bad Request"));
      }
    }
  }
}
