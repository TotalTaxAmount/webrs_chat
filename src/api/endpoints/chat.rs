use std::{
  collections::HashMap,
  fs::File,
  io::{Read, Seek, Write},
  path::Path,
  time::{SystemTime, UNIX_EPOCH},
};

use log::{error, info, warn};
use rand::Rng;
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

  fn new_message(req: Request<'a>, tokens: HashMap<String, String>) -> Option<Response<'a>> {
    let req_parsed: Value = match serde_json::from_slice(&req.get_data()) {
      Ok(v) => v,
      Err(_) => {
        warn!("[Request {}] Invalid json", req.get_id());
        return Some(Response::basic(400, "Bad Request"));
      }
    };

    let user = match req_parsed["user"].as_str() {
      Some(s) => s,
      None => return Some(Response::basic(400, "Bad Request")),
    };

    let content = match req_parsed["content"].as_str() {
      Some(s) => s,
      None => return Some(Response::basic(400, "Bad Request")),
    };

    let token = match req_parsed["token"].as_str() {
      Some(s) => s,
      None => return Some(Response::basic(400, "Bad Request")),
    };

    if token
      != match tokens.get(user) {
        Some(t) => t.as_str(),
        None => {
          error!("[Request {}] User {} not logged in", req.get_id(), user);
          let mut res = Response::new(401, "application/json");
          res.set_data(
            to_string(&json!({
              "error": "Not logged in"
            }))
            .unwrap()
            .into_bytes(),
          );
          return Some(res);
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

    Some(Response::basic(200, "OK"))
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

    let mut rng = rand::thread_rng();

    match req.get_endpoint().rsplit_once("/") {
      Some(s) if s.1 == "send" => return Chat::<'r>::new_message(req, self.tokens.clone()),
      Some(s) if s.1 == "auth" => {
        // TODO: Put this into different function
        let req_json: Value = match serde_json::from_slice(&req.get_data()) {
          Ok(p) => p,
          Err(e) => {
            return None;
          }
        };

        let auth_type = match req_json["type"].as_str() {
          Some(t) => t,
          None => {
            error!("[Request {}] No auth type in request", req.get_id());
            return Some(
              Response::from_json(
                400,
                json!({
                  "error": "No auth type given"
                }),
              )
              .unwrap(),
            );
          }
        };

        let username = match req_json["user"].as_str() {
          Some(t) => t,
          None => {
            error!("[Request {}] No username in request", req.get_id());
            return Some(
              Response::from_json(
                400,
                json!({
                  "error": "No user given"
                }),
              )
              .unwrap(),
            );
          }
        };

        let password = match req_json["password"].as_str() {
          Some(t) => t,
          None => {
            warn!("[Request {}] No password in request", req.get_id());
            ""
          }
        };

        let mut f = match File::options()
          .write(true)
          .read(true)
          .open(Path::new(AUTH_FILE))
        {
          Ok(f) => f,
          Err(e) => {
            warn!("[Request {}] Error opening file: {}", req.get_id(), e);
            return Some(Response::basic(500, "Internal Server Error"));
          }
        };

        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        f.flush().unwrap();

        let auth_file_json: Value = match serde_json::from_slice(&buf) {
          Ok(v) => v,
          Err(e) => {
            warn!("[Request {}] Failed to parse json: {}", req.get_id(), e);
            return Some(Response::basic(400, "Bad Request"));
          }
        };

        match auth_type {
          "new" => {
            let users = auth_file_json["users"]
              .as_array()
              .unwrap_or(&Vec::new())
              .clone();
            for f in users {
              if f["user"] == username {
                warn!(
                  "[Request {}] User '{}' already exists",
                  req.get_id(),
                  username
                );

                return Some(
                  Response::from_json(
                    422,
                    json!({
                      "error": "User already exists"
                    }),
                  )
                  .unwrap(),
                );
              }
            }

            let user = json!({
              "user": req_json["user"],
              "password": req_json["password"]
            });

            let mut users = auth_file_json["users"]
              .as_array()
              .unwrap_or(&Vec::new())
              .clone();
            users.insert(users.len(), user);

            f.set_len(0).unwrap();
            f.seek(std::io::SeekFrom::Start(0)).unwrap();
            f.write_all(
              to_string(&json!({
                "users": users
              }))
              .unwrap()
              .as_bytes(),
            )
            .unwrap();
            f.flush().unwrap();

            Some(
              Response::from_json(
                200,
                json!({
                  "successes": format!("Created new user {}", req_json["user"].as_str().unwrap())
                }),
              )
              .unwrap(),
            )
          }
          "login" => {
            if !req_json["user"].as_str().is_some() || !req_json["password"].as_str().is_some() {
              warn!("[Request {}] No username or password", req.get_id());
              return Some(
                Response::from_json(
                  400,
                  json!({
                     "error": "No username or password"
                  }),
                )
                .unwrap(),
              );
            }

            let users = auth_file_json["users"]
              .as_array()
              .unwrap_or(&Vec::new())
              .clone();
            for user in users {
              if user["user"].as_str() == Some(username)
                && user["password"].as_str() == Some(password)
              {
                let random_data: &[u8; 16] = &rng.gen();
                let token = random_data
                  .iter()
                  .map(|b| format!("{:02x}", b))
                  .collect::<String>();

                self.tokens.insert(
                  req_json["user"].as_str().unwrap().to_string(),
                  token.clone(),
                );

                return Some(
                  Response::from_json(
                    200,
                    json!({
                      "token": token
                    }),
                  )
                  .unwrap(),
                );
              }
            }
            return Some(
              Response::from_json(
                401,
                json!({
                  "error": "Invalid username or password"
                }),
              )
              .unwrap(),
            );
          }

          "check" => {
            let token = match self.tokens.get(username) {
              Some(t) => t,
              None => {
                info!(
                  "[Request {}] User '{}' not logged in",
                  req.get_id(),
                  username
                );
                return Some(
                  Response::from_json(
                    401,
                    json!({
                      "error": format!("User {} is not logged in", username)
                    }),
                  )
                  .unwrap(),
                );
              }
            };

            if token == req_json["token"].as_str().unwrap_or("") {
              return Some(
                Response::from_json(
                  200,
                  json!({
                    "successes": "Token is valid"
                  }),
                )
                .unwrap(),
              );
            }
            Some(
              Response::from_json(
                401,
                json!({
                  "error": "Token is invalid"
                }),
              )
              .unwrap(),
            )
          }

          _ => {
            return Some(
              Response::from_json(
                400,
                json!({
                  "error": format!("Unknown auth type {}", auth_type)
                }),
              )
              .unwrap(),
            );
          }
        }
      }
      s => {
        error!("[Request {}] '{:?}' is unrecognized", req.get_id(), s);
        return Some(Response::basic(400, "Bad Request"));
      }
    }
  }
}
