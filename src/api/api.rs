use log::trace;

use crate::{api::endpoints::test::ApiTest, ReqTypes, Request, Response};

use super::Method;

#[derive(Clone)]
pub struct Api<T> where T: Method {
  pub api_methods: Vec<T>
}

impl<T> Api<T> where T: Method {
  pub fn new(methods: Vec<T>) -> Self {
    Api {
      api_methods: methods
    }
  }

  pub fn handle_api_request(req: Request) -> Option<Response> {
    let api_methods: Vec<T> = vec![
      
      // ApiTest { x: 0, endpoint: "/test"}
    ];

    let endpoint = match req.get_endpoint().split_once("/api") {
      Some(s) if s.1 != "" => s.1,
      _ => {
        return Some(Response::new(400, "text/html").with_description("Bad Request"));
      },
    };
  
    trace!("Endpoint: {}", endpoint);
    
    let mut res: Option<Response> = None;

    for mut m in api_methods.into_iter() {
      if m.get_endpoint() == endpoint {
        res = match req.get_type() {
          ReqTypes::GET => m.handle_get(req.clone()),
          ReqTypes::POST => m.handle_post(req.clone())
        };

        if res.is_some() {
          break;
        }
      }
    }
    res
  }
    
}