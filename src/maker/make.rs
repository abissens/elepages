use crate::config::Value;
use crate::maker::config::{ComposeUnitConfig, StageValue};
use crate::pages_error::PagesError;
use crate::stages::{
    ComposeStage, ComposeUnit, ExtSelector, GitAuthors, HandlebarsDir, HandlebarsStage, IndexStage, MdStage, PrefixSelector, RegexSelector, SequenceStage, ShadowPages, Stage, SubSetSelector,
    UnionStage,
};
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub trait StageMaker {
    fn make(&self, config: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>>;
}

pub trait SelectorMaker {
    fn make(&self, config: &Value, env: &Env) -> anyhow::Result<Box<dyn SubSetSelector>>;
}

pub struct Maker {
    pub named_stage_makers: HashMap<String, Box<dyn StageMaker>>,
    pub named_selector_makers: HashMap<String, Box<dyn SelectorMaker>>,
}

pub struct GitAuthorsStageMaker;
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

impl StageMaker for GitAuthorsStageMaker {
    fn make(&self, _: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let root_path: &PathBuf = env
            .get_downcast::<PathBuf>("root_path")?
            .ok_or_else(|| PagesError::ElementNotFound("root_path not found in env".to_string()))?;

        Ok(Arc::new(GitAuthors { repo_path: root_path.to_path_buf() }))
    }
}

impl StageMaker for IndexesStageMaker {
    fn make(&self, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(IndexStage))
    }
}

impl StageMaker for MdStageMaker {
    fn make(&self, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(MdStage))
    }
}

impl StageMaker for ShadowStageMaker {
    fn make(&self, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(ShadowPages::default()))
    }
}

impl StageMaker for HandlebarsStageMaker {
    fn make(&self, _: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let root_path: &PathBuf = env
            .get_downcast::<PathBuf>("root_path")?
            .ok_or_else(|| PagesError::ElementNotFound("root_path not found in env".to_string()))?;
        Ok(Arc::new(HandlebarsStage {
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
        let mut named_stage_makers = HashMap::new();
        let mut named_selector_makers = HashMap::new();

        named_stage_makers.insert("git_authors".into(), Box::new(GitAuthorsStageMaker) as Box<dyn StageMaker>);
        named_stage_makers.insert("indexes".into(), Box::new(IndexesStageMaker) as Box<dyn StageMaker>);
        named_stage_makers.insert("md".into(), Box::new(MdStageMaker) as Box<dyn StageMaker>);
        named_stage_makers.insert("shadow".into(), Box::new(ShadowStageMaker) as Box<dyn StageMaker>);
        named_stage_makers.insert("handlebars".into(), Box::new(HandlebarsStageMaker) as Box<dyn StageMaker>);

        named_selector_makers.insert("prefix".into(), Box::new(PrefixSelectorMaker) as Box<dyn SelectorMaker>);
        named_selector_makers.insert("regex".into(), Box::new(RegexSelectorMaker) as Box<dyn SelectorMaker>);
        named_selector_makers.insert("ext".into(), Box::new(ExtSelectorMaker) as Box<dyn SelectorMaker>);

        Maker {
            named_stage_makers,
            named_selector_makers,
        }
    }

    pub fn make(&self, stage_config: &StageValue, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let stage = match stage_config {
            StageValue::Sequence(values) => Arc::new(SequenceStage {
                stages: values.iter().map(|value| self.make(value, env)).collect::<anyhow::Result<Vec<Arc<dyn Stage>>>>()?,
            }) as Arc<dyn Stage>,
            StageValue::Union { union: values } => Arc::new(UnionStage {
                stages: values.iter().map(|value| self.make(value, env)).collect::<anyhow::Result<Vec<Arc<dyn Stage>>>>()?,
                parallel: true,
            }) as Arc<dyn Stage>,
            StageValue::Composition { compose: configs } => Arc::new(ComposeStage {
                units: configs
                    .iter()
                    .map(|value| {
                        Ok(match value {
                            ComposeUnitConfig::Create(value) => Arc::new(ComposeUnit::CreateNewSet(self.make(value, env)?)),
                            ComposeUnitConfig::Replace {
                                selector: (sel_name, sel_config),
                                inner: stage_value,
                            } => {
                                let selector_maker = self
                                    .named_selector_makers
                                    .get(sel_name)
                                    .ok_or_else(|| PagesError::ElementNotFound(format!("selector {} not found", sel_name)))?;
                                let selector = selector_maker.make(sel_config, env)?;
                                Arc::new(ComposeUnit::ReplaceSubSet(selector, self.make(stage_value, env)?))
                            }
                        })
                    })
                    .collect::<anyhow::Result<Vec<Arc<ComposeUnit>>>>()?,
                parallel: true,
            }) as Arc<dyn Stage>,
            StageValue::Named { name, config } => {
                let stage_maker = self.named_stage_makers.get(name).ok_or_else(|| PagesError::ElementNotFound(format!("stage {} not found", name)))?;
                stage_maker.make(config, env)? as Arc<dyn Stage>
            }
            StageValue::NamedWithoutConfig(name) => {
                let stage_maker = self.named_stage_makers.get(name).ok_or_else(|| PagesError::ElementNotFound(format!("stage {} not found", name)))?;
                stage_maker.make(&Value::None, env)? as Arc<dyn Stage>
            }
        };

        Ok(stage)
    }
}
