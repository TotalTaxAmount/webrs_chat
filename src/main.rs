mod endpoints;
use std::{fs::File, io::{Read, Write}, path::Path, process::exit, sync::Arc, vec};

use chat_test::Config;
use endpoints::chat::Chat;
use log::{error, warn};
use tokio::sync::Mutex;
use webrs::{api::ApiMethod, server::WebrsHttp};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  if let Err(_) = std::env::var("SERVER_LOG") {
    std::env::set_var("SERVER_LOG", "info");
  }

  pretty_env_logger::formatted_timed_builder()
    .parse_env("SERVER_LOG")
    .format_timestamp_millis()
    .init();

  let api_methods: Vec<Arc<Mutex<dyn ApiMethod + Send + Sync>>> = vec![Chat::new("/chat")];
  
  let set_default = !Path::new("config.toml").exists();

  let mut config_file = File::options()
    .create(true)
    .read(true)
    .write(true)
    .append(true)
    .open(Path::new("config.toml"))
    .unwrap();

  if set_default {
    let toml_string = toml::to_string(&Config::default()).unwrap();
    config_file.write_all(toml_string.as_bytes()).unwrap();
    warn!("No config file! Created default");
    exit(0);
  }


  let mut buf: Vec<u8> = Vec::new();
  config_file.read_to_end(&mut buf).unwrap();
  let config: Config = toml::from_str(&String::from_utf8(buf).unwrap()).unwrap();

  let http = WebrsHttp::new(
    api_methods,
    config.port,
    (config.compression.zstd, config.compression.br, config.compression.gzip),
    config.content_dir,
  );

  match http.start().await {
    Ok(_) => {},
    Err(e) => {
      error!("Failed to start http server: {}", e);
      exit(0)
    },
  };
  
  Ok(())
}
