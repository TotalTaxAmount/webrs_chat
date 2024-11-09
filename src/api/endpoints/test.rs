use crate::{api::Method, Response};

pub struct ApiTest<'a> {
  pub x: u8,
  pub endpoint: &'a str
}

impl<'a> Method for ApiTest<'a> {
    fn get_endpoint(&self) -> &str {
        self.endpoint
    }

    fn handle_get(&self, req: crate::Request) -> Option<crate::Response> {
      let mut res = Response::new(200, "text/plain");
      res.set_data(self.x.to_string().as_bytes().to_vec());
      Some(res)
    }

    fn handle_post(&mut self, req: crate::Request) -> Option<Response> {
      self.x = u8::from_be(*req.get_data().get(0).unwrap());
      Some(Response::basic(200, "OK"))
    }
}
