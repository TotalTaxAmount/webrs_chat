mod handlers;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use log::{error, info, trace, warn};
// use std::{net::{SocketAddr, TcpListener, TcpStream}, thread::sleep, time::Duration};

use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}, sync::Mutex, time::sleep};
use handlers::{get::handle_get, post::handle_post};
use web_srv::{respond, ReqTypes, Request, Response};


async fn handle(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
  let (mut r_stream, w_stream) = stream.split();

  let w_stream = Arc::new(Mutex::new(w_stream));

  loop {
    let mut request: Vec<u8> = Vec::new();
    let mut buf: [u8; 4096] = [0; 4096];
    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
        let len = match r_stream.read(&mut buf).await {
            Ok(0) => return Ok(()),
            Ok(len) => len,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
              warn!("Would block, retrying...");
              sleep(Duration::from_secs(5)).await;
              continue;
            }
            Err(e) => {
              warn!("Read error: {}", e);
              break;
            }
        };

        request.extend_from_slice(&buf[..len]);
    }

    let req = Request::parse(request.as_slice());

    if req.is_none() {
      error!("Invalid request");
      respond(w_stream.clone(), Response::new(404, "").as_error("Not Found")).await;
      continue;
    } 

    let req_id = req.as_ref().unwrap().get_id();

    info!(
      "[Request {}] from {}: {:?} HTTP/1.1 {}", 
      req.as_ref().unwrap().get_id(), 
      addr.ip(), 
      req.as_ref().unwrap().req_type, 
      req.as_ref().unwrap().endpoint
    );

    let res = match req.as_ref().unwrap().req_type {
        ReqTypes::GET => handle_get(req.clone().unwrap()),
        ReqTypes::POST => handle_post(req.clone().unwrap())
    };
    
    if let Some(r) = res {
      respond(w_stream.clone(), r).await;
    } else {
      warn!("[Request {}] No response", req_id);
      respond(w_stream.clone(), Response::new(400, "").as_error("Bad Request")).await;
    }

    if let Some(c) = req.as_ref().unwrap().get_headers().get("Connection") {
      if c.to_ascii_lowercase() != "keep-alive" { 
        trace!("[Request {}] Connection: {}", req_id, c);
        break; 
      }
    } else {
      trace!("[Request {}] No connection header", req_id);
      break;
    }
  }

  trace!("Connection to {} closed", addr.ip());

  Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
  if let Err(_) = std::env::var("RUST_LOG") {
    std::env::set_var("RUST_LOG", "info");
  }

  pretty_env_logger::init();

  let listener = TcpListener::bind("0.0.0.0:8080").await?;
  info!("Started listening on port 8080");
  
  while let Ok((s, a)) = listener.accept().await {
    tokio::spawn(async move {
      let _ = handle(s, a).await;
    });
  }
  
  Ok(())
}
