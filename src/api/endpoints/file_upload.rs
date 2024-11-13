use core::str;
use std::{char::from_u32, collections::HashMap, ffi::OsStr, fs::File, io::{Read, Write}, path::Path, vec};

use json::object;
use log::{trace, warn};
use uid::{Id, IdU16};

use crate::{api::Method, Response};

pub struct FileUpload<'a> {
  pub endpoint: &'a str,
  pub files: HashMap<u16, &'a Path>
}

impl Method for FileUpload<'_> {
  fn get_endpoint(&self) -> &str {
    return self.endpoint;
  }

  fn handle_get<'s, 'r>(&'s self, req: crate::Request<'r>) -> Option<crate::Response<'r>> 
  where 
    'r: 's 
  {
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

    let mime_type = match mime_guess::from_ext(f.extension().unwrap_or(OsStr::new("")).to_str()?).first() {
      Some(t) => {
        trace!("{}", t.essence_str() );
        "text/plain"
      },
      None => {
        warn!("[Request {}] Could not get mime type for {:?}", req.get_id(), f.as_os_str().to_str());
        "text/plain"
      },
    };

    let data = match File::open(Path::new(format!("files/{}", f.as_os_str().to_str()?).as_str())) {
      Ok(mut f) => {
        let mut buf: Vec<u8> = Vec::new();
        let _ = f.read_to_end(&mut buf);
        buf
      },
      Err(e) => {
        warn!("[Request {}] Failed to read file: {}", req.get_id(), e);
        return Some(Response::basic(500, "Internal Server Error"));
      },
    };


    let mut res: Response<'r> = Response::<'r>::new(200, "text/plain");
    res.set_data(data);
    Some(res)
  }
  
  // curl -X POST http://localhost:8080/api/file/upload -d 
  fn handle_post<'s, 'r>(&'s mut self, req: crate::Request<'r>) -> Option<crate::Response<'r>> 
  where 
    'r: 's 
  {
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

    let parsed = match json::parse(&String::from_utf8(req.get_data()).unwrap()) {
      Ok(j) => j,
      Err(e) => {
        warn!("Help");
        return Some(Response::basic(400, "Bad Request"));
      },
    };

    let id = IdU16::<Self>::new().get();
    let raw_p = format!("files/{}-{}", req.get_id(), &parsed["filename"].as_str()?);

    let mut file = match File::create_new(Path::new(&raw_p.clone())) {
        Ok(f) => f,
        Err(e) => {
          warn!("{}", e);
          return None;
        },
    };
    
    let _ = file.write_all(&parsed["data"].as_str()?.as_bytes().to_vec());
    // self.files.insert(id, Path::new(&raw));    
    let mut res: Response = Response::new(200, "text/json");

    res.set_data_as_slice(object! {
        id: id
      }.take().to_string().as_bytes()
    );

    Some(res)
  }
}