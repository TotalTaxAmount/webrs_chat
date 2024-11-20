use std::{
  collections::HashMap,
  fs::File,
  io::{Read, Seek, Write},
  path::Path,
};

use log::{error, info, warn};
use serde_json::{json, to_string, Value};
use sha2::{Digest, Sha256};

use rand::Rng;
use webrs::{request::Request, response::Response};

const AUTH_FILE: &str = "auth.json";

pub(super) fn handle_auth<'r>(
  tokens: &mut HashMap<String, String>,
  req: Request<'r>,
) -> Option<Response<'r>> {
  let mut rng = rand::thread_rng();

  let req_json: Value = match serde_json::from_slice(&req.get_data()) {
    Ok(p) => p,
    Err(e) => {
      error!(
        "[Request {}] Failed to parse request json: {}",
        req.get_id(),
        e
      );
      return Some(
        Response::from_json(
          400,
          json!({
            "error": "Invalid json"
          }),
        )
        .unwrap(),
      );
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

  let password: Option<&str> = req_json["password"].as_str();

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
      return Some(Response::basic(500, "Internal Server Error"));
    }
  };

  match auth_type {
    "new" => {
      // Create new user
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

      let mut hasher = Sha256::new();
      hasher.update(password.unwrap());
      let hashed_password = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
      let user = json!({
        "user": req_json["user"],
        "password":  hashed_password
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
            "success": format!("Created new user {}", req_json["user"].as_str().unwrap())
          }),
        )
        .unwrap(),
      )
    }
    "login" => {
      // Login existing user
      if !password.is_some() {
        warn!("[Request {}] No password", req.get_id());
        return Some(
          Response::from_json(
            400,
            json!({
               "error": "No password"
            }),
          )
          .unwrap(),
        );
      }

      let registered_users = auth_file_json["users"]
        .as_array()
        .unwrap_or(&Vec::new())
        .clone();

      let mut hasher = Sha256::new();
      hasher.update(password.unwrap());
      let hashed_password = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

      for user in registered_users {
        if user["user"].as_str() == Some(username)
          && user["password"].as_str() == Some(&hashed_password)
        {
          let random_bytes: &[u8; 16] = &rng.gen();
          let token = random_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

          tokens.insert(
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
      let token = match tokens.get(username) {
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
              "success": "Token is valid"
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
