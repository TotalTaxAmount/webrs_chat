mod handlers;
use log::{info, warn};
use std::{io::Read, net::{SocketAddr, TcpListener, TcpStream}};

use handlers::{get::handle_get, post::handle_post};
use web_srv::{respond, ReqTypes, Request, Response};


fn handle(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
  let mut request: Vec<u8> = Vec::new();
  let mut buf: [u8; 4096] = [0; 4096];
  while !request.windows(4).any(|w| w == b"\r\n\r\n") {
      let len = stream.read(&mut buf)?;
      request.extend_from_slice(&buf[..len]);
  }

  let req = Request::parse(request.as_slice());

  if req.is_none() {
    println!("[ERROR] Invalid request");
    return Ok(());
  } 

  info!("Request {} from {}: {:?} HTTP/1.1 {}", req.as_ref().unwrap().get_id(), addr.ip(), req.as_ref().unwrap().req_type, req.as_ref().unwrap().endpoint);

  let res: Option<Response<'_>>;

  match req.as_ref().unwrap().req_type {
      ReqTypes::GET => res = handle_get(req.unwrap()),
      ReqTypes::POST => res = handle_post(req.unwrap()),
  }
  
  if res.is_none() {
    warn!("No response");
    return Ok(());
  }

  respond(stream, res.unwrap());
 
  Ok(())
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    info!("Started listening on port 8080");
    
    // while let Ok((stream, _)) = listener.accept() {
    //   tokio::spawn(async move {
    //     let _ = handle(stream);
    //   });
    // }

    while let Ok((stream, addr)) = listener.accept() {
      let _ = handle(stream, addr);
    }
    
    Ok(())
}
