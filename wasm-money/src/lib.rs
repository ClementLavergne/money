//! # Money - WebAssembly API
//!
//! `money` is a collection of utilities to make tracking money expenses.

mod utils;

use js_sys::Array;
use rust_money::Account;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Manages account data.
#[wasm_bindgen]
pub struct AccountClient {
    account: Account,
}

#[wasm_bindgen]
impl AccountClient {
    /// Instantiates a new object.
    #[wasm_bindgen(constructor)]
    pub fn create() -> AccountClient {
        utils::set_panic_hook();

        AccountClient {
            account: Account::create(),
        }
    }

    /// Add a new tag.
    pub fn add_tag(&mut self, tag: &str) {
        self.account.add_tag(tag);
    }

    /// Remove a tag.
    pub fn remove_tag(&mut self, tag: &str) {
        self.account.remove_tag(tag);
    }

    /// Get tags as `JsValues`.
    pub fn tags(&self) -> Array {
        self.account.tags().iter().map(JsValue::from).collect()
    }

    /// Add a new resource.
    pub fn add_resource(&mut self, resource: &str) {
        self.account.add_resource(resource);
    }

    /// Remove a resource.
    pub fn remove_resource(&mut self, resource: &str) {
        self.account.remove_resource(resource);
    }

    /// Get resources as `JsValues`.
    pub fn resources(&self) -> Array {
        self.account.resources().iter().map(JsValue::from).collect()
    }
}
