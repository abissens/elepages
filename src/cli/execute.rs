use crate::cli::writer::Writer;
use crate::cli::FsWriter;
use crate::maker::{ComposeUnitConfig, Env, Maker, SelectorConfig, StageValue};
use crate::pages::{FsLoader, Loader};
use crate::pages_error::PagesError;
use crate::stages::ProcessingResult;
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
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
}

impl Executor {
    pub fn execute(&self) -> anyhow::Result<Execution> {
        let start = Instant::now();

        let input_bundle = self.loader.load()?;
        let loading_elapsed = start.elapsed();

        let stage = self.maker.make(None, &self.stage_config, &self.env)?;
        let stage_making_elapsed = start.elapsed();

        let (result_bundle, processing_result) = stage.process(&input_bundle)?;
        let processing_elapsed = start.elapsed();

        self.writer.write(&result_bundle)?;
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
        let input_dir = match params.input_dir {
            Some(d) => d,
            None => curr_dir.clone(),
        };
        let output_dir = match params.output_dir {
            Some(d) => d,
            None => curr_dir.join("output"),
        };

        let stage_config = if let Some(config_file) = params.config_path {
            Executor::read_config(&config_file)?
        } else if input_dir.join("stage.yaml").exists() {
            Executor::read_config(&input_dir.join("stage.yaml"))?
        } else if input_dir.join("stage.json").exists() {
            Executor::read_config(&input_dir.join("stage.json"))?
        } else {
            Executor::default_config()
        };

        let loader = Box::new(FsLoader::new(input_dir));
        let maker = Maker::default();
        let writer = Box::new(FsWriter::new(output_dir)?);

        let mut env = Env::new();
        env.insert("root_path".to_string(), Box::new(current_dir()?));

        Ok(Self {
            loader,
            maker,
            writer,
            stage_config,
            env,
        })
    }

    fn default_config() -> StageValue {
        StageValue::Sequence(vec![
            StageValue::ProcessorStage {
                processor_type: "shadow".to_string(),
                config: Default::default(),
            },
            StageValue::ProcessorStage {
                processor_type: "git_metadata".to_string(),
                config: Default::default(),
            },
            StageValue::Composition {
                compose: vec![ComposeUnitConfig::Replace {
                    inner: StageValue::ProcessorStage {
                        processor_type: "md".to_string(),
                        config: Default::default(),
                    },
                    selector: SelectorConfig::Base {
                        path: None,
                        tag: None,
                        tags: None,
                        ext: Some("md".to_string()),
                        author: None,
                        publishing: None,
                    },
                }],
            },
            StageValue::Composition {
                compose: vec![ComposeUnitConfig::Create(StageValue::ProcessorStage {
                    processor_type: "indexes".to_string(),
                    config: Default::default(),
                })],
            },
        ])
    }

    fn read_config(config_file: &Path) -> anyhow::Result<StageValue> {
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
