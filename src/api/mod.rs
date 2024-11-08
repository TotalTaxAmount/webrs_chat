use crate::{Request, Response};

pub mod api;
pub mod endpoints;

pub trait Method {
    fn get_endpoint(&self) -> &str;

    fn handle_get(&self, req: Request) -> Option<Response>;

    fn handle_post(&mut self, req: Request) -> Option<Response>;
}