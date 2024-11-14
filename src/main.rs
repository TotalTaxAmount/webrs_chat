pub mod api;
pub mod handlers;

use std::{f32::INFINITY, io::SeekFrom, net::SocketAddr, sync::Arc, time::Duration};

use log::{info, trace, warn};
// use stdnet::{SocketAddr, TcpListener, TcpStream}, thread::sleep, time::Duration};

use handlers::Handlers;
use tokio::{
  io::AsyncReadExt,
  net::{TcpListener, TcpStream},
  sync::Mutex,
  time::sleep,
};
use web_srv::{respond, ReqTypes, Request, Response};

async fn handle(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
  let (mut r_stream, w_stream) = stream.split();
  let w_stream = Arc::new(Mutex::new(w_stream));

  loop {
    let mut raw: Vec<u8> = Vec::new();
    let mut buf: [u8; 4096] = [0; 4096];
    while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
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

      raw.extend_from_slice(&buf[..len]);
    }

    let req: Request = match Request::parse(raw.as_slice()) {
      Ok(r) => r,
      Err(e) => {
        respond(
          w_stream.clone(),
          Response::basic(e.get_code(), e.get_description()),
        )
        .await;
        continue;
      }
    };

    let req_id = req.get_id();

    info!(
      "[Request {}] from {}: {:?} {} HTTP/1.1",
      req_id,
      addr.ip(),
      req.get_type(),
      req.get_endpoint()
    );

    let res = Handlers::handle_request(req.clone()).await;

    if let Some(r) = res {
      respond(w_stream.clone(), r).await;
    } else {
      warn!("[Request {}] No response", req_id);
      respond(w_stream.clone(), Response::basic(400, "Bad Request")).await;
    }

    if let Some(c) = req.get_headers().get("Connection") {
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
