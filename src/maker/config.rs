use crate::config::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ComposeUnitConfig {
    Create(StageValue),
    Replace { inner: StageValue, selector: (String, Value) },
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
        config: Value,
    },
    NamedWithoutConfig(String),
}
