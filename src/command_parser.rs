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

  /// This variable keeps track if we need to read input until we hit either:
  /// An `Esc` event which would clear the `input_sequence` and return `NoMatch` or
  /// A `Enter` event which would return the parsed event and clear the input sequence
  argument: Vec<char>,
  argument_type: ArgumentType,
}

#[derive(Clone, PartialEq)]
pub enum ArgumentType {
  FilterArgument,
  SearchArgument,
  None,
}

impl CommandParser {
  pub fn new() -> Self {
    return CommandParser {
      input_sequence: Vec::new(),
      argument: Vec::new(),
      argument_type: ArgumentType::None,
    };
  }

  /// Takes an `rustbox::Event` and updates the internal `argument` value.
  ///
  /// Return wheter or not the event given will end the sequence.
  fn update_argument(&mut self, key_event: &rustbox::Event) -> bool {
    return match key_event {
      &rustbox::Event::KeyEvent(key) => {
        match key {
          rustbox::Key::Enter => {
            self.argument_type = ArgumentType::None;
            self.argument.clear();
            return true;
          },
          rustbox::Key::Esc => {
            self.argument_type = ArgumentType::None;
            self.input_sequence.clear();
            return true;
          },
          rustbox::Key::Backspace => {
            if self.argument.len() > 0 {
              self.argument.pop();
              return false;
            } else {
              self.argument_type = ArgumentType::None;
              self.argument.clear();
              return false;
            }
          },
          rustbox::Key::Char(c) => {
            self.argument.push(c);
            return false;
          },
          _ => false
        }
      },
      _ => false,
    }
  }

  pub fn get_argument_type(&self) -> ArgumentType {
    return self.argument_type.clone();
  }

  pub fn get_argument(&self) -> String {
    return self.argument.iter().cloned().collect();
  }

  pub fn handle_input(&mut self, key_event: &rustbox::Event) -> Action {
    if self.argument_type != ArgumentType::None {
      let argument: String = self.argument.iter().cloned().collect();
      let argument_type = self.argument_type.clone();

      if self.update_argument(key_event) {
        return match argument_type {
          ArgumentType::FilterArgument => {
            Action::NAction(action::Action::FilterList(argument))
          },
          ArgumentType::SearchArgument => {
            Action::NAction(action::Action::SearchTrack(argument))
          },
          _ => Action::NoMatch,
        };
      } else {
        return Action::NoMatch;
      }
    } else {
      match key_event {
        &rustbox::Event::KeyEvent(key) => {
          match key {
            rustbox::Key::Char('/') => {
              self.argument_type = ArgumentType::FilterArgument;
              return Action::NoMatch;
            },
            rustbox::Key::Char('s') => {
              self.argument_type = ArgumentType::SearchArgument;
              return Action::NoMatch;
            },
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
