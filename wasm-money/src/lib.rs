//! # Money - WebAssembly API
//!
//! `money` is a collection of utilities to make tracking money expenses.

#[macro_use]
mod utils;

use chrono::NaiveDate;
use js_sys::Array;
use rust_money::order::{Order, TransactionState};
use rust_money::Account;
use std::convert::TryFrom;
use std::str::FromStr;
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

    /// Loads data from file.
    pub fn load(&mut self, content: &str) -> bool {
        match Account::try_from(content) {
            Ok(new) => {
                self.account = new;
                true
            }
            Err(error) => {
                log!("{}", error);
                false
            }
        }
    }

    /// Adds a new tag.
    pub fn add_tag(&mut self, tag: &str) -> bool {
        self.account.add_tag(tag).is_none()
    }

    /// Removes a tag.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        self.account.remove_tag(tag).is_none()
    }

    // Filters in or out the selected tag.
    pub fn toggle_tag(&mut self, tag: &str) {
        self.account.toggle_tag(tag);
    }

    /// Gets tags as `JsValues`.
    pub fn tags(&self) -> Array {
        self.account
            .tags()
            .iter()
            .map(|(key, _)| JsValue::from(key))
            .collect()
    }

    /// Adds a new resource.
    pub fn add_resource(&mut self, resource: &str) -> bool {
        self.account.add_resource(resource).is_none()
    }

    /// Remove a resource.
    pub fn remove_resource(&mut self, resource: &str) -> bool {
        self.account.remove_resource(resource).is_none()
    }

    // Filters in or out the selected resource.
    pub fn toggle_resource(&mut self, resource: &str) {
        self.account.toggle_resource(resource);
    }

    /// Gets resources as `JsValues`.
    pub fn resources(&self) -> Array {
        self.account
            .resources()
            .iter()
            .map(|(key, _)| JsValue::from(key))
            .collect()
    }

    /// Creates a default order.
    pub fn add_order(&mut self) {
        self.account.add_order();
    }

    /// Deletes a selected order.
    pub fn delete_order(&mut self, index: usize) -> bool {
        if let Some(order) = self.account.get_order_mut(index) {
            order.visible = false;
            self.account.delete_hidden_orders();
            true
        } else {
            false
        }
    }

    /// Sets date of a selected order.
    pub fn set_order_date(&mut self, index: usize, date: &str) -> bool {
        if let Some(order) = self.account.get_order_mut(index) {
            if date.is_empty() {
                order.date = None;
                true
            } else {
                match NaiveDate::from_str(date) {
                    Ok(result) => {
                        order.date = Some(result);
                        true
                    }
                    Err(_) => false,
                }
            }
        } else {
            false
        }
    }

    /// Sets description of a selected order.
    pub fn set_order_description(&mut self, index: usize, description: &str) -> bool {
        if let Some(order) = self.account.get_order_mut(index) {
            order.description = description.into();
            true
        } else {
            false
        }
    }

    /// Sets amount of a selected order.
    pub fn set_order_amount(&mut self, index: usize, amount: f32) -> bool {
        if let Some(order) = self.account.get_order_mut(index) {
            order.amount = amount;
            true
        } else {
            false
        }
    }

    /// Sets resource of a selected order.
    pub fn set_order_resource(&mut self, index: usize, resource: &str) -> bool {
        // Extract available strings.
        let available_resources = self
            .account
            .resources()
            .keys()
            .cloned()
            .collect::<Vec<String>>();

        if let Some(order) = self.account.get_order_mut(index) {
            order.set_resource(resource, available_resources.as_slice());
            true
        } else {
            false
        }
    }

    /// Sets tags of a selected order.
    pub fn set_order_tags(&mut self, index: usize, tags: Array) -> bool {
        // Extract available strings.
        let available_tags = self.account.tags().keys().cloned().collect::<Vec<String>>();

        if let Some(order) = self.account.get_order_mut(index) {
            // Clear all tags
            order.clear_tags();
            // Add each tag and make sure that no error happens
            let available_tags = available_tags.as_slice();
            !tags.iter().fold(false, |acc, value| {
                acc | order.add_tag(value.as_string().unwrap().as_str(), available_tags)
            })
        } else {
            false
        }
    }

    /// Sets state of a selected order.
    pub fn set_order_state(&mut self, index: usize, state: &str) -> bool {
        if let Some(order) = self.account.get_order_mut(index) {
            let order_state = match state {
                "Pending" => TransactionState::Pending,
                "InProgress" => TransactionState::InProgress,
                _ => TransactionState::Done,
            };

            order.set_state(order_state);
            true
        } else {
            false
        }
    }

    /// Exports all orders as `Array`.
    pub fn orders(&self) -> Array {
        self.account
            .orders()
            .iter()
            .enumerate()
            .map(|(id, order)| serialize_order_json(id, order))
            .collect()
    }

    /// Exports filtered orders as `Array`.
    pub fn filtered_orders(&self) -> Array {
        self.account
            .filtered_orders()
            .iter()
            .map(|(id, order)| serialize_order_json(*id, order))
            .collect()
    }

    /// Sums each order amount.
    pub fn sum_orders(&self) -> f32 {
        self.account.orders().iter().map(|order| order.amount).sum()
    }

    /// Converts account data into YAML string.
    pub fn serialize_account_yaml(&self) -> JsValue {
        JsValue::from(serde_yaml::to_string(&self.account).unwrap())
    }
}

/// Converts `Order` to string with its corresponding ID.
fn serialize_order_json(id: usize, order: &Order) -> JsValue {
    let json_order = serde_json::json!({"id": id, "order": order});

    JsValue::from(json_order.to_string())
}
