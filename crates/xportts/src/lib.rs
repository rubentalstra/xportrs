use xportrs::Xpt;

use wasm_bindgen::prelude::*;
use tsify::Tsify;
use serde::{Serialize, Deserialize};

#[derive(Tsify, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Dataset(xportrs::Dataset);

#[wasm_bindgen]
impl Dataset {
    pub fn new(domain_code: &str) -> Self {
        Self(xportrs::Dataset::new(domain_code, vec![]).unwrap())
    }
}

#[wasm_bindgen]
pub fn to_xpt(dataset: Dataset) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    Xpt::writer(dataset.0.clone()).finalize().unwrap().write_to(&mut buf).unwrap();
    buf
}
