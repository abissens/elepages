use crate::config::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum DateQueryConfig {
    BeforeDate {
        #[serde(alias = "beforeDate")]
        before_date: String,
    },
    AfterDate {
        #[serde(alias = "afterDate")]
        after_date: String,
    },
    BeforeTime {
        #[serde(alias = "beforeTime")]
        before_time: String,
    },
    AfterTime {
        #[serde(alias = "afterTime")]
        after_time: String,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum SelectorConfig {
    PathShortCut(String),
    ConjunctionSelectorConfig(Vec<SelectorConfig>),
    Base {
        path: Option<String>,
        tag: Option<String>,
        tags: Option<Vec<String>>,
        ext: Option<String>,
        author: Option<String>,
        publishing: Option<DateQueryConfig>,
    },
    Conjunction {
        and: Vec<SelectorConfig>,
    },
    Disjunction {
        or: Vec<SelectorConfig>,
    },
    Not {
        not: Box<SelectorConfig>,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ComposeUnitConfig {
    Create(StageValue),
    Replace { inner: StageValue, selector: SelectorConfig },
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
    Append {
        append: Box<StageValue>,
    },
    Replace {
        by: Box<StageValue>,
        replace: SelectorConfig,
    },
    ProcessorStage {
        #[serde(alias = "type")]
        processor_type: String,
        #[serde(default)]
        config: Value,
    },
    ProcessorWithoutConfigStage(String),
    Copy {
        #[serde(alias = "copy")]
        copy_selector: SelectorConfig,
        dest: String,
    },
    Move {
        #[serde(alias = "move")]
        move_selector: SelectorConfig,
        dest: String,
    },
    Ignore {
        #[serde(alias = "ignore")]
        ignore_selector: SelectorConfig,
    },
}
