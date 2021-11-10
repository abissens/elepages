use crate::cli::execute::{Execution, Executor};
use crate::cli::fs_writer::FsWriter;
use crate::config::Value;
use crate::maker::{ComposeUnitConfig, Env, Maker, StageValue};
use crate::pages::FsLoader;
use crate::pages_error::PagesError;
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Parameters {
    input_dir: PathBuf,
    output_dir: PathBuf,
    stage_config: StageValue,
}

impl Parameters {
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
                    selector: ("ext".to_string(), Value::String("md".to_string())),
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

    pub fn new(input_dir: Option<PathBuf>, output_dir: Option<PathBuf>, stage_config_file: Option<PathBuf>) -> anyhow::Result<Self> {
        let curr_dir = current_dir()?;
        let input_dir = match input_dir {
            Some(d) => d,
            None => curr_dir.clone(),
        };
        let output_dir = match output_dir {
            Some(d) => d,
            None => curr_dir.join("output"),
        };

        let stage_config = if let Some(config_file) = stage_config_file {
            Parameters::read_config(&config_file)?
        } else if input_dir.join("stage.yaml").exists() {
            Parameters::read_config(&input_dir.join("stage.yaml"))?
        } else if input_dir.join("stage.json").exists() {
            Parameters::read_config(&input_dir.join("stage.json"))?
        } else {
            Parameters::default_config()
        };

        let params = Self { input_dir, output_dir, stage_config };

        Ok(params)
    }
}

pub fn run(p: &Parameters) -> anyhow::Result<Execution> {
    let loader = FsLoader::new(p.input_dir.clone());
    let maker = Maker::default();
    let writer = FsWriter::new(p.output_dir.clone())?;

    let mut env = Env::new();
    env.insert("root_path".to_string(), Box::new(current_dir()?));

    let executor = Executor::new(&loader, maker, &writer, &p.stage_config, &env);

    executor.execute()
}
