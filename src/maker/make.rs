use crate::maker::config::{ComposeUnitConfig, StageValue, ValueConfig};
use crate::pages_error::PagesError;
use crate::stages::compose_stage::{ComposeStage, ComposeUnit, ExtSelector, PrefixSelector, RegexSelector, SubSetSelector};
use crate::stages::git_authors::GitAuthors;
use crate::stages::handlebars_stage::{HandlebarsDir, HandlebarsStage};
use crate::stages::indexes_stage::IndexStage;
use crate::stages::md_stage::MdStage;
use crate::stages::sequence_stage::SequenceStage;
use crate::stages::shadow_pages::ShadowPages;
use crate::stages::stage::Stage;
use crate::stages::union_stage::UnionStage;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

pub trait StageMaker {
    fn make(&self, config: &ValueConfig, env: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>>;
}

pub trait SelectorMaker {
    fn make(&self, config: &ValueConfig, env: &HashMap<String, ValueConfig>) -> anyhow::Result<Box<dyn SubSetSelector>>;
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

impl StageMaker for GitAuthorsStageMaker {
    fn make(&self, _: &ValueConfig, env: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>> {
        if let Some(ValueConfig::String(root_path)) = env.get("root_path") {
            return Ok(Arc::new(GitAuthors {
                repo_path: PathBuf::from_str(root_path)?,
            }));
        }
        Err(PagesError::NamedValueNotFound("root_path".into()).into())
    }
}

impl StageMaker for IndexesStageMaker {
    fn make(&self, _: &ValueConfig, _: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(IndexStage))
    }
}

impl StageMaker for MdStageMaker {
    fn make(&self, _: &ValueConfig, _: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(MdStage))
    }
}

impl StageMaker for ShadowStageMaker {
    fn make(&self, _: &ValueConfig, _: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(ShadowPages::default()))
    }
}

impl StageMaker for HandlebarsStageMaker {
    fn make(&self, _: &ValueConfig, env: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>> {
        if let Some(ValueConfig::String(root_path)) = env.get("root_path") {
            return Ok(Arc::new(HandlebarsStage {
                lookup: Arc::new(HandlebarsDir::new(&PathBuf::from_str(root_path)?)?),
            }));
        }
        Err(PagesError::NamedValueNotFound("path not found".into()).into())
    }
}

impl SelectorMaker for PrefixSelectorMaker {
    fn make(&self, config: &ValueConfig, _: &HashMap<String, ValueConfig>) -> anyhow::Result<Box<dyn SubSetSelector>> {
        if let ValueConfig::String(prefix) = config {
            return Ok(Box::new(PrefixSelector(prefix.split('/').map(|s| s.to_string()).collect())));
        }
        Err(PagesError::NamedValueNotFound("prefix not found".into()).into())
    }
}

impl SelectorMaker for RegexSelectorMaker {
    fn make(&self, config: &ValueConfig, _: &HashMap<String, ValueConfig>) -> anyhow::Result<Box<dyn SubSetSelector>> {
        if let ValueConfig::String(regexp) = config {
            return Ok(Box::new(RegexSelector(Regex::new(regexp)?)));
        }
        Err(PagesError::NamedValueNotFound("regexp not found".into()).into())
    }
}

impl SelectorMaker for ExtSelectorMaker {
    fn make(&self, config: &ValueConfig, _: &HashMap<String, ValueConfig>) -> anyhow::Result<Box<dyn SubSetSelector>> {
        if let ValueConfig::String(ext) = config {
            return Ok(Box::new(ExtSelector(ext.into())));
        }
        Err(PagesError::NamedValueNotFound("path not found".into()).into())
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

    pub fn make(&self, value: &StageValue, env: &HashMap<String, ValueConfig>) -> anyhow::Result<Arc<dyn Stage>> {
        let stage = match value {
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
                                let selector_maker = self.named_selector_makers.get(sel_name).ok_or_else(|| PagesError::NamedValueNotFound("selector not found".into()))?;
                                let selector = selector_maker.make(sel_config, env)?;
                                Arc::new(ComposeUnit::ReplaceSubSet(selector, self.make(stage_value, env)?))
                            }
                        })
                    })
                    .collect::<anyhow::Result<Vec<Arc<ComposeUnit>>>>()?,
                parallel: true,
            }) as Arc<dyn Stage>,
            StageValue::Named { name, config } => {
                let stage_maker = self.named_stage_makers.get(name).ok_or_else(|| PagesError::NamedValueNotFound("stage not found".into()))?;
                stage_maker.make(config, env)? as Arc<dyn Stage>
            }
            StageValue::NamedWithoutConfig(name) => {
                let stage_maker = self.named_stage_makers.get(name).ok_or_else(|| PagesError::NamedValueNotFound("stage not found".into()))?;
                stage_maker.make(&ValueConfig::None, env)? as Arc<dyn Stage>
            }
        };

        Ok(stage)
    }
}
