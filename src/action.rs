extern crate rustbox;
extern crate rustify;

#[derive(Debug, PartialEq)]
pub enum Action {
  Select,
  PlayNextTrack,
  QueueTrack,
  TogglePlayback,

  MoveBottom,
  MoveDown,
  MoveTop,
  MoveUp,

  Back,
  Quit,

  Noop,
}

pub fn next_action(rustify_event: Option<rustify::Event>) -> Action {
  if rustify_event.is_some() {
    return next_rustify_event(rustify_event.unwrap());
  } else {
    return Action::Noop;
  }
}

fn next_rustify_event(event: rustify::Event) -> Action {
  return match event {
    rustify::Event::EndOfTrack => Action::PlayNextTrack,
    _ => Action::Noop,
  }
}
