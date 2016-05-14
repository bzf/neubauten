extern crate gcc;

fn main() {
  gcc::compile_library("libkey.a", &["src/spotify_key.c"]);
}
