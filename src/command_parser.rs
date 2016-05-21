use action;
use rustbox;

#[derive(Debug, PartialEq)]
pub enum Action {
  NAction(action::Action),

  /// When parsing a sequence that partially matches an action, the
  /// `IncompleteSequence` variant says that there is a match but not a
  /// complete match.
  IncompleteSequence,

  /// When parsing a sequence that does not match to any actions the `NoMatch`
  /// variant will be used.
  NoMatch,
}

pub struct CommandParser {
  input_sequence: Vec<char>,
}

impl CommandParser {
  pub fn new() -> Self {
    return CommandParser {
      input_sequence: Vec::new(),
    };
  }

  pub fn handle_input(&mut self, key_event: &rustbox::Event) -> Action {
    match key_event {
      &rustbox::Event::KeyEvent(key) => {
        match key {
          rustbox::Key::Char(c) => {
            self.input_sequence.push(c);
          },
          rustbox::Key::Enter => {
            self.input_sequence.clear();
            return Action::NAction(action::Action::Select);
          },
          rustbox::Key::Esc => {
            if self.input_sequence.is_empty() {
              return Action::NAction(action::Action::Back);
            } else {
              self.input_sequence.clear();
            }
          },
          _ => (),
        }
      },
      _ => (),
    }

    return self.parse_input_sequence();
  }

  fn parse_input_sequence(&mut self) -> Action {
    let registered_commands: Vec<(String, action::Action)> = vec![
      ("gg".to_string(), action::Action::MoveTop),
      ("j".to_string(), action::Action::MoveDown),
      ("k".to_string(), action::Action::MoveUp),
      ("G".to_string(), action::Action::MoveBottom),
      ("q".to_string(), action::Action::QueueTrack),
      ("e".to_string(), action::Action::Quit),
      (" ".to_string(), action::Action::TogglePlayback),
      (">".to_string(), action::Action::PlayNextTrack),
    ];

    let mut found_partial_match = false;
    let mut exact_match: Option<action::Action> = None;
    let sequence: String = self.input_sequence.iter().cloned().collect();

    for (command, action) in registered_commands {
      if command.starts_with(&sequence) {
        found_partial_match = true;

        if command == sequence {
          exact_match = Some(action);
          break;
        }
      }
    }

    if exact_match.is_some() {
      self.input_sequence.clear();
      return Action::NAction(exact_match.unwrap());
    } else if found_partial_match {
      return Action::IncompleteSequence;
    } else {
      self.input_sequence.clear();
      return Action::NoMatch;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use action;
  use rustbox;

  #[test]
  fn it_returns_move_top_for_sequence_gg() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Char('g');
    let key_event = rustbox::Event::KeyEvent(key);

    parser.handle_input(&key_event);
    assert_eq!(parser.handle_input(&key_event), Action::NAction(action::Action::MoveTop));
  }

  #[test]
  fn it_returns_incomplete_sequence_for_sequence_g() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Char('g');
    let key_event = rustbox::Event::KeyEvent(key);

    assert_eq!(parser.handle_input(&key_event), Action::IncompleteSequence);
  }

  #[test]
  fn it_returns_move_to_bottom_for_sequence_g() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Char('G');
    let key_event = rustbox::Event::KeyEvent(key);

    assert_eq!(parser.handle_input(&key_event), Action::NAction(action::Action::MoveBottom));
  }

  #[test]
  fn it_returns_toggle_playback_on_space() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Char(' ');
    let key_event = rustbox::Event::KeyEvent(key);

    assert_eq!(parser.handle_input(&key_event), Action::NAction(action::Action::TogglePlayback));
  }

  #[test]
  fn it_returns_no_match_for_non_matching_sequences() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Char('z');
    let key_event = rustbox::Event::KeyEvent(key);

    assert_eq!(parser.handle_input(&key_event), Action::NoMatch);
  }

  #[test]
  fn it_returns_back_on_esc_with_empty_input_sequence() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Esc;
    let key_event = rustbox::Event::KeyEvent(key);

    assert_eq!(parser.handle_input(&key_event), Action::NAction(action::Action::Back));
  }

  #[test]
  fn it_returns_select_on_enter() {
    let mut parser = CommandParser::new();
    let key = rustbox::Key::Enter;
    let key_event = rustbox::Event::KeyEvent(key);

    assert_eq!(parser.handle_input(&key_event), Action::NAction(action::Action::Select));
  }
}
