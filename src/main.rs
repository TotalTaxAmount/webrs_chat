mod endpoints;
use std::{ops::Mul, sync::{Arc}, vec};

use endpoints::chat::Chat;
use tokio::sync::Mutex;
use webrs::{api::Method, WebrsHttp};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  
  let api_methods : Vec<Arc<Mutex<dyn Method + Send + Sync>>> = vec![
    Chat::new("/chat")
  ];

  let http = WebrsHttp::new(api_methods, 8080, (true, true, true), "/content".to_string());

  let _ = http.start().await;
  Ok(())
}
