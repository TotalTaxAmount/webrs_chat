use crate::{Request, Response};

pub mod api;
pub mod endpoints;

pub trait Method<'f>: Send + Sync {
    fn get_endpoint(&self) -> &str;

    fn handle_get<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> where 'f: 'r ;

    fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> where 'f: 'r;
}