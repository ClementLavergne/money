//! # Money - WebAssembly API
//!
//! `money` is a collection of utilities to make tracking money expenses.

#[macro_use]
mod utils;

use chrono::NaiveDate;
use js_sys::Array;
use rust_money::ext::ExclusiveItemExt;
use rust_money::filter::category::{Category, CategoryFilter};
use rust_money::filter::{Filter, ItemSelector};
use rust_money::order::{Order, TransactionState};
pub use rust_money::Account;
use std::convert::TryFrom;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use CategoryType::Resource;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Defines available category types.
#[wasm_bindgen]
#[derive(PartialEq, Debug)]
pub enum CategoryType {
    /// A **resource** identifies something which represents/holds money.
    Resource,
    /// A **tag** identifies a category of expense.
    /// Could be: an object, a person, a firm, .. it's up to you!
    Tag,
}

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

/// Returns all categories of a given type as `JsValues`.
#[wasm_bindgen]
pub fn get_account_categories(account: &Account, category_type: CategoryType) -> Array {
    if let Resource = category_type {
        account.resources().sorted_keys()
    } else {
        account.tags().sorted_keys()
    }
}

/// Exports filtered orders as `Array`.
#[wasm_bindgen]
pub fn get_account_filtered_orders(account: &Account, filter: &Filter) -> Array {
    account
        .filtered_orders(filter)
        .iter()
        .map(|(id, order)| serialize_order_as_json(*id, order))
        .collect()
}

/// Deletes a selected order.
#[wasm_bindgen]
pub fn toggle_account_order_visibility(account: &mut Account, index: usize) -> bool {
    if let Some(order) = account.get_order_mut(index) {
        order.visible = !order.visible;
        true
    } else {
        false
    }
}

/// Deletes a selected order.
#[wasm_bindgen]
pub fn delete_account_order(account: &mut Account, index: usize) -> bool {
    account.delete_order(index)
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
    let available_resources = account.resources().clone();

    if let Some(order) = account.get_order_mut(index) {
        order.set_resource(resource, available_resources.as_slice());
        true
    } else {
        false
    }
}

/// Sets tags of a selected order.
///
/// # Return
/// * `false` if at least one tag does not match with available ones, but correct tags are still added.
/// * `true` otherwise.
#[wasm_bindgen]
pub fn set_account_order_tags(account: &mut Account, index: usize, tags: Array) -> bool {
    // Extract available strings.
    let available_tags = account.tags().clone();

    if let Some(order) = account.get_order_mut(index) {
        // Clear all tags
        order.clear_tags();
        // Add each tag and make sure that no error happens
        let available_tags = available_tags.as_slice();
        !tags.iter().fold(false, |acc, value| {
            acc | if let Some(tag) = value.as_string() {
                !order.add_tag(tag.as_str(), available_tags)
            } else {
                false
            }
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

/// Disables filtering of all categories of a given type.
#[wasm_bindgen]
pub fn clear_filter_categories(filter: &mut Filter, category_type: CategoryType) {
    if let Resource = category_type {
        *filter.get_resource_option_mut() = CategoryFilter::CategoryIgnored;
    } else {
        *filter.get_tag_option_mut() = CategoryFilter::CategoryIgnored;
    }
}

/// Initializes each incoming category to `Selected`.
#[wasm_bindgen]
pub fn set_filter_categories(filter: &mut Filter, category_type: CategoryType, names: &Array) {
    let filter_option: &mut CategoryFilter;

    if let Resource = category_type {
        filter_option = filter.get_resource_option_mut();
    } else {
        filter_option = filter.get_tag_option_mut();
    }

    filter_option.set(
        names
            .iter()
            .filter_map(|category_name| {
                if let Some(category_string) = category_name.as_string() {
                    Some(Category(category_string, ItemSelector::Selected))
                } else {
                    None
                }
            })
            .collect::<Vec<Category>>()
            .into_iter(),
    );
}

/// Adds a new category to filter options.
#[wasm_bindgen]
pub fn add_filter_category(filter: &mut Filter, category_type: CategoryType, name: &str) {
    if let Resource = category_type {
        filter
            .get_resource_option_mut()
            .add(Category(name.into(), ItemSelector::Selected));
    } else {
        filter
            .get_tag_option_mut()
            .add(Category(name.into(), ItemSelector::Selected));
    }
}

/// Removes a category from filtering options.
#[wasm_bindgen]
pub fn remove_filter_category(
    filter: &mut Filter,
    category_type: CategoryType,
    name: &str,
) -> bool {
    if let Resource = category_type {
        filter.get_resource_option_mut().remove(name)
    } else {
        filter.get_tag_option_mut().remove(name)
    }
}

/// Returns the filtering option for a given *tag*, if available.
#[wasm_bindgen]
pub fn get_filter_category_state(
    filter: &mut Filter,
    category_type: CategoryType,
    name: &str,
) -> Option<ItemSelector> {
    let filter_option: &CategoryFilter;

    if let Resource = category_type {
        filter_option = filter.resource_option();
    } else {
        filter_option = filter.tag_option();
    }

    if let CategoryFilter::Enabled(items) = filter_option {
        if let Some(index) = items.iter().position(|item| item.0 == name) {
            Some(items[index].1)
        } else {
            None
        }
    } else {
        None
    }
}

/// Toggles selected category filtering.
#[wasm_bindgen]
pub fn toggle_filter_category(
    filter: &mut Filter,
    category_type: CategoryType,
    name: &str,
) -> Option<ItemSelector> {
    let filter_option: &mut CategoryFilter;

    if let Resource = category_type {
        filter_option = filter.get_resource_option_mut();
    } else {
        filter_option = filter.get_tag_option_mut();
    }

    if let Some(selector) = filter_option.toggle(name) {
        Some(*selector)
    } else {
        None
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

/// Sums each displayed order amount.
#[wasm_bindgen]
pub fn sum_filtered_orders(account: &Account, filter: &Filter) -> f32 {
    account
        .filtered_orders(filter)
        .iter()
        .map(|item| item.1.amount)
        .sum()
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
