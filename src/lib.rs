use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
  pub port: u16,
  pub content_dir: String,
  pub compression: Compression
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Compression {
  pub zstd: bool,
  pub br: bool,
  pub gzip: bool
}

impl Default for Config {
  fn default() -> Self {
    Self { 
      port: 8080, 
      content_dir: "content".to_string(), 
      compression: Compression {
        zstd: true,
        br: true,
        gzip: true
      }
    }
  }
}