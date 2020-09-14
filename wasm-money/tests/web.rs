//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use js_sys::Array;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen_test::*;
use wasm_money::CategoryType::{Resource, Tag};
use wasm_money::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn check_sorted_tags() {
    let mut account = Account::create();
    let tags = ["Food", "Transport", "Service", "Video Games"];
    let mut sorted_tags = tags.to_vec();
    sorted_tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let expected = sorted_tags
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    tags.iter().for_each(|tag| {
        account.add_tag(tag);
    });

    assert_eq!(
        get_account_categories(&account, Tag).to_vec(),
        expected.to_vec()
    );

    account.remove_tag(tags[1]);
    account.remove_tag(tags[3]);

    let mut sorted_tags = [tags[0], tags[2]].to_vec();
    sorted_tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let expected = sorted_tags
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    assert_eq!(
        get_account_categories(&account, Tag).to_vec(),
        expected.to_vec()
    );
}

#[wasm_bindgen_test]
pub fn check_sorted_resources() {
    let mut account = Account::create();
    let resources = ["Cash", "Bank"];
    let mut sorted_resources = resources.to_vec();
    sorted_resources.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let expected = sorted_resources
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    resources.iter().for_each(|resource| {
        account.add_resource(resource);
    });

    assert_eq!(
        get_account_categories(&account, Resource).to_vec(),
        expected.to_vec()
    );

    account.remove_resource("Bank");

    let mut sorted_resources = [resources[0]].to_vec();
    sorted_resources.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let expected = sorted_resources
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    assert_eq!(
        get_account_categories(&account, Resource).to_vec(),
        expected.to_vec()
    );
}

#[wasm_bindgen_test]
fn set_correct_order_tags() {
    let mut account = Account::create();
    let tags = ["Car", "Games", "Other", "Sport"];
    let order_tags = ["Car", "Games", "Other"];
    let array = order_tags
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    tags.iter().for_each(|tag| {
        account.add_tag(tag);
    });
    account.add_order();
    assert_eq!(set_account_order_tags(&mut account, 0, array), true);
}

#[wasm_bindgen_test]
fn set_some_incorrect_order_tags() {
    let mut account = Account::create();
    let tags = ["Car", "Games", "Other", "Sport"];
    let order_tags = ["Car", "Games", "Incorrect!"];
    let array = order_tags
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    tags.iter().for_each(|tag| {
        account.add_tag(tag);
    });
    account.add_order();
    assert_eq!(set_account_order_tags(&mut account, 0, array), false);
}

#[wasm_bindgen_test]
fn set_unknown_order_tags() {
    let mut account = Account::create();
    let order_tags = ["Car", "Games", "Sport"];
    let array = order_tags
        .iter()
        .map(|item| JsValue::from(item.to_string()))
        .collect::<Array>();

    account.add_order();
    assert_eq!(set_account_order_tags(&mut account, 0, array), false);
}
