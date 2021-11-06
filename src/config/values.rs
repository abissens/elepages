use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum Value {
    None,
    String(String),
    I32(i32),
    Bool(bool),
    Map(HashMap<String, Value>),
    Vec(Vec<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}
