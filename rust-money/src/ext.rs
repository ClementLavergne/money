//! # Extension for `Vec<String>` type.

#[cfg(feature = "wasmbind")]
use js_sys::Array;
use serde::{Deserialize, Serialize};
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

/// Extension for `Vec<Category>` to manage unique keys.
pub trait ExclusiveItemExt {
    fn add_exclusive(&mut self, key: &str) -> Option<RequestFailure>;
    fn remove_exclusive(&mut self, key: &str) -> Option<RequestFailure>;
    fn toggle_selection(&mut self, key: &str) -> Option<RequestFailure>;
    fn keys(&self) -> Vec<String>;
    #[cfg(feature = "wasmbind")]
    fn js_keys(&self) -> Array;
}

/// Stores current state of a given filter parameter.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ItemSelector {
    /// Filter out.
    Discarded,
    /// Filter in.
    Selected,
}

/// Key-value tuple struct which manages either *tag* or *resource*.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Category(pub String, pub ItemSelector);

impl ItemSelector {
    /// Toggles the state.
    pub fn toggle(&mut self) {
        *self = match *self {
            ItemSelector::Discarded => ItemSelector::Selected,
            ItemSelector::Selected => ItemSelector::Discarded,
        };
    }
}

impl ExclusiveItemExt for Vec<Category> {
    /// Adds a new item if not exists yet.
    fn add_exclusive(&mut self, key: &str) -> Option<RequestFailure> {
        if !key.is_empty() {
            if !key.chars().all(char::is_whitespace) {
                if !self.iter().any(|item| item.0 == key) {
                    self.push(Category(key.into(), ItemSelector::Selected));
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
        if let Some(index) = self.iter().position(|item| item.0 == key) {
            self.remove(index);
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    // Toggles selection of a key
    fn toggle_selection(&mut self, key: &str) -> Option<RequestFailure> {
        if let Some(index) = self.iter().position(|item| item.0 == key) {
            self.get_mut(index).unwrap().1.toggle();
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    // Export filter keys.
    fn keys(&self) -> Vec<String> {
        self.iter().map(|item| item.0.clone()).collect()
    }

    // Export filter keys.
    #[cfg(feature = "wasmbind")]
    fn js_keys(&self) -> Array {
        self.iter().map(|item| JsValue::from(&item.0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_valid_key() {
        let items = (0..3)
            .map(|id| Category(format!("Key {}", id), ItemSelector::Selected))
            .collect::<Vec<Category>>();

        let mut list: Vec<Category> = items.as_slice()[..2].to_vec();

        assert_eq!(list.add_exclusive(items[2].0.as_str()), None);
        assert_eq!(list, items);
    }

    #[test]
    fn remove_known_key() {
        let items = (0..3)
            .map(|id| Category(format!("Key {}", id), ItemSelector::Selected))
            .collect::<Vec<Category>>();

        let mut list: Vec<Category> = items.clone();

        assert_eq!(list.remove_exclusive(items[2].0.as_str()), None);
        assert_eq!(list, items.as_slice()[..2].to_vec());
    }

    #[test]
    fn discard_adding_existing_key() {
        let items = (0..3)
            .map(|id| Category(format!("Key {}", id), ItemSelector::Selected))
            .collect::<Vec<Category>>();

        let mut list: Vec<Category> = items.clone();

        assert_eq!(
            list.add_exclusive(items[2].0.as_str()),
            Some(RequestFailure::ExistingItem)
        );
        assert_eq!(list, items);
    }

    #[test]
    fn discard_adding_empty_key() {
        let mut list: Vec<Category> = Vec::new();

        assert_eq!(list.add_exclusive(""), Some(RequestFailure::EmptyArgument));
        assert_eq!(list, []);
    }

    #[test]
    fn discard_adding_incorrect_key() {
        let mut list: Vec<Category> = Vec::new();

        assert_eq!(
            list.add_exclusive("  "),
            Some(RequestFailure::IncorrectArgument)
        );
        assert_eq!(list, []);
    }

    #[test]
    fn discard_removing_unknown_key() {
        let items = (0..3)
            .map(|id| Category(format!("Key {}", id), ItemSelector::Selected))
            .collect::<Vec<Category>>();

        let mut list: Vec<Category> = items.clone();

        assert_eq!(
            list.remove_exclusive("Key 4"),
            Some(RequestFailure::UnknownItem)
        );
        assert_eq!(list, items);
    }
}
