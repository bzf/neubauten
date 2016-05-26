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
mod action;
mod view;
mod command_parser;

use view::{NeubautenView};
use action::{Action};
use command_parser::{CommandParser};

use std::default::Default;
use std::sync::{Arc, Mutex};

use rustbox::{RustBox};
use rustbox::{Color};

fn get_application_key() -> Vec<u8> {
  let ptr = &g_appkey as *const _ as *const u8;
  let slice = unsafe { std::slice::from_raw_parts(ptr, g_appkey_size) };
  return slice.to_vec();
}

enum PlaybackTrack {
  QueueTrack(rustify::Track),

  /// Contains the playlist, track and the `track_index`
  PlaylistTrack(rustify::Playlist, rustify::Track, usize),
}

fn result_to_option<T, E>(event: Result<T, E>) -> Option<T> {
  return match event {
    Ok(e) => Some(e),
    Err(_) => None,
  }
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
    list::List::new(playlists, height - 2, width)
  );

  // Keep around views we have been on when going into another so that we can
  // easily implement back later on.
  let mut views: Vec<NeubautenView> = Vec::new();
  views.push(initial_view);

  let mut playback_queue: Vec<rustify::Track> = Vec::new();
  let mut current_track: Option<PlaybackTrack> = None;
  let mut command_parser = CommandParser::new();
  let mut last_action: action::Action = action::Action::Noop;

  // Listen to events
  loop {
    let mut current_view: NeubautenView = views.pop().unwrap();
    let mut next_view: Option<NeubautenView> = None;

    let mut action: Action = action::Action::Noop;
    let rustify_event = receiver.try_recv();
    if action::next_action(result_to_option(rustify_event)) == action::Action::Noop {
      let rustbox_event = rustbox.peek_event(std::time::Duration::from_millis(100), false);

      if rustbox_event.is_ok() {
        let command = command_parser.handle_input(&rustbox_event.unwrap());

        match command {
          command_parser::Action::NAction(neubauten_action) => {
            action = neubauten_action;
          },
          _ => (),
        }
      }
    }

    // Update the view
    rustbox.clear();
    print_view(&mut current_view, &rustbox);
    print_status_bar(&current_track, session.is_playing(), &rustbox);
    print_command_bar(&last_action, &rustbox);
    rustbox.present();

    // Process that action
    match action {
      Action::Select => {
        match &current_view {
          &NeubautenView::TrackView(ref playlist, ref list) => {
            let track = list.get_selected_item();
            session.play_track(&track);

            let playlist_track = PlaybackTrack::PlaylistTrack(
              playlist.clone(),
              track,
              list.get_selected_index(),
            );

            current_track = Some(playlist_track);
          },
          &NeubautenView::PlaylistView(ref list) => {
            let playlist = list.get_selected_item();
            let tracks = playlist.tracks();

            let next_list = list::List::new(tracks, height - 2, width - 2);
            next_view = Some(NeubautenView::TrackView(playlist, next_list));
          },
        }
      },
      Action::PlayNextTrack => {
        if !playback_queue.is_empty() {
          let next_track = playback_queue.remove(0);
          session.play_track(&next_track);
          current_track = Some(PlaybackTrack::QueueTrack(next_track));
        } else {
          match current_track {
            Some(t) => {
              match t {
                PlaybackTrack::PlaylistTrack(playlist, _, track_index) => {
                  let next_track = playlist.track((track_index + 1) as i32);

                  // Pick the next track in the playlist and play that
                  match next_track {
                    Some(track) => {
                      session.play_track(&track);

                      current_track = Some(
                        PlaybackTrack::PlaylistTrack(
                          playlist.clone(),
                          track,
                          track_index + 1,
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
      Action::MoveUp => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => list.handle_up(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.handle_up(),
        }
      },
      Action::MoveDown => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => list.handle_down(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.handle_down(),
        }
      },
      Action::MoveTop => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => list.handle_top(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.handle_top(),
        }
      },
      Action::MoveBottom => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => list.handle_bottom(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.handle_bottom(),
        }
      },
      Action::QueueTrack => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => {
            let track = list.get_selected_item();
            playback_queue.push(track);
          },
          _ => (),
        }
      },
      Action::TogglePlayback => {
        let is_playing = session.is_playing();
        session.toggle_playback(!is_playing);
      },
      Action::FilterList(ref filter) => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => list.set_filter(filter),
          &mut NeubautenView::PlaylistView(ref mut list) => list.set_filter(filter),
        }
      },
      Action::Back => {
        match &mut current_view {
          &mut NeubautenView::TrackView(_, ref mut list) => list.clear_filter(),
          &mut NeubautenView::PlaylistView(ref mut list) => list.clear_filter(),
        }
      },
      Action::Quit => {
        break
      },
      _ => (),
    }

    if action != Action::Back || views.len() == 0 {
      views.push(current_view);
    }

    if next_view.is_some() {
      views.push(next_view.unwrap());
    }

    if action != action::Action::Noop {
      last_action = action;
    }
  }
}

fn print_view(view: &mut NeubautenView, rustbox: &rustbox::RustBox) {
  match view {
    &mut NeubautenView::PlaylistView(ref mut list) => {
      list.print(&rustbox, 0, 0, false);
    },
    &mut NeubautenView::TrackView(_, ref mut list) => {
      list.print(&rustbox, 0, 0, false);
    }
  }
}

fn print_status_bar(current_track: &Option<PlaybackTrack>,
                    playing: bool,
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
        &PlaybackTrack::PlaylistTrack(_, ref track, _) => {
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
  if current_track.is_some() && !playing {
    value.push_str("[Paused]");
  }

  for _ in 0..number_of_spaces {
    value.push(' ');
  }

  rustbox.print(0, y_pos, rustbox::RB_BOLD, Color::White, Color::Cyan, &value);
}

fn print_command_bar(action: &action::Action,
                     rustbox: &rustbox::RustBox) {
  let y_pos = rustbox.height() - 1;

  let value: String = match action {
    &action::Action::FilterList(ref string) => format!("Filter: {}", string),
    _ => "".to_string(),
  };

  rustbox.print(0, y_pos, rustbox::RB_NORMAL, Color::Default, Color::Default, &value);
}
