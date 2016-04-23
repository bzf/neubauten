extern crate rustbox;
extern crate core;

use std;
use std::fmt::{Display};
use rustbox::{Color};

pub struct ListComponent<T: Display> {
  /// Container for all of the items that should be displayed.
  items: Vec<T>,

  cursor_index: usize,
  width: usize,
  height: usize,

  /// Contains indexes of the `items` that matches the `current_filter`
  matching_indexes: Vec<usize>,

  /// This value is used for knowing from which item in the `items` vector. If
  /// we have more items than available `rows` we need to have some scrolling
  /// mechanic.
  print_from_index: usize,
}

impl<T: Display> ListComponent<T> {
  pub fn new(items: Vec<T>, height: usize, width: usize) -> Self {
    let mut list = ListComponent {
      items: items,
      cursor_index: 0,
      width: width,
      height: height,
      matching_indexes: Vec::new(),
      print_from_index: 0,
    };

    list.update_matches(&None);

    return list;
  }

  pub fn update_matches(&mut self, filter: &Option<String>) {
    self.matching_indexes.clear();

    for (index, item) in self.items.iter().enumerate() {
      if self.does_item_match_filter(&item, filter) {
        self.matching_indexes.push(index);
      }
    }
  }

  pub fn print(&mut self, rustbox: &rustbox::RustBox, x_pos: usize, y_pos: usize, filter: &Option<String>, reset_cursor: bool) {
    if self.items.is_empty() {
      return;
    }

    self.update_matches(filter);

    // TODO: Reset the `cursor_index` if we have a (new) filter
    if reset_cursor {
      self.cursor_index = 0;
    }

    let mut items: Vec<&T> = Vec::new();
    for index in self.matching_indexes.iter() {
      let item: &T = self.items.get(index.clone()).unwrap();
      items.push(item);
    }

    let mut index = 0;

    let max_index = std::cmp::min(self.height, items.len());
    let items_to_display = &items[(self.print_from_index)..(max_index + self.print_from_index)];

    for item in items_to_display {
      let number_of_spaces = self.width - item.to_string().len() - 1;
      let mut value: String = format!(" {}", item);

      for _ in 0..number_of_spaces {
        value.push(' ');
      }

      if (index + self.print_from_index) == self.cursor_index {
        rustbox.print(x_pos, y_pos + index, rustbox::RB_BOLD, Color::White, Color::Black, &value);
      } else {
        rustbox.print(x_pos, y_pos + index, rustbox::RB_NORMAL, Color::Default, Color::Default, &value);
      }

      index += 1;
    }
  }

  pub fn handle_down(&mut self) {
    if !self.is_cursor_at_last_item() {
      self.cursor_index += 1;

      if self.is_cursor_out_of_screen() {
        self.print_from_index += 1;
      }
    }
  }

  pub fn handle_up(&mut self) {
    if self.cursor_index > 0 {
      self.cursor_index -= 1;
    }

    if self.is_cursor_out_of_screen() {
      self.print_from_index -= 1;
    }
  }

  pub fn get_selected_index(&self) -> usize {
    return self.matching_indexes[self.cursor_index];
  }

  fn is_cursor_out_of_screen(&self) -> bool {
    let screen_starts_at = self.print_from_index;
    let screen_ends_at = screen_starts_at + self.height - 1;

    return (self.cursor_index < screen_starts_at) || (self.cursor_index > screen_ends_at);
  }

  fn is_cursor_at_last_item(&self) -> bool {
    return self.cursor_index == (self.matching_indexes.len() - 1);
  }

  fn does_item_match_filter(&self, item: &T, filter: &Option<String>) -> bool {
    return match filter {
      &Some(ref filter) => self.does_string_match_filter(&item.to_string(), filter),

      // If there's no filter then we want to show them all
      &None => true,
    }
  }

  fn does_string_match_filter(&self, value: &str, filter: &str) -> bool {
    // If the filter is empty that means that we've successfully matched the
    // full filter
    if filter.is_empty() {
      return true;
    }

    let (first_char, next_filter) = filter.split_at(1);

    return match value.find(first_char) {
      Some(index) => {
        let (_, next_value) = value.split_at(index);
        self.does_string_match_filter(next_value, next_filter)
      },
      None => false,
    }
  }
}

#[test]
fn defaults_the_cursor_to_zero_index() {
  let items = vec![ "foo", "bar", "baz" ];
  let list = ListComponent::new(items, 1, 10);

  let index = list.get_selected_index();

  assert_eq!(index, 0);
}

#[test]
fn it_can_move_the_cursor_down() {
  let items = vec![ "foo", "bar", "baz" ];

  let mut list = ListComponent::new(items, 1, 10);
  list.handle_down();

  assert_eq!(list.get_selected_index(), 1);
}

#[test]
fn it_can_move_the_cursor_up() {
  let items = vec![ "foo", "bar", "baz" ];

  let mut list = ListComponent::new(items, 1, 10);
  list.handle_down();
  list.handle_up();

  assert_eq!(list.get_selected_index(), 0);
}

#[test]
fn it_cant_move_above_the_first_item() {
  let items = vec![ "foo" ];
  let mut list = ListComponent::new(items, 1, 10);

  list.handle_up();

  assert_eq!(list.get_selected_index(), 0);
}

#[test]
fn it_cant_move_below_the_last_item() {
  let items = vec![ "foo", "bar" ];
  let mut list = ListComponent::new(items, 1, 10);

  list.handle_down();
  list.handle_down();

  assert_eq!(list.get_selected_index(), 1);
}
