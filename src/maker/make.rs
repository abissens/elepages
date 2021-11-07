use crate::config::Value;
use crate::maker::config::{ComposeUnitConfig, StageValue};
use crate::pages_error::PagesError;
use crate::stages::{
    ComposeStage, ComposeUnit, ExtSelector, GitMetadata, HandlebarsDir, HandlebarsStage, IndexStage, MdStage, PrefixSelector, RegexSelector, SequenceStage, ShadowPages, Stage, SubSetSelector,
    UnionStage,
};
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub trait StageMaker {
    fn make(&self, name: Option<&str>, config: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>>;
}

pub trait SelectorMaker {
    fn make(&self, config: &Value, env: &Env) -> anyhow::Result<Box<dyn SubSetSelector>>;
}

pub struct Maker {
    pub processor_stage_makers: HashMap<String, Box<dyn StageMaker>>,
    pub selector_makers: HashMap<String, Box<dyn SelectorMaker>>,
}

pub struct GitMetadataStageMaker;
pub struct IndexesStageMaker;
pub struct MdStageMaker;
pub struct ShadowStageMaker;
pub struct HandlebarsStageMaker;

pub struct PrefixSelectorMaker;
pub struct RegexSelectorMaker;
pub struct ExtSelectorMaker;

pub struct Env {
    pub(crate) values: HashMap<String, Box<dyn Any>>,
}

impl Env {
    pub fn new() -> Self {
        Self { values: Default::default() }
    }

    pub fn get(&self, key: &str) -> Option<&dyn Any> {
        self.values.get(key).map(|b| b.as_ref())
    }

    pub fn get_downcast<T: 'static>(&self, key: &str) -> anyhow::Result<Option<&T>> {
        match self.values.get(key) {
            None => Ok(None),
            Some(a) => Ok(a.downcast_ref::<T>()),
        }
    }

    pub fn insert(&mut self, key: String, value: Box<dyn Any>) -> Option<Box<dyn Any>> {
        self.values.insert(key, value)
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl StageMaker for GitMetadataStageMaker {
    fn make(&self, name: Option<&str>, _: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let root_path: &PathBuf = env
            .get_downcast::<PathBuf>("root_path")?
            .ok_or_else(|| PagesError::ElementNotFound("root_path not found in env".to_string()))?;

        Ok(Arc::new(GitMetadata {
            name: name.unwrap_or("git metadata stage").to_string(),
            repo_path: root_path.to_path_buf(),
        }))
    }
}

impl StageMaker for IndexesStageMaker {
    fn make(&self, name: Option<&str>, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(IndexStage {
            name: name.unwrap_or("index stage").to_string(),
        }))
    }
}

impl StageMaker for MdStageMaker {
    fn make(&self, name: Option<&str>, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(MdStage {
            name: name.unwrap_or("markdown stage").to_string(),
        }))
    }
}

impl StageMaker for ShadowStageMaker {
    fn make(&self, name: Option<&str>, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(ShadowPages::default(name.unwrap_or("shadow pages stage").to_string())))
    }
}

impl StageMaker for HandlebarsStageMaker {
    fn make(&self, name: Option<&str>, _: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let root_path: &PathBuf = env
            .get_downcast::<PathBuf>("root_path")?
            .ok_or_else(|| PagesError::ElementNotFound("root_path not found in env".to_string()))?;
        Ok(Arc::new(HandlebarsStage {
            name: name.unwrap_or("handlebars stage").to_string(),
            lookup: Arc::new(HandlebarsDir::new(root_path)?),
        }))
    }
}

impl SelectorMaker for PrefixSelectorMaker {
    fn make(&self, config: &Value, _: &Env) -> anyhow::Result<Box<dyn SubSetSelector>> {
        if let Value::String(prefix) = config {
            return Ok(Box::new(PrefixSelector(prefix.split('/').map(|s| s.to_string()).collect())));
        }
        Err(PagesError::ElementNotFound("config should be a prefix string".to_string()).into())
    }
}

impl SelectorMaker for RegexSelectorMaker {
    fn make(&self, config: &Value, _: &Env) -> anyhow::Result<Box<dyn SubSetSelector>> {
        if let Value::String(regexp) = config {
            return Ok(Box::new(RegexSelector(Regex::new(regexp)?)));
        }
        Err(PagesError::ElementNotFound("config should be a regex string".into()).into())
    }
}

impl SelectorMaker for ExtSelectorMaker {
    fn make(&self, config: &Value, _: &Env) -> anyhow::Result<Box<dyn SubSetSelector>> {
        if let Value::String(ext) = config {
            return Ok(Box::new(ExtSelector(ext.into())));
        }
        Err(PagesError::ElementNotFound("config should be an ext string".into()).into())
    }
}

impl Maker {
    pub fn default() -> Self {
        let mut processor_stage_makers = HashMap::new();
        let mut selector_makers = HashMap::new();

        processor_stage_makers.insert("git_metadata".into(), Box::new(GitMetadataStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("indexes".into(), Box::new(IndexesStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("md".into(), Box::new(MdStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("shadow".into(), Box::new(ShadowStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("handlebars".into(), Box::new(HandlebarsStageMaker) as Box<dyn StageMaker>);

        selector_makers.insert("prefix".into(), Box::new(PrefixSelectorMaker) as Box<dyn SelectorMaker>);
        selector_makers.insert("regex".into(), Box::new(RegexSelectorMaker) as Box<dyn SelectorMaker>);
        selector_makers.insert("ext".into(), Box::new(ExtSelectorMaker) as Box<dyn SelectorMaker>);

        Maker {
            processor_stage_makers,
            selector_makers,
        }
    }

    pub fn make(&self, name: Option<&str>, stage_config: &StageValue, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let stage = match stage_config {
            StageValue::Named { name, stage } => self.make(Some(name), stage, env)?,
            StageValue::Sequence(values) => Arc::new(SequenceStage {
                name: name.unwrap_or("sequence stage").to_string(),
                stages: values.iter().map(|value| self.make(None, value, env)).collect::<anyhow::Result<Vec<Arc<dyn Stage>>>>()?,
            }) as Arc<dyn Stage>,
            StageValue::Union { union: values } => Arc::new(UnionStage {
                name: name.unwrap_or("union stage").to_string(),
                stages: values.iter().map(|value| self.make(None, value, env)).collect::<anyhow::Result<Vec<Arc<dyn Stage>>>>()?,
                parallel: true,
            }) as Arc<dyn Stage>,
            StageValue::Composition { compose: configs } => Arc::new(ComposeStage {
                name: name.unwrap_or("compose stage").to_string(),
                units: configs
                    .iter()
                    .map(|value| {
                        Ok(match value {
                            ComposeUnitConfig::Create(value) => Arc::new(ComposeUnit::CreateNewSet(self.make(None, value, env)?)),
                            ComposeUnitConfig::Replace {
                                selector: (sel_name, sel_config),
                                inner: stage_value,
                            } => {
                                let selector_maker = self
                                    .selector_makers
                                    .get(sel_name)
                                    .ok_or_else(|| PagesError::ElementNotFound(format!("selector {} not found", sel_name)))?;
                                let selector = selector_maker.make(sel_config, env)?;
                                Arc::new(ComposeUnit::ReplaceSubSet(selector, self.make(None, stage_value, env)?))
                            }
                        })
                    })
                    .collect::<anyhow::Result<Vec<Arc<ComposeUnit>>>>()?,
                parallel: true,
            }) as Arc<dyn Stage>,
            StageValue::ProcessorStage { processor_type, config } => {
                let stage_maker = self
                    .processor_stage_makers
                    .get(processor_type)
                    .ok_or_else(|| PagesError::ElementNotFound(format!("stage {} not found", processor_type)))?;
                stage_maker.make(name, config, env)? as Arc<dyn Stage>
            }
            StageValue::ProcessorWithoutConfigStage(processor_type) => {
                let stage_maker = self
                    .processor_stage_makers
                    .get(processor_type)
                    .ok_or_else(|| PagesError::ElementNotFound(format!("stage {} not found", processor_type)))?;
                stage_maker.make(name, &Value::None, env)? as Arc<dyn Stage>
            }
        };

        Ok(stage)
    }
}
