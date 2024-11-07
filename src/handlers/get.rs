
use std::{fs::File, io::Read};

use log::{error, trace, warn};

use crate::{ReqTypes, Request, Response};

use super::handle_encoding;

pub fn handle_get(req: Request) -> Option<Response> {
  if req.req_type != ReqTypes::GET {
    warn!("[Request {}] Request method is {:?} not GET", req.get_id(), req.req_type);
    return None;
  }

  let mut path: String = req.endpoint.to_string();
  

  if path.ends_with('/') {
   path.push_str("index"); 
  }

  let content_type = if let Some(dot_pos) = path.rfind('.') { &path[(dot_pos + 1)..] } else { "html" };
  let f_name = format!("{}.{}", &path[0..path.find(".").unwrap_or(path.len())], content_type);

  let mut f = File::open(format!("./content/{}", f_name));
  let mut res = Response::new(200, "text/html");

  let mime_type = Box::leak(mime_guess::from_path(f_name.clone()).first().unwrap().essence_str().to_string().into_boxed_str()); // Holy shit


  match &mut f {
    Ok(f) => { 
      let mut res_data: Vec<u8> = vec![];

      let _ = f.read_to_end(&mut res_data);
      
      let req_id = req.get_id();
      let final_data = handle_encoding(req, res_data);
      
      if final_data.1.is_some() {
        trace!("[Request {}] Using '{}' compression", req_id,  final_data.1.as_ref().unwrap());
        res.add_header("Content-Encoding", final_data.1.unwrap());
      }
      res.set_code(200);
      res.set_content_type(mime_type);
      res.set_data(final_data.0);

    },
    Err(_) => {
      error!("404 {} not found", path);
      res.set_code(404);
      res.set_data("
      <html>
        <body>
          <h1>404 Not found</h1>
        </body>
      </html>".as_bytes().to_vec());
    }
  }

  Some(res)
}