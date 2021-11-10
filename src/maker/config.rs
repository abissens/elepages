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
    Named {
        name: String,
        stage: Box<StageValue>,
    },
    Sequence(Vec<StageValue>),
    Union {
        union: Vec<StageValue>,
    },
    Composition {
        compose: Vec<ComposeUnitConfig>,
    },
    ProcessorStage {
        #[serde(alias = "type")]
        processor_type: String,
        #[serde(default)]
        config: Value,
    },
    ProcessorWithoutConfigStage(String),
}
