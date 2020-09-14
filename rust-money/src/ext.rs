//! # Extension for `Vec<String>` type.

#[cfg(feature = "wasmbind")]
use js_sys::Array;
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;

/// Defines error types.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(PartialEq, Debug)]
pub enum RequestFailure {
    /// User input is incorrect.
    IncorrectArgument,
    /// User input is empty.
    EmptyArgument,
    /// Specified item can not be removed as it does not exist.
    UnknownItem,
    /// Specified item can not be added as it already did.
    ExistingItem,
}

/// Extension for `Vec<String>` to manage unique keys.
pub trait ExclusiveItemExt {
    fn add_exclusive(&mut self, key: &str) -> Option<RequestFailure>;
    fn remove_exclusive(&mut self, key: &str) -> Option<RequestFailure>;
    #[cfg(feature = "wasmbind")]
    fn sorted_keys(&self) -> Array;
}

impl ExclusiveItemExt for Vec<String> {
    /// Adds a new item if not exists yet.
    fn add_exclusive(&mut self, key: &str) -> Option<RequestFailure> {
        if !key.is_empty() {
            if !key.chars().all(char::is_whitespace) {
                if !self.iter().any(|item| item == key) {
                    self.push(key.into());
                    None
                } else {
                    Some(RequestFailure::ExistingItem)
                }
            } else {
                Some(RequestFailure::IncorrectArgument)
            }
        } else {
            Some(RequestFailure::EmptyArgument)
        }
    }

    /// Removes an existing item.
    fn remove_exclusive(&mut self, key: &str) -> Option<RequestFailure> {
        if let Some(index) = self.iter().position(|item| item == key) {
            self.remove(index);
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    // Export sorted filter keys.
    #[cfg(feature = "wasmbind")]
    fn sorted_keys(&self) -> Array {
        let mut sorted_vec = self.clone();
        sorted_vec.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        sorted_vec.iter().map(JsValue::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_valid_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list: Vec<String> = items.as_slice()[..2].to_vec();

        assert_eq!(list.add_exclusive(items[2].as_str()), None);
        assert_eq!(list, items);
    }

    #[test]
    fn remove_known_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list = items.clone();

        assert_eq!(list.remove_exclusive(items[2].as_str()), None);
        assert_eq!(list, items.as_slice()[..2].to_vec());
    }

    #[test]
    fn discard_adding_existing_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list = items.clone();

        assert_eq!(
            list.add_exclusive(items[2].as_str()),
            Some(RequestFailure::ExistingItem)
        );
        assert_eq!(list, items);
    }

    #[test]
    fn discard_adding_empty_key() {
        let mut list: Vec<String> = Vec::new();

        assert_eq!(list.add_exclusive(""), Some(RequestFailure::EmptyArgument));
        assert_eq!(list.is_empty(), true);
    }

    #[test]
    fn discard_adding_incorrect_key() {
        let mut list: Vec<String> = Vec::new();

        assert_eq!(
            list.add_exclusive("  "),
            Some(RequestFailure::IncorrectArgument)
        );
        assert_eq!(list.is_empty(), true);
    }

    #[test]
    fn discard_removing_unknown_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list = items.clone();

        assert_eq!(
            list.remove_exclusive("Key 4"),
            Some(RequestFailure::UnknownItem)
        );
        assert_eq!(list, items);
    }
}
