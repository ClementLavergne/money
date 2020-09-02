//! # Money - WebAssembly API
//!
//! `money` is a collection of utilities to make tracking money expenses.

#[macro_use]
mod utils;

use chrono::NaiveDate;
use js_sys::Array;
use rust_money::order::{Order, TransactionState};
pub use rust_money::Account;
use std::convert::TryFrom;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Resets an existing account from **YAML** data.
/// Returns `true` if operation succeded, `false` otherwise.
#[wasm_bindgen]
pub fn load_account_data(account: &mut Account, data: &str) -> bool {
    match Account::try_from(data) {
        Ok(new) => {
            *account = new;
            true
        }
        Err(error) => {
            log!("{}", error);
            false
        }
    }
}

/// Returns tags as `JsValues`.
#[wasm_bindgen]
pub fn get_account_tags(account: &Account) -> Array {
    account
        .tags()
        .iter()
        .map(|(key, _)| JsValue::from(key))
        .collect()
}

/// Returns resources as `JsValues`.
#[wasm_bindgen]
pub fn get_account_resources(account: &Account) -> Array {
    account
        .resources()
        .iter()
        .map(|(key, _)| JsValue::from(key))
        .collect()
}

/// Exports all orders as `Array`.
#[wasm_bindgen]
pub fn get_account_orders(account: &Account) -> Array {
    account
        .orders()
        .iter()
        .enumerate()
        .map(|(id, order)| serialize_order_as_json(id, order))
        .collect()
}

/// Exports filtered orders as `Array`.
#[wasm_bindgen]
pub fn get_account_filtered_orders(account: &Account) -> Array {
    account
        .filtered_orders()
        .iter()
        .map(|(id, order)| serialize_order_as_json(*id, order))
        .collect()
}

/// Deletes a selected order.
#[wasm_bindgen]
pub fn delete_account_order(account: &mut Account, index: usize) -> bool {
    if let Some(order) = account.get_order_mut(index) {
        order.visible = false;
        account.delete_hidden_orders();
        true
    } else {
        false
    }
}

/// Sets date of a selected order.
#[wasm_bindgen]
pub fn set_account_order_date(account: &mut Account, index: usize, date: &str) -> bool {
    if let Some(order) = account.get_order_mut(index) {
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
#[wasm_bindgen]
pub fn set_account_order_description(
    account: &mut Account,
    index: usize,
    description: &str,
) -> bool {
    if let Some(order) = account.get_order_mut(index) {
        order.description = description.into();
        true
    } else {
        false
    }
}

/// Sets amount of a selected order.
#[wasm_bindgen]
pub fn set_account_order_amount(account: &mut Account, index: usize, amount: f32) -> bool {
    if let Some(order) = account.get_order_mut(index) {
        order.amount = amount;
        true
    } else {
        false
    }
}

/// Sets resource of a selected order.
#[wasm_bindgen]
pub fn set_account_order_resource(account: &mut Account, index: usize, resource: &str) -> bool {
    // Extract available strings.
    let available_resources = account.resources().keys().cloned().collect::<Vec<String>>();

    if let Some(order) = account.get_order_mut(index) {
        order.set_resource(resource, available_resources.as_slice());
        true
    } else {
        false
    }
}

/// Sets tags of a selected order.
#[wasm_bindgen]
pub fn set_account_order_tags(account: &mut Account, index: usize, tags: Array) -> bool {
    // Extract available strings.
    let available_tags = account.tags().keys().cloned().collect::<Vec<String>>();

    if let Some(order) = account.get_order_mut(index) {
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
#[wasm_bindgen]
pub fn set_account_order_state(
    account: &mut Account,
    index: usize,
    state: TransactionState,
) -> bool {
    if let Some(order) = account.get_order_mut(index) {
        order.set_state(state);
        true
    } else {
        false
    }
}

/// Converts account data into YAML string.
#[wasm_bindgen]
pub fn serialize_account_as_yaml(account: &Account) -> JsValue {
    JsValue::from(serde_yaml::to_string(account).unwrap())
}

/// Converts `Order` to string with its corresponding ID.
fn serialize_order_as_json(id: usize, order: &Order) -> JsValue {
    let json_order = serde_json::json!({"id": id, "order": order});

    JsValue::from(json_order.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_existing_order() {
        let mut account = Account::create();
        let mut expected_orders = [Order::default(), Order::default(), Order::default()];
        expected_orders
            .iter_mut()
            .enumerate()
            .for_each(|(id, order)| {
                order.description = format!("Order {}", id);
                account.add_order();
                account.get_order_mut(id).unwrap().description = order.description.clone();
            });

        assert_eq!(delete_account_order(&mut account, 1), true);
        assert_eq!(account.orders().len(), 2);
        assert_eq!(account.orders()[0].description, "Order 0".to_string());
        assert_eq!(account.orders()[1].description, "Order 2".to_string());
    }
}
