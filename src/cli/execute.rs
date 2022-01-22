use crate::cli::writer::Writer;
use crate::cli::FsWriter;
use crate::config::Value;
use crate::maker::{Maker, SelectorConfig, StageValue};
use crate::pages::{Env, FsLoader, Loader, PrintLevel, ROOT_PATH_KEY};
use crate::pages_error::PagesError;
use crate::stages::{PageGeneratorBag, PageGeneratorBagImpl, ProcessingResult};
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Executor {
    pub loader: Box<dyn Loader>,
    pub maker: Maker,
    pub writer: Box<dyn Writer>,
    pub stage_config: StageValue,
    pub env: Env,
}

pub struct ExecutorParams {
    pub input_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub git_repo_path_config: Option<String>,
    pub handlebars_config: Option<Value>,
    pub print_level: Option<PrintLevel>,
}

impl Executor {
    pub fn execute(&self) -> anyhow::Result<Execution> {
        let start = Instant::now();

        let input_bundle = self.loader.load(&self.env)?;
        let loading_elapsed = start.elapsed();

        let stage = self.maker.make(None, &self.stage_config, &self.env)?;
        let stage_making_elapsed = start.elapsed();

        let gen_bag: Arc<dyn PageGeneratorBag> = PageGeneratorBagImpl::new();
        let (result_bundle, processing_result) = stage.process(&input_bundle, &self.env, &gen_bag)?;
        let processing_elapsed = start.elapsed();

        self.writer.write(&result_bundle, &self.env, &gen_bag)?;
        let writing_elapsed = start.elapsed();

        Ok(Execution {
            loading_elapsed,
            stage_making_elapsed,
            processing_elapsed,
            writing_elapsed,
            processing_result,
        })
    }

    pub fn new(params: ExecutorParams) -> anyhow::Result<Self> {
        let curr_dir = current_dir()?;
        let mut input_dir = match &params.input_dir {
            Some(d) => d.clone(),
            None => curr_dir.clone(),
        };
        let mut output_dir = match &params.output_dir {
            Some(d) => d.clone(),
            None => curr_dir.join("output"),
        };

        if input_dir.is_relative() {
            input_dir = curr_dir.join(input_dir);
        }

        if output_dir.is_relative() {
            output_dir = curr_dir.join(output_dir);
        }

        let stage_config = if let Some(config_file) = params.config_path {
            Executor::read_config(&curr_dir, &config_file)?
        } else if input_dir.join("stages.yaml").exists() {
            Executor::read_config(&curr_dir, &input_dir.join("stages.yaml"))?
        } else if input_dir.join("stages.json").exists() {
            Executor::read_config(&curr_dir, &input_dir.join("stages.json"))?
        } else {
            Executor::default_config(&params)
        };

        let loader = Box::new(FsLoader::new(input_dir.clone()));
        let maker = Maker::default();
        let writer = Box::new(FsWriter::new(output_dir)?);

        let env = Env::default_for_level(params.print_level);
        env.insert(ROOT_PATH_KEY.to_string(), Value::String(input_dir.to_string_lossy().to_string()));

        Ok(Self {
            loader,
            maker,
            writer,
            stage_config,
            env,
        })
    }

    fn default_config(params: &ExecutorParams) -> StageValue {
        let mut stages = vec![
            StageValue::ProcessorStage {
                processor_type: "shadow".to_string(),
                config: Default::default(),
            },
            StageValue::ProcessorStage {
                processor_type: "git_metadata".to_string(),
                config: match &params.git_repo_path_config {
                    None => Default::default(),
                    Some(config) => Value::String(config.to_string()),
                },
            },
            StageValue::Append {
                append: Box::new(StageValue::ProcessorStage {
                    processor_type: "indexes".to_string(),
                    config: Default::default(),
                }),
            },
            StageValue::Replace {
                by: Box::new(StageValue::ProcessorStage {
                    processor_type: "md".to_string(),
                    config: Default::default(),
                }),
                replace: SelectorConfig::Base {
                    path: None,
                    tag: None,
                    tags: None,
                    ext: Some(".md".to_string()),
                    author: None,
                    publishing: None,
                },
            },
        ];
        if let Some(handlebars_config) = &params.handlebars_config {
            stages.push(StageValue::Replace {
                by: Box::new(StageValue::ProcessorStage {
                    processor_type: "handlebars".to_string(),
                    config: handlebars_config.clone(),
                }),
                replace: SelectorConfig::Base {
                    path: None,
                    tag: None,
                    tags: None,
                    ext: Some(".html".to_string()),
                    author: None,
                    publishing: None,
                },
            })
        }
        stages.push(StageValue::ProcessorStage {
            processor_type: "path_generator".to_string(),
            config: Value::None,
        });
        StageValue::Sequence(stages)
    }

    fn read_config(curr_dir: &Path, config_file: &Path) -> anyhow::Result<StageValue> {
        if config_file.is_relative() {
            return Executor::read_config(curr_dir, &curr_dir.join(config_file));
        }
        if !config_file.exists() {
            return Err(PagesError::ElementNotFound(format!("config file {} not found", config_file.to_string_lossy())).into());
        }
        if let Some(ext) = config_file.extension().map(|e| e.to_string_lossy()) {
            if ext == "yaml" {
                let result = serde_yaml::from_reader(fs::File::open(config_file)?)?;
                return Ok(result);
            } else if ext == "json" {
                let result = serde_json::from_reader(fs::File::open(config_file)?)?;
                return Ok(result);
            }
        }
        return Err(PagesError::ElementNotFound(format!("config file {} cannot be parsed", config_file.to_string_lossy())).into());
    }
}

#[derive(Debug)]
pub struct Execution {
    pub loading_elapsed: Duration,
    pub stage_making_elapsed: Duration,
    pub processing_elapsed: Duration,
    pub writing_elapsed: Duration,
    pub processing_result: ProcessingResult,
}
