//! # Money
//!
//! `money` is a collection of utilities to make tracking money expenses.

mod ext;
mod utils;

use ext::ExclusiveItemExt;
use js_sys::Array;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Manages account data.
#[wasm_bindgen]
pub struct Account {
    tags: Vec<String>,
    resources: Vec<String>,
}

#[wasm_bindgen]
impl Account {
    /// Instantiates a new object.
    #[wasm_bindgen(constructor)]
    pub fn create() -> Account {
        utils::set_panic_hook();

        Account {
            tags: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Add a new tag.
    pub fn add_tag(&mut self, tag: &str) {
        self.tags.add_exclusive(tag);
    }

    /// Remove a tag.
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.remove_exclusive(tag);
    }

    /// Get tags as `JsValues`.
    pub fn export_tags(&self) -> Array {
        self.tags.iter().map(JsValue::from).collect()
    }

    /// Add a new resource.
    pub fn add_resource(&mut self, tag: &str) {
        self.resources.add_exclusive(tag);
    }

    /// Remove a resource.
    pub fn remove_resource(&mut self, tag: &str) {
        self.resources.remove_exclusive(tag);
    }

    /// Get resources as `JsValues`.
    pub fn export_resources(&self) -> Array {
        self.resources.iter().map(JsValue::from).collect()
    }
}
