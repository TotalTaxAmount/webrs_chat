use std::collections::HashMap;

use log::warn;

use crate::{api::Method, Response};

pub struct FileUpload<'a> {
  pub endpoint: &'a str,
  pub files: HashMap<u16, (&'a str, Vec<u8>)>
}

impl Method for FileUpload<'static> {
    fn get_endpoint(&self) -> &str {
        return self.endpoint;
    }

    fn handle_get<'s, 'r>(&'s self, req: crate::Request<'r>) -> Option<crate::Response<'r>> {
      
      if !req.get_endpoint().trim_start_matches(self.get_endpoint()).starts_with("download") {
        return Some(Response::basic(400, "Bad Request"));
      }

      let id = match req.get_endpoint().rsplit('/').next().unwrap_or("").parse::<u16>() {
          Ok(i) => i,
          Err(e) => {
            warn!("[Request {}] Failed to parse u16: {}", req.get_id(), e);
            return Some(Response::basic(400, "Bad Request"));
          },
      };

      let f = match self.files.get(&id) {
        Some(d) => d,
        None => {
          warn!("[Request {}] File with id {} not found", req.get_id(), id);
          return Some(Response::basic(404, "Not Found"));
        },
      };

      let mut res: Response<'r> = Response::<'r>::new(200, f.0);
      res.set_data(f.1.clone());  
      Some(res)
    }

    fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>> {
        todo!()
    }
}