use list;
use rustify;

pub enum NeubautenView {
  PlaylistView(list::ListComponent<rustify::Playlist>),

  #[allow(dead_code)]
  TrackView(usize, rustify::Playlist, list::ListComponent<rustify::Track>),
}
