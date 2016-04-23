#![deny(warnings)]

extern crate rustbox;
extern crate rustify;
extern crate libc;
extern crate rustc_serialize;

extern {
  static g_appkey: *const u8;
  static g_appkey_size: libc::size_t;
}

mod configuration;
mod list;

use std::default::Default;
use std::sync::{Arc, Mutex};

use rustbox::{RustBox};
use rustbox::Key;
use rustbox::{Color};

fn get_application_key() -> Vec<u8> {
  let ptr = &g_appkey as *const _ as *const u8;
  let slice = unsafe { std::slice::from_raw_parts(ptr, g_appkey_size) };
  return slice.to_vec();
}

enum NeubautenView {
  PlaylistView(list::ListComponent<rustify::Playlist>),
  TrackView(usize, rustify::Playlist, list::ListComponent<rustify::Track>),
}

enum NeubautenAction {
  NextView(NeubautenView),

  PlayTrack,
  PlayNextTrack,

  StartReadingCommand,
  StopReadingCommand,
  AddCharacterToCommand(char),
  PopCharacterFromCommand,

  QueueTrack,

  MoveDown,
  MoveUp,

  Back,
  Quit,

  Noop,
}

enum PlaybackTrack {
  QueueTrack(rustify::Track),

  /// Contains `playlist_index`, `track_index` and the actual track
  PlaylistTrack(usize, usize, rustify::Track),
}

fn main() {
  // Create the configuration directory (if it doesn't exist)
  std::fs::create_dir_all(configuration::root_dir()).unwrap();

  let player = Arc::new(Mutex::new(rustify::OpenALPlayer::new()));
  let spotify_path = configuration::spotify_path();
  let (mut session, receiver) = rustify::Session::new(
    get_application_key(),
    &spotify_path,
    &spotify_path,
    "neubauten",
    player,
  );

  // Sign in to Spotify
  let credentials = configuration::read_credentials();
  session.login(&credentials.username, &credentials.password);

  // Wait until we've signed in
  loop {
    let event = receiver.recv().unwrap();

    if let rustify::Event::LoggedIn = event {
      break
    }
  }

  let rustbox = match RustBox::init(Default::default()) {
    Result::Ok(v) => v,
    Result::Err(e) => panic!("{}", e),
  };

  let height = rustbox.height();
  let width = rustbox.width();

  let playlists = session.playlists();

  let initial_view: NeubautenView = NeubautenView::PlaylistView(
    list::ListComponent::new(playlists, height - 2, width)
  );

  // Keep around views we have been on when going into another so that we can
  // easily implement back later on.
  let mut views: Vec<NeubautenView> = Vec::new();
  views.push(initial_view);

  let mut playback_queue: Vec<rustify::Track> = Vec::new();
  let mut current_track: Option<PlaybackTrack> = None;
  let mut current_filter: Option<String> = None;
  let mut is_reading_command: bool = false;

  // Listen to events
  loop {
    // Print the current view somehow
    rustbox.clear();
    print_view(views.last_mut().unwrap(), &current_filter, is_reading_command, &rustbox);
    print_status_bar(&current_track, &rustbox);
    print_command_bar(&current_filter, &rustbox);
    rustbox.present();

    // Wait for a action
    let action = get_next_action(views.last().unwrap(), &rustbox, &session, is_reading_command, &receiver);

    // Process that action
    match action {
      NeubautenAction::NextView(view) => {
        current_filter = None;
        is_reading_command = false;

        views.push(view)
      },
      NeubautenAction::Back => {
        current_filter = None;

        if current_filter.is_some() {
          current_filter = None;
          is_reading_command = false;
        } else if views.len() > 1 {
          views.pop();
        }
      },
      NeubautenAction::PlayTrack => {
        match views.last().unwrap() {
          &NeubautenView::TrackView(playlist_index, ref playlist, ref list) => {
            let track_index = list.get_selected_index();
            let track = playlist.track(track_index as i32).unwrap();
            session.play_track(&track);

            let playlist_track = PlaybackTrack::PlaylistTrack(
              playlist_index,
              track_index,
              track,
            );

            current_track = Some(playlist_track);
            ()
          },
          _ => (),
        }
      },
      NeubautenAction::PlayNextTrack => {
        if !playback_queue.is_empty() {
          let next_track = playback_queue.remove(0);
          session.play_track(&next_track);
          current_track = Some(PlaybackTrack::QueueTrack(next_track));
        } else {
          match current_track {
            Some(t) => {
              match t {
                PlaybackTrack::PlaylistTrack(playlist_index, track_index, _) => {
                  let playlist = session.playlist(playlist_index as i32).unwrap();
                  let next_track = playlist.track((track_index + 1) as i32);

                  // Pick the next track in the playlist and play that
                  match next_track {
                    Some(track) => {
                      session.play_track(&track);

                      current_track = Some(
                        PlaybackTrack::PlaylistTrack(
                          playlist_index,
                          track_index + 1,
                          track,
                        )
                      );
                    },
                    None => {
                      current_track = None;
                    }
                  }
                },
                _ => {
                  current_track = None;
                },
              }
            },
            None => (),
          }
        }
      },
      NeubautenAction::MoveUp => {
        match views.last_mut().unwrap() {
          &mut NeubautenView::TrackView(_, _, ref mut list) => list.handle_up(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.handle_up(),
        }
      },
      NeubautenAction::MoveDown => {
        match views.last_mut().unwrap() {
          &mut NeubautenView::TrackView(_, _, ref mut list) => list.handle_down(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.handle_down(),
        }
      },
      NeubautenAction::QueueTrack => {
        match views.last_mut().unwrap() {
          &mut NeubautenView::TrackView(_, ref playlist, ref mut list) => {
            let track_index = list.get_selected_index();
            let track = playlist.track(track_index as i32).unwrap();
            playback_queue.push(track);
            ()
          },
          _ => (),
        }
      },
      NeubautenAction::StartReadingCommand => {
        current_filter = Some("".to_string());
        is_reading_command = true;
      },
      NeubautenAction::StopReadingCommand => {
        is_reading_command = false;
      },
      NeubautenAction::AddCharacterToCommand(character) => {
        let mut value: String = current_filter.unwrap();
        value.push(character);
        current_filter = Some(value);
      },
      NeubautenAction::PopCharacterFromCommand => {
        let mut value: String = current_filter.unwrap();

        if !value.is_empty() {
          value.pop();
        }

        current_filter = Some(value);
      },
      NeubautenAction::Quit => {
        break
      },
      NeubautenAction::Noop => (),
    }
  }
}

fn print_view(view: &mut NeubautenView, filter: &Option<String>, filter_updated: bool, rustbox: &rustbox::RustBox) {
  match view {
    &mut NeubautenView::PlaylistView(ref mut list) => {
      list.print(&rustbox, 0, 0, filter, filter_updated);
    },
    &mut NeubautenView::TrackView(_, _, ref mut list) => {
      list.print(&rustbox, 0, 0, filter, filter_updated);
    }
  }
}

fn print_status_bar(current_track: &Option<PlaybackTrack>,
                    rustbox: &rustbox::RustBox) {
  let y_pos = rustbox.height() - 2;
  let width = rustbox.width();

  let playback: String = match current_track {
    &Some(ref t) => {
      match t {
        &PlaybackTrack::QueueTrack(ref track) => {
          let minutes: u64 = track.duration().as_secs() / 60;
          let seconds: u64 = track.duration().as_secs() % 60;
          format!("Playback: {} [{}:{seconds:>0width$}]", track.to_string(), minutes, seconds=seconds, width=2)
        },
        &PlaybackTrack::PlaylistTrack(_, _, ref track) => {
          let minutes: u64 = track.duration().as_secs() / 60;
          let seconds: u64 = track.duration().as_secs() % 60;
          format!("Playback: {} [{}:{seconds:>0width$}]", track.to_string(), minutes, seconds=seconds, width=2)
        }
      }
    },
    &None => "Playback: -".to_string(),
  };

  let number_of_spaces = width - playback.len();
  let mut value: String = playback.clone();

  for _ in 0..number_of_spaces {
    value.push(' ');
  }

  rustbox.print(0, y_pos, rustbox::RB_BOLD, Color::White, Color::Cyan, &value);
}

fn get_next_action(current_view: &NeubautenView,
                   rustbox: &rustbox::RustBox,
                   session: &rustify::Session,
                   reading_command: bool,
                   rustify_channel: &std::sync::mpsc::Receiver<rustify::Event>) -> NeubautenAction {
  let height = rustbox.height();
  let width = rustbox.width();

  // If the rustify channel has pending events, deal with those first
  let rustify_event = rustify_channel.try_recv();
  if rustify_event.is_ok() {
    return match rustify_event.unwrap() {
      rustify::Event::EndOfTrack => NeubautenAction::PlayNextTrack,
      _ => NeubautenAction::Noop,
    };
  }

  // maybe we should loop and peek for events?
  let rustbox_event = rustbox.peek_event(std::time::Duration::from_millis(100), false);
  if rustbox_event.is_ok() {
    let event = rustbox_event.unwrap();

    if reading_command {
      match event {
        rustbox::Event::KeyEvent(key) => {
          match key {
            Key::Char(character) => return NeubautenAction::AddCharacterToCommand(character),
            Key::Backspace => return NeubautenAction::PopCharacterFromCommand,
            Key::Enter => return NeubautenAction::StopReadingCommand,
            _ => (),
          }
        },
        _ => (),
      }
    }

    return match event {
      rustbox::Event::KeyEvent(key) => {
        match key {
          Key::Enter => {
            match current_view {
              &NeubautenView::PlaylistView(ref list) => {
                let playlist_index = list.get_selected_index();
                let playlist = session.playlist(playlist_index as i32).unwrap();

                let tracks = playlist.tracks();

                let next_view = NeubautenView::TrackView(
                  playlist_index,
                  playlist,
                  list::ListComponent::new(tracks, height - 1, width),
                );

                NeubautenAction::NextView(next_view)
              },
              &NeubautenView::TrackView(_, _, _) => NeubautenAction::PlayTrack
            }
          },
          Key::Char('j') => NeubautenAction::MoveDown,
          Key::Char('k') => NeubautenAction::MoveUp,
          Key::Char('q') => {
            match current_view {
              &NeubautenView::TrackView(_, _, _) => NeubautenAction::QueueTrack,
              _ => NeubautenAction::Noop,
            }
          },
          Key::Char('>') => NeubautenAction::PlayNextTrack,
          Key::Char('e') => NeubautenAction::Quit,
          Key::Char('/') => NeubautenAction::StartReadingCommand,
          Key::Esc => NeubautenAction::Back,
          _ => {
            NeubautenAction::Noop
          }
        }
      },
      _ => {
        NeubautenAction::Noop
      },
    };
  }

  return NeubautenAction::Noop;
}

fn print_command_bar(filter: &Option<String>, rustbox: &rustbox::RustBox) {
  let y_pos = rustbox.height() - 1;

  let command = match filter {
    &Some(ref f) => {
      let mut value = "Search: ".to_string();
      value.push_str(f);
      value
    },
    &None => "".to_string(),
  };

  rustbox.print(0, y_pos, rustbox::RB_NORMAL, Color::Black, Color::Default, &command);
}
