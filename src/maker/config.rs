use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ValueConfig {
    None,
    String(String),
    I32(i32),
    Bool(bool),
    Map(HashMap<String, ValueConfig>),
    Vec(Vec<ValueConfig>),
}

impl Default for ValueConfig {
    fn default() -> Self {
        ValueConfig::None
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ComposeUnitConfig {
    Create(StageValue),
    Replace { inner: StageValue, selector: (String, ValueConfig) },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum StageValue {
    Sequence(Vec<StageValue>),
    Union {
        union: Vec<StageValue>,
    },
    Composition {
        compose: Vec<ComposeUnitConfig>,
    },
    Named {
        #[serde(alias = "stage")]
        name: String,
        #[serde(default)]
        config: ValueConfig,
    },
    NamedWithoutConfig(String),
}
