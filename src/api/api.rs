use std::{collections::HashMap, sync::Arc};

use log::{error, trace};
use tokio::sync::Mutex;

use crate::{ReqTypes, Request, Response};

use super::{endpoints::file_upload::FileUpload, Method};

#[derive(Clone)]
pub struct Api {
  api_methods: Vec<Arc<Mutex<dyn Method + Send + Sync>>>
}

impl Api {
  pub fn new() -> Self {
      Api {
        api_methods: vec![
          Arc::new(Mutex::new(FileUpload { endpoint: "/file" }))
        ]
      }
  }

  pub async fn handle_api_request<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> {
    let endpoint = match req.get_endpoint().split_once("/api") {
      Some(s) if s.1 != "" => s.1,
      _ => {
        return Some(Response::basic(400, "Bad Request"));
      },
    };
  
    trace!("Endpoint: {}", endpoint);
    
    let mut res: Option<Response>;
    
    for m in &self.api_methods {
      let mut locked_m = match m.try_lock() {
        Ok(m) => m,
        Err(e) => {
          error!("[Request {}] Failed to acquire lock on method: {}", req.get_id(), e);
          return None;
        }
      };
      if endpoint.starts_with(locked_m.get_endpoint()) {
        res = match req.get_type() {
          ReqTypes::GET => locked_m.handle_get(req.clone()),
          ReqTypes::POST => locked_m.handle_post(req.clone())
        };

        if res.is_some() {
          return res;
        }
      }
    }
        
    None
  }
}