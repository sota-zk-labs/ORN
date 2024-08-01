use std::collections::HashMap;
use math_parse::MathParse;
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

pub fn get_variables_map(a: &HashMap<String, ConstantValue>) -> HashMap<String, String> {
    a.iter().map(|(k, v)| {
        (k.clone(), v.value.to_string())
    }).collect::<HashMap<_, _>>()
}

pub fn get_constant_values() -> HashMap<String, ConstantValue> {
    let constant_values = toml::from_str(CONSTANT_VALUES).unwrap();
    let variables_map = get_variables_map(&constant_values);
    constant_values.into_iter().map(|(k, mut v)| {
        let value = match MathParse::parse(&v.value) {
            Err(_) => {
                v.value
            }
            Ok(expression) => {
                match expression.solve_int(Some(&variables_map)) {
                    Err(_) => {
                        v.value
                    }
                    Ok(result) => {
                        format!("0x{:x}", result).to_string()
                    }
                }
            }
        };
        v.value = value;
        (k, v)
    }).collect::<HashMap<_, _>>()
}

