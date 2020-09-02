//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use js_sys::Array;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen_test::*;
use wasm_money::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn check_tags() {
    let mut account = Account::create();

    account.add_tag("Food");
    account.add_tag("Transport");
    account.add_tag("Service");
    account.add_tag("Video Games");

    let expected = Array::new();
    expected.set(0, JsValue::from("Food"));
    expected.set(1, JsValue::from("Service"));
    expected.set(2, JsValue::from("Transport"));
    expected.set(3, JsValue::from("Video Games"));

    assert_eq!(get_account_tags(&account).to_vec(), expected.to_vec());

    account.remove_tag("Transport");
    account.remove_tag("Video Games");

    let expected = Array::new();
    expected.set(0, JsValue::from("Food"));
    expected.set(1, JsValue::from("Service"));

    assert_eq!(get_account_tags(&account).to_vec(), expected.to_vec());
}

#[wasm_bindgen_test]
pub fn check_resources() {
    let mut account = Account::create();

    account.add_resource("Bank");
    account.add_resource("Cash");

    let expected = Array::new();
    expected.set(0, JsValue::from("Bank"));
    expected.set(1, JsValue::from("Cash"));

    assert_eq!(get_account_resources(&account).to_vec(), expected.to_vec());

    account.remove_resource("Bank");

    let expected = Array::new();
    expected.set(0, JsValue::from("Cash"));

    assert_eq!(get_account_resources(&account).to_vec(), expected.to_vec());
}
