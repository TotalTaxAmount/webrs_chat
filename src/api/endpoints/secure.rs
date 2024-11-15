use log::{info, error};
use serde_json::Value;
use tokio::sync::oneshot::error;

use crate::{api::Method, Response};

pub struct Secure<'a> {
  endpoint: &'a str
}

impl<'a> Secure<'a> {
  pub fn new(endpoint: &'a str) -> Self {
    Self { endpoint }
  }  
}

impl<'a> Method for Secure<'a> {
    fn get_endpoint(&self) -> &str {
        self.endpoint
    }

    fn handle_get<'s, 'r>(&'s self, req: crate::Request<'r>) -> Option<crate::Response<'r>>
      where
        'r: 's {
      None
    }

    fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>>
      where
        'r: 's {
      if req.get_data().len() == 0 {
        info!("No data");
        return None;
      };


      info!("{:?}", String::from_utf8(req.get_data()));
      Some(Response::basic(200, "Ok"))
    }
}
