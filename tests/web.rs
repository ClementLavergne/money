//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::JsValue;

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
extern crate money;
use money::Account;
extern crate js_sys;
use js_sys::Array;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn manage_resources_and_tags() {
    let mut account = Account::create();

    account.add_tag("Food");
    account.add_tag("Transport");
    account.add_tag("");
    account.add_tag("Service");
    account.add_tag("Video Games");
    account.remove_tag("Vehicle");
    account.remove_tag("Transport");
    account.add_resource("Bank 1");
    account.add_resource("Bank 2");
    account.add_resource("Bank 1");
    account.remove_resource("");

    let resource_arr = Array::new();
    let tag_arr = Array::new();

    tag_arr.set(0, JsValue::from("Food"));
    tag_arr.set(1, JsValue::from("Service"));
    tag_arr.set(2, JsValue::from("Video Games"));
    resource_arr.set(0, JsValue::from("Bank 1"));
    resource_arr.set(1, JsValue::from("Bank 2"));

    assert_eq!(tag_arr.to_vec(), account.export_tags().to_vec());
    assert_eq!(resource_arr.to_vec(), account.export_resources().to_vec());
}
