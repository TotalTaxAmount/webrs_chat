use crate::{api::Method, Request, Response};

pub struct ApiTest<'a> {
  pub x: u8,
  pub endpoint: &'a str
}

impl<'a> Method for ApiTest<'a> {
    fn get_endpoint(&self) -> &str {
        self.endpoint
    }

    fn handle_get<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
      let mut res = Response::new(200, "text/plain");
      res.set_data_as_slice(self.x.to_string().as_bytes());
      Some(res)
    }

    fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> {
      self.x = String::from_utf8(req.get_data()).unwrap().parse::<u8>().unwrap();
      Some(Response::basic(200, "OK"))
    }
}
