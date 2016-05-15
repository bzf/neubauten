extern crate rustbox;
extern crate rustify;

#[derive(Debug, PartialEq)]
pub enum NeubautenAction {
  Select,
  PlayNextTrack,
  QueueTrack,

  MoveDown,
  MoveUp,

  Back,
  Quit,

  Noop,
}

pub fn next_action(key_event: Option<rustbox::Event>,
                   rustify_event: Option<rustify::Event>) -> NeubautenAction {
  if rustify_event.is_some() {
    return next_rustify_event(rustify_event.unwrap());
  } else if key_event.is_some() {
    return next_keyboard_action(key_event.unwrap());
  } else {
    return NeubautenAction::Noop;
  }
}

fn next_rustify_event(event: rustify::Event) -> NeubautenAction {
  return match event {
    rustify::Event::EndOfTrack => NeubautenAction::PlayNextTrack,
    _ => NeubautenAction::Noop,
  }
}

fn next_keyboard_action(event: rustbox::Event) -> NeubautenAction {
  return match event {
    rustbox::Event::KeyEvent(key) => {
      match key {
        rustbox::Key::Char('j') => NeubautenAction::MoveDown,
        rustbox::Key::Char('k') => NeubautenAction::MoveUp,
        rustbox::Key::Char('>') => NeubautenAction::PlayNextTrack,
        rustbox::Key::Char('e') => NeubautenAction::Quit,
        rustbox::Key::Char('q') => NeubautenAction::QueueTrack,
        rustbox::Key::Enter => NeubautenAction::Select,
        rustbox::Key::Esc => NeubautenAction::Back,
        _ => NeubautenAction::Noop,
      }
    }
    _ => NeubautenAction::Noop,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rustbox;
  use rustify;

  #[test]
  fn it_parses_rustify_event_over_key_event() {
    let key = rustbox::Key::Char('j');
    let key_event = rustbox::Event::KeyEvent(key);
    let rustify_event = rustify::Event::EndOfTrack;

    let action = next_action(Some(key_event), Some(rustify_event));

    assert_eq!(action, NeubautenAction::PlayNextTrack);
  }

  #[test]
  fn it_returns_play_next_track_on_event_end_of_track() {
    let event = rustify::Event::EndOfTrack;

    let action = next_action(None, Some(event));

    assert_eq!(action, NeubautenAction::PlayNextTrack);
  }

  #[test]
  fn it_returns_none_when_nothing_matches() {
    let key = rustbox::Key::Char('z');
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::Noop);
  }

  #[test]
  fn it_returns_move_down_on_key_j() {
    let key = rustbox::Key::Char('j');
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::MoveDown);
  }

  #[test]
  fn it_returns_move_up_on_key_k() {
    let key = rustbox::Key::Char('k');
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::MoveUp);
  }

  #[test]
  fn it_returns_play_next_track_on_key_gt() {
    let key = rustbox::Key::Char('>');
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::PlayNextTrack);
  }

  #[test]
  fn it_returns_exit_on_key_e() {
    let key = rustbox::Key::Char('e');
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::Quit);
  }

  #[test]
  fn it_returns_back_on_key_esc() {
    let key = rustbox::Key::Esc;
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::Back);
  }

  #[test]
  fn it_returns_select_on_key_enter() {
    let key = rustbox::Key::Enter;
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::Select);
  }

  #[test]
  fn it_returns_queue_track_on_key_q() {
    let key = rustbox::Key::Char('q');;
    let event = rustbox::Event::KeyEvent(key);

    let action = next_action(Some(event), None);

    assert_eq!(action, NeubautenAction::QueueTrack);
  }
}
