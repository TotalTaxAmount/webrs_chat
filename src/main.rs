use std::{collections::HashMap, io::Read, net::{TcpListener, TcpStream}, vec};
use std::fs::File;

use flate2::{read::GzEncoder, Compression};
use web_srv::{respond, ReqTypes, Request, Response};

fn handle(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
  let mut request: Vec<u8> = Vec::new();
  let mut buf: [u8; 4096] = [0; 4096];
  while !request.windows(4).any(|w| w == b"\r\n\r\n") {
      let len = stream.read(&mut buf)?;
      request.extend_from_slice(&buf[..len]);
  }

  let request = Request::parse(request.as_slice());

  if request.is_none() {
    println!("[ERROR] Invalid request");
    return Ok(());
  } 

  let mut path = String::from(request.as_ref().unwrap().endpoint.trim_end_matches(".html"));

  if request.as_ref().unwrap().req_type != ReqTypes::GET {
    println!("[ERROR] {:?} is not supported", request.as_ref().unwrap().req_type);
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
      let mut headers: HashMap<&str, &str> = HashMap::new();

      let _ = f.read_to_end(&mut res_data);

      if request.as_ref().unwrap().headers.contains_key("Accept-Encoding") && (request.unwrap().headers.get("Accept-Encoding").unwrap().contains("gzip") || request.unwrap().headers.get("Accept-Encoding").unwrap().contains("x-gzip")) {
        println!("[INFO] Using gzip");
        let mut encoder = GzEncoder::new(res_data.as_slice(), Compression::default());

        let mut response: Vec<u8> = Vec::new();
        
        let _ = encoder.read_to_end(&mut response);
        headers.insert("Content-Encoding", "gzip");
        res_data = response;
      } 
      let res = Response {
        code: 200,
        content_type: match content_type {
          "png" => "image/png",
          "jpg" => "image/jpeg",
          "html" => "text/html",
          "css" => "text/css",
          "mp4" => "video/mp4",
          _ => {
            println!("[ERROR] Unknown content type {}", content_type);
            "text/plain"
          }
        },
        data: res_data,
        headers
      };
      respond(stream, res);
    },
    Err(_) => {
      println!("[ERROR 404] {} not found", path);
      let res = Response {
        code: 404,
        content_type: "text/html",
        data: String::from("
        <html>
          <body>
            <h1>404 Not found</h1>
          </body>
        </html>").as_bytes().to_vec(),
        headers: HashMap::new()
      };
      respond(stream, res);
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
