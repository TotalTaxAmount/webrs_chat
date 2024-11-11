use std::{char::from_u32, collections::HashMap};

use json::object;
use log::{trace, warn};
use uid::{Id, IdU16};

use crate::{api::Method, Response};

pub struct FileUpload<'a> {
  pub endpoint: &'a str,
  pub files: HashMap<u16, (&'a str, Vec<u8>)>
}

impl<'f> Method<'f> for FileUpload<'f> {
  fn get_endpoint(&self) -> &str {
    return self.endpoint;
  }

  fn handle_get<'s, 'r>(&'s self, req: crate::Request<'r>) -> Option<crate::Response<'r>> where 'f: 'r {
    if !req.get_endpoint().split("/").any(|f| f == "download") {
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
  
  // curl -X POST http://localhost:8080/api/file/upload -d 
  fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>> where 'f: 'r {
    if !req.get_endpoint().ends_with("upload") {
      return Some(Response::basic(400, "Bad Request"));
    }

    let content_type = match req.get_headers().get("Content-Type") {
      Some(t) => t,
      None => {
        warn!("[Request {}] No 'Content-Type' header", req.get_id());
        "text/plain"
      },
    };

    if req.get_data().len() == 0 {
      warn!("[Request {}] No data", req.get_id());
      return Some(Response::basic(400, "Bad Request (No data)"));
    }

    let id = IdU16::<Self>::new().get();
    self.files.insert(id, (content_type, req.get_data()));
    
    let mut res: Response = Response::new(200, "text/json");

    res.set_data_as_slice(object! {
        id: id
      }.take().to_string().as_bytes()
    );

    Some(res)
  }
}