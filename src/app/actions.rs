use std::collections::HashMap;
use std::fmt::{
    self,
    Display
};
use std::slice::Iter;

use crate::inputs::key::Key;

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Quit,
    Tab,
    ShiftTab,
    Up,
    Down,
    Right,
    Left,
    SelectFolder,
    TakeUserInput,
    Escape,
    Enter,
    LogHelp,
    SaveSettings,
}

impl Action {
    /// All available actions
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 13] = [
            Action::Quit,
            Action::Tab,
            Action::ShiftTab,
            Action::Up,
            Action::Down,
            Action::Right,
            Action::Left,
            Action::SelectFolder,
            Action::TakeUserInput,
            Action::Escape,
            Action::Enter,
            Action::LogHelp,
            Action::SaveSettings
        ];
        ACTIONS.iter()
    }

    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::Tab => &[Key::Tab],
            Action::ShiftTab => &[Key::ShiftTab],
            Action::Up => &[Key::Up],
            Action::Down => &[Key::Down],
            Action::Right => &[Key::Right],
            Action::Left => &[Key::Left],
            Action::SelectFolder => &[Key::Char('f')],
            Action::TakeUserInput => &[Key::Char('i')],
            Action::Escape => &[Key::Esc],
            Action::Enter => &[Key::Enter],
            Action::LogHelp => &[Key::Char('h')],
            Action::SaveSettings => &[Key::Ctrl('s')]
        }
    }

    pub fn all() -> Vec<Action> {
        Action::iterator().cloned().collect()
    }
}

/// Could display a user friendly short description of action
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::Tab => "Focus next",
            Action::ShiftTab => "Focus previous",
            Action::Up => "Go up",
            Action::Down => "Go down",
            Action::Right => "Go right",
            Action::Left => "Go left",
            Action::SelectFolder => "Select folder",
            Action::TakeUserInput => "Enter input mode",
            Action::Escape => "Go to previous mode",
            Action::Enter => "Accept",
            Action::LogHelp => "Show help",
            Action::SaveSettings => "Save settings"
        };
        write!(f, "{}", str)
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        // Check key unicity
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {} with actions {}", key, actions)
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        // Ok, we can create contextual actions
        Self(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_action_by_key() {
        let actions: Actions = vec![Action::Quit, Action::Tab].into();
        let result = actions.find(Key::Ctrl('c'));
        assert_eq!(result, Some(&Action::Quit));
    }

    #[test]
    fn should_find_action_by_key_not_found() {
        let actions: Actions = vec![Action::Quit, Action::Tab].into();
        let result = actions.find(Key::Alt('w'));
        assert_eq!(result, None);
    }

    #[test]
    fn should_create_actions_from_vec() {
        let _actions: Actions = vec![
            Action::Quit,
            Action::Tab,
            Action::ShiftTab,
        ]
        .into();
    }

    #[test]
    #[should_panic]
    fn should_panic_when_create_actions_conflict_key() {
        let _actions: Actions = vec![
            Action::Quit,
            Action::Quit,
            Action::Tab,
            Action::Tab,
            Action::Tab,
        ]
        .into();
    }
}
