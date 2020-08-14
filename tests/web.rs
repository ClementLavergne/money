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
pub fn extract_tag_array() {
    let mut data = Account::create();

    data.add_tag("Food");
    data.add_tag("Transport");
    data.add_tag("Service");

    let output_array = data.get_tags();
    let expected_array = Array::new();

    expected_array.set(0, JsValue::from("Food"));
    expected_array.set(1, JsValue::from("Transport"));
    expected_array.set(2, JsValue::from("Service"));
    assert_eq!(expected_array.to_vec(), output_array.to_vec());
}
