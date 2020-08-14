//! # Money
//!
//! `money` is a collection of utilities to make tracking money expenses.

mod utils;

use wasm_bindgen::prelude::*;

extern crate js_sys;
use js_sys::Array;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Manages account data.
#[wasm_bindgen]
pub struct Account {
    available_tags: Vec<String>,
}

#[wasm_bindgen]
impl Account {
    /// Instantiates a new object.
    #[wasm_bindgen(constructor)]
    pub fn create() -> Account {
        utils::set_panic_hook();

        let empty_tags = Vec::new();

        Account {
            available_tags: empty_tags,
        }
    }

    /// Adds a new tag if not exists yet.
    pub fn add_tag(&mut self, tag: &str) {
        if !tag.is_empty() {
            if self.available_tags.iter().any(|i| i == tag) {
                println!("tag '{}' already exists!", tag)
            } else {
                self.available_tags.push(tag.to_string())
            }
        }
    }

    /// Removes an existing tag.
    pub fn remove_tag(&mut self, tag: &str) {
        let index = self.available_tags.iter().position(|x| x == tag);
        if let Some(i) = index {
            self.available_tags.remove(i);
        }
    }

    /// Extracts available tags as `JsValue`.
    pub fn get_tags(&self) -> Array {
        self.available_tags.iter().map(JsValue::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_empty_tag() {
        let mut data = Account::create();

        assert_eq!(data.available_tags.len(), 0);
        data.add_tag("");
        assert_eq!(data.available_tags.len(), 0);
    }

    #[test]
    fn avoid_tag_redundancy() {
        let mut data = Account::create();

        assert_eq!(data.available_tags.len(), 0);
        data.add_tag("Food");
        assert_eq!(vec![String::from("Food")], data.available_tags);
        data.add_tag("Transport");
        assert_eq!(
            vec![String::from("Food"), String::from("Transport")],
            data.available_tags
        );
        data.add_tag("Food");
        data.add_tag("Service");
        assert_eq!(
            vec![
                String::from("Food"),
                String::from("Transport"),
                String::from("Service")
            ],
            data.available_tags
        );
    }

    #[test]
    fn safe_tag_remove() {
        let mut data = Account::create();

        data.add_tag("Food");
        data.add_tag("Transport");
        data.add_tag("Service");
        assert_eq!(
            vec![
                String::from("Food"),
                String::from("Transport"),
                String::from("Service")
            ],
            data.available_tags
        );
        data.remove_tag("Food");
        assert_eq!(
            vec![String::from("Transport"), String::from("Service")],
            data.available_tags
        );
        data.remove_tag("Hangout");
        assert_eq!(
            vec![String::from("Transport"), String::from("Service")],
            data.available_tags
        );
    }
}
