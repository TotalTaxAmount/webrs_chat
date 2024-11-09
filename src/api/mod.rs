use crate::{Request, Response};

pub mod api;
pub mod endpoints;

pub trait Method: Send + Sync {
    fn get_endpoint(&self) -> &str;

    fn handle_get<'a>(&self, req: Request) -> Option<Response<'a>>;

    fn handle_post<'a>(&mut self, req: Request) -> Option<Response<'a>>;
}