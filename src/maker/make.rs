use crate::config::{FromValue, Value};
use crate::maker::config::{ComposeUnitConfig, StageValue};
use crate::maker::{DateQueryConfig, SelectorConfig};
use crate::pages::{AuthorSelector, DateQuery, Env, ExtSelector, Logical, PathSelector, PublishingDateSelector, Selector, TagSelector};
use crate::pages_error::PagesError;
use crate::remote::{GitReference, GitRemote};
use crate::stages::{
    AppendStage, ComposeStage, ComposeUnit, CopyCut, GitMetadata, HandlebarsDir, HandlebarsStage, IndexStage, MdStage, PathGenerator, ReplaceStage, SequenceStage, ShadowPages, Stage, UnionStage,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use dirs;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

pub trait StageMaker {
    fn make(&self, name: Option<&str>, config: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>>;
}

pub struct Maker {
    pub processor_stage_makers: HashMap<String, Box<dyn StageMaker>>,
}

pub struct GitMetadataStageMaker;
pub struct IndexesStageMaker;
pub struct MdStageMaker;
pub struct ShadowStageMaker;
pub struct HandlebarsStageMaker;
pub struct PathGeneratorStageMaker;

impl StageMaker for GitMetadataStageMaker {
    fn make(&self, name: Option<&str>, config: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let root_path: &PathBuf = env
            .get_downcast::<PathBuf>("root_path")?
            .ok_or_else(|| PagesError::ElementNotFound("root_path not found in env".to_string()))?;
        let (repo_path, pages_rel_path) = match config {
            Value::String(config_repo_path) => {
                let p = PathBuf::from_str(config_repo_path)?;
                let r = root_path.canonicalize()?.strip_prefix(&p.canonicalize()?)?.to_path_buf();
                (p, Some(r))
            }
            _ => (root_path.to_path_buf(), None),
        };
        Ok(Arc::new(GitMetadata {
            name: name.unwrap_or("git metadata stage").to_string(),
            repo_path,
            pages_rel_path,
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

struct HandlebarsStageMakerConfig {
    path: Option<String>,
    remote: Option<String>,
    git_reference: Option<GitReference>,
}

impl FromValue for HandlebarsStageMakerConfig {
    fn from_value(value: Value) -> anyhow::Result<Self> {
        if let Ok(s) = String::from_value(value.clone()) {
            if s.starts_with("http://") || s.starts_with("https://") {
                return Ok(HandlebarsStageMakerConfig {
                    path: None,
                    remote: Some(s),
                    git_reference: Some(GitReference::Branch("main".to_string())),
                });
            }
            return Ok(HandlebarsStageMakerConfig {
                path: Some(s),
                remote: None,
                git_reference: None,
            });
        }
        if let Ok(m) = <HashMap<String, String>>::from_value(value) {
            #[allow(clippy::manual_map)]
            let git_reference = if let Some(r) = m.get("commit").map(|v| GitReference::Commit(v.to_string())) {
                Some(r)
            } else if let Some(r) = m.get("branch").map(|v| GitReference::Branch(v.to_string())) {
                Some(r)
            } else if let Some(r) = m.get("tag").map(|v| GitReference::Tag(v.to_string())) {
                Some(r)
            } else {
                None
            };
            return Ok(HandlebarsStageMakerConfig {
                path: m.get("path").cloned(),
                remote: m.get("remote").cloned(),
                git_reference,
            });
        }
        Err(PagesError::ValueParsing("expecting Value::String or Value::Map".to_string()).into())
    }
}

impl StageMaker for HandlebarsStageMaker {
    fn make(&self, name: Option<&str>, config: &Value, env: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        let template_path: PathBuf;
        let hbs_config = HandlebarsStageMakerConfig::from_value(config.clone())?;
        if let Some(remote) = hbs_config.remote {
            let home_dir = dirs::home_dir().ok_or_else(|| PagesError::ElementNotFound("cannot locate home directory".to_string()))?;
            let git_remote = GitRemote::new(&home_dir, &remote, &hbs_config.git_reference.unwrap_or_else(|| GitReference::Branch("main".to_string())), env)?;
            template_path = if let Some(sub_dir) = hbs_config.path {
                let sub_path = PathBuf::from_str(&sub_dir)?;
                git_remote.local_dir.join(&sub_path)
            } else {
                git_remote.local_dir
            };
        } else if let Some(local_dir) = hbs_config.path {
            template_path = PathBuf::from_str(&local_dir)?;
        } else {
            return Err(PagesError::ElementNotFound("cannot find configuration".to_string()).into());
        }
        Ok(Arc::new(HandlebarsStage {
            name: name.unwrap_or("handlebars stage").to_string(),
            lookup: Arc::new(HandlebarsDir { base_path: template_path }),
        }))
    }
}

impl StageMaker for PathGeneratorStageMaker {
    fn make(&self, name: Option<&str>, _: &Value, _: &Env) -> anyhow::Result<Arc<dyn Stage>> {
        Ok(Arc::new(PathGenerator::new(name.unwrap_or("path generator stage").to_string())))
    }
}

impl Maker {
    pub fn default() -> Self {
        let mut processor_stage_makers = HashMap::new();

        processor_stage_makers.insert("git_metadata".into(), Box::new(GitMetadataStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("indexes".into(), Box::new(IndexesStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("md".into(), Box::new(MdStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("shadow".into(), Box::new(ShadowStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("handlebars".into(), Box::new(HandlebarsStageMaker) as Box<dyn StageMaker>);
        processor_stage_makers.insert("path_generator".into(), Box::new(PathGeneratorStageMaker) as Box<dyn StageMaker>);

        Maker { processor_stage_makers }
    }

    fn make_date_config(date_config: &DateQueryConfig) -> anyhow::Result<DateQuery> {
        match date_config {
            DateQueryConfig::BeforeDate { before_date } => {
                if before_date == "now" {
                    let ts = DateTime::<Utc>::from(SystemTime::now()).date().and_hms(0, 0, 0).timestamp();
                    return Ok(DateQuery::Before(ts));
                }
                let ts = NaiveDate::from_str(before_date)?.and_hms(0, 0, 0).timestamp();
                Ok(DateQuery::Before(ts))
            }
            DateQueryConfig::AfterDate { after_date } => {
                if after_date == "now" {
                    let ts = DateTime::<Utc>::from(SystemTime::now()).date().and_hms(23, 59, 59).timestamp();
                    return Ok(DateQuery::After(ts));
                }
                let ts = NaiveDate::from_str(after_date)?.and_hms(23, 59, 59).timestamp();
                Ok(DateQuery::After(ts))
            }
            DateQueryConfig::BeforeTime { before_time } => {
                if before_time == "now" {
                    let ts = DateTime::<Utc>::from(SystemTime::now()).timestamp();
                    return Ok(DateQuery::Before(ts));
                }
                let ts = NaiveDateTime::from_str(before_time)?.timestamp();
                Ok(DateQuery::Before(ts))
            }
            DateQueryConfig::AfterTime { after_time } => {
                if after_time == "now" {
                    let ts = DateTime::<Utc>::from(SystemTime::now()).timestamp();
                    return Ok(DateQuery::After(ts));
                }
                let ts = NaiveDateTime::from_str(after_time)?.timestamp();
                Ok(DateQuery::After(ts))
            }
        }
    }

    fn make_selector(config: &SelectorConfig) -> anyhow::Result<Arc<dyn Selector>> {
        let selector: Arc<dyn Selector> = match config {
            SelectorConfig::Conjunction { and } => Arc::new(Logical::And(and.iter().map(|sc| Maker::make_selector(sc)).collect::<anyhow::Result<Vec<Arc<dyn Selector>>>>()?)) as Arc<dyn Selector>,
            SelectorConfig::Disjunction { or } => Arc::new(Logical::Or(or.iter().map(|sc| Maker::make_selector(sc)).collect::<anyhow::Result<Vec<Arc<dyn Selector>>>>()?)) as Arc<dyn Selector>,
            SelectorConfig::Not { not } => Arc::new(Logical::Not(Maker::make_selector(not)?)) as Arc<dyn Selector>,

            SelectorConfig::PathShortCut(path) => Arc::new(PathSelector {
                query: path.split('/').map(|s| s.to_string()).collect(),
            }) as Arc<dyn Selector>,
            SelectorConfig::ConjunctionSelectorConfig(and) => Maker::make_selector(&SelectorConfig::Conjunction { and: and.to_vec() })?,
            SelectorConfig::Base {
                path,
                tag,
                tags,
                ext,
                author,
                publishing,
            } => {
                let mut selectors: Vec<Arc<dyn Selector>> = vec![];
                if let Some(v) = path {
                    selectors.push(Arc::new(PathSelector {
                        query: v.split('/').map(|s| s.to_string()).collect(),
                    }) as Arc<dyn Selector>)
                }
                if let Some(v) = tag {
                    selectors.push(Arc::new(TagSelector { tag: v.to_string() }) as Arc<dyn Selector>)
                }
                if let Some(v) = tags {
                    if v.len() == 1 {
                        selectors.push(Arc::new(TagSelector { tag: v[0].to_string() }) as Arc<dyn Selector>)
                    }
                    if v.len() > 1 {
                        selectors.push(Arc::new(Logical::And(v.iter().map(|v| Arc::new(TagSelector { tag: v.to_string() }) as Arc<dyn Selector>).collect())) as Arc<dyn Selector>)
                    }
                }
                if let Some(v) = ext {
                    selectors.push(Arc::new(ExtSelector { ext: v.to_string() }) as Arc<dyn Selector>)
                }
                if let Some(v) = author {
                    selectors.push(Arc::new(AuthorSelector { author: v.to_string() }) as Arc<dyn Selector>)
                }
                if let Some(v) = publishing {
                    selectors.push(Arc::new(PublishingDateSelector { query: Maker::make_date_config(v)? }) as Arc<dyn Selector>)
                }
                if selectors.is_empty() {
                    return Err(PagesError::ValueParsing("cannot parse selector".to_string()).into());
                }
                if selectors.len() == 1 {
                    selectors.pop().unwrap()
                } else {
                    Arc::new(Logical::And(selectors)) as Arc<dyn Selector>
                }
            }
        };

        Ok(selector)
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
                            ComposeUnitConfig::Create { append: value } => Arc::new(ComposeUnit::CreateNewSet(self.make(None, value, env)?)),
                            ComposeUnitConfig::Replace {
                                replace: selector_config,
                                by: stage_value,
                            } => {
                                let selector = Maker::make_selector(selector_config)?;
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
            StageValue::Copy { dest, copy_selector } => {
                let selector = Maker::make_selector(copy_selector)?;
                Arc::new(CopyCut::Copy {
                    name: name.unwrap_or("copy stage").to_string(),
                    selector,
                    dest: dest.split('/').map(|s| s.to_string()).collect(),
                })
            }
            StageValue::Move { dest, move_selector } => {
                let selector = Maker::make_selector(move_selector)?;
                Arc::new(CopyCut::Move {
                    name: name.unwrap_or("move stage").to_string(),
                    selector,
                    dest: dest.split('/').map(|s| s.to_string()).collect(),
                })
            }
            StageValue::Ignore { ignore_selector } => {
                let selector = Maker::make_selector(ignore_selector)?;
                Arc::new(CopyCut::Ignore {
                    name: name.unwrap_or("ignore stage").to_string(),
                    selector,
                })
            }
            StageValue::Append { append } => Arc::new(AppendStage {
                name: name.unwrap_or("append stage").to_string(),
                inner: self.make(None, append, env)?,
            }),
            StageValue::Replace { replace, by } => Arc::new(ReplaceStage {
                name: name.unwrap_or("replace stage").to_string(),
                inner: self.make(None, by, env)?,
                selector: Maker::make_selector(replace)?,
            }),
        };

        Ok(stage)
    }
}
