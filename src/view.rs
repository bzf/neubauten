use list;
use rustify;

pub enum NeubautenView {
  PlaylistView(list::List<rustify::Playlist>),

  #[allow(dead_code)]
  TrackView(rustify::Playlist, list::List<rustify::Track>),

  SearchView(rustify::Search, list::List<rustify::Track>),
}
