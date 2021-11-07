use crate::cli::writer::Writer;
use crate::maker::{Env, Maker, StageValue};
use crate::pages::Loader;
use std::time::{Duration, Instant};

pub struct Executor<'a> {
    pub loader: &'a dyn Loader,
    pub maker: Maker,
    pub writer: &'a dyn Writer,
    pub stage_config: &'a StageValue,
    pub env: &'a Env,
}

impl<'a> Executor<'a> {
    pub fn new(loader: &'a dyn Loader, maker: Maker, writer: &'a dyn Writer, stage_config: &'a StageValue, env: &'a Env) -> Self {
        Self {
            loader,
            maker,
            writer,
            stage_config,
            env,
        }
    }

    pub fn execute(&self) -> anyhow::Result<Execution> {
        let start = Instant::now();

        let input_bundle = self.loader.load()?;
        let loading_elapsed = start.elapsed();

        let stage = self.maker.make(None, self.stage_config, self.env)?;
        let stage_making_elapsed = start.elapsed();

        let result_bundle = stage.process(&input_bundle)?;
        let processing_elapsed = start.elapsed();

        self.writer.write(&result_bundle)?;
        let writing_elapsed = start.elapsed();

        Ok(Execution {
            loading_elapsed,
            stage_making_elapsed,
            processing_elapsed,
            writing_elapsed,
        })
    }
}

#[derive(Debug)]
pub struct Execution {
    pub loading_elapsed: Duration,
    pub stage_making_elapsed: Duration,
    pub processing_elapsed: Duration,
    pub writing_elapsed: Duration,
}
