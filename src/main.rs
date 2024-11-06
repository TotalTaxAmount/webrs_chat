use core::str;
use std::{alloc::System, io::Read, net::{TcpListener, TcpStream}};
use std::fs::File;

use web_srv::respond;

fn handle(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
  let mut buff: [u8; 1024] = [0; 1024];
  let _ = stream.read(&mut buff);
  let req: Vec<&str> = str::from_utf8(&buff).unwrap().split(' ').collect();

  if req.len() < 2 {
    println!("[Error] wrong request len");
    return Ok(());
  } 

  let req_type = req[0];
  let mut path = String::from(req[1].trim_end_matches(".html"));


  if req_type != "GET" {
    println!("[Error 403] Unsupported request type: {}", req_type);
    respond(stream, String::from("
      <html>
        <body>
          <h1>403 Forbidden</h1>
        <body>
      <html>
    ").as_bytes(), 403, "text/html");
    return Ok(());
  }

  if path.ends_with('/') {
   path.push_str("index"); 
  }

  let content_type = if let Some(dot_pos) = path.rfind('.') { &path[(dot_pos + 1)..] } else { "html" };
  let name = format!("{}.{}", &path[0..path.find(".").unwrap_or(path.len())], content_type);

  let mut f = File::open(format!("./content/{}", name));


  match &mut f {
    Ok(f) => { 
      let mut res_data: Vec<u8> = vec![];
      let _ = f.read_to_end(&mut res_data); 
      respond(stream, res_data.as_slice(), 200, match content_type { // TODO: This could be better
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "html" => "text/html",
        "css" => "text/css",
        "mp4" => "video/mp4",
        "js" => "text/javascript",
        _ => {
          println!("[Error] unknown content type {}", content_type);
          return Ok(());
        }
      });
    },
    Err(_) => {
      println!("[Error 404] {} not found", path);
      respond(stream, 
        String::from("
        <html>
          <body>
            <h1>404 Not found</h1>
          </body>
        </html>").as_bytes(), 404, "text/html");     
    },
  }
  Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    
    for s in listener.incoming() {
      let _ = handle(s?);
    }
    Ok(())
}
