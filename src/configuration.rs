use std;
use std::io::Read;
use rustc_serialize::json;

#[derive(RustcDecodable)]
pub struct Credentials {
  pub username: String,
  pub password: String,
}

pub fn root_dir() -> std::path::PathBuf {
  let mut path = std::env::home_dir().unwrap();
  path.push(".config/neubauten/");
  return path;
}

pub fn spotify_path() -> String {
  let mut path = std::env::home_dir().unwrap();
  path.push(".config/neubauten/tmp/");
  return String::from(path.to_str().unwrap());
}

pub fn read_credentials() -> Credentials {
  let mut configuration_path = root_dir();
  configuration_path.push("init.json");
  let path = configuration_path.to_str().unwrap();

  match std::fs::File::open(path) {
    Ok(mut file) => {
      let mut data = String::new();
      file.read_to_string(&mut data).unwrap();
      return json::decode(&data).unwrap();
    },
    Err(_) => panic!("No configuration file found, aborting..."),
  }
}
