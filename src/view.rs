use list;
use rustify;

pub enum NeubautenView {
  PlaylistView(list::List<rustify::Playlist>),

  #[allow(dead_code)]
  TrackView(usize, rustify::Playlist, list::List<rustify::Track>),
}
