use crate::{Request, Response};

pub mod api;
pub mod endpoints;

pub trait Method: Send + Sync {
    fn get_endpoint(&self) -> &str;

    fn handle_get<'a, 'b>(&'a self, req: Request<'b>) -> Option<Response<'b>>;

    fn handle_post<'a, 'b>(&'a mut self, req: Request<'b>) -> Option<Response<'b>>;
}