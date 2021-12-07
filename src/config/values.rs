use crate::pages_error::PagesError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

pub trait FromValue: Sized {
    fn from_value(value: Value) -> anyhow::Result<Self>;
}

impl FromValue for String {
    fn from_value(value: Value) -> anyhow::Result<Self> {
        if let Value::String(s) = value {
            return Ok(s);
        }
        Err(PagesError::ValueParsing("expecting Value::String".to_string()).into())
    }
}

impl FromValue for Vec<String> {
    fn from_value(value: Value) -> anyhow::Result<Self> {
        if let Value::Vec(vec) = value {
            let result: anyhow::Result<Self> = vec.into_iter().map(String::from_value).collect();
            return result;
        }
        Err(PagesError::ValueParsing("expecting Value::Vec".to_string()).into())
    }
}

impl FromValue for HashMap<String, String> {
    fn from_value(value: Value) -> anyhow::Result<Self> {
        if let Value::Map(m) = value {
            let mut result = HashMap::new();
            for (k, v) in m {
                result.insert(k, String::from_value(v)?);
            }
            return Ok(result);
        }
        Err(PagesError::ValueParsing("expecting Value::Map".to_string()).into())
    }
}
