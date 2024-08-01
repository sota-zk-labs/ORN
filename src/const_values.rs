use std::collections::HashMap;

use serde::Deserialize;

const CONSTANT_VALUES: &str = include_str!("const_values.toml");

#[derive(Debug)]
pub enum ConstantTypes {
    String { value: String },
    Number { value: String },
    Hex { value: String },
}

#[derive(Debug, Deserialize)]
pub struct ConstantValue {
    r#type: String,
    value: String,
    comment: String,
}

pub fn get_constant_values() -> HashMap<String, ConstantValue> {
    toml::from_str(CONSTANT_VALUES).unwrap()
}
