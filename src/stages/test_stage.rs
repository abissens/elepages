use crate::pages::{Env, PageBundle};
use crate::stages::stage::Stage;
use crate::stages::{PageGeneratorBag, ProcessingResult};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::cmp::Ordering;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub(crate) struct TestStage {
    pub(crate) bundle: Option<Arc<dyn PageBundle>>,
    pub(crate) err_msg: Option<String>,
    pub(crate) launched: Arc<Mutex<i8>>,
}

impl TestStage {
    pub(crate) fn new(bundle: Option<Arc<dyn PageBundle>>, err_msg: Option<String>) -> Self {
        Self {
            bundle,
            err_msg,
            launched: Arc::new(Mutex::new(0)),
        }
    }

    pub(crate) fn ok(bundle: Arc<dyn PageBundle>) -> Self {
        TestStage::new(Some(bundle), None)
    }

    pub(crate) fn err(error: &str) -> Self {
        TestStage::new(None, Some(error.to_string()))
    }
}

impl Stage for TestStage {
    fn name(&self) -> String {
        "test stage".to_string()
    }

    fn process(&self, _: &Arc<dyn PageBundle>, _: &Env, _: &Arc<dyn PageGeneratorBag>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        let result_bundle = match &self.bundle {
            Some(b) => {
                let mut l = self.launched.lock().unwrap();
                *l = *l + 1;
                Ok(Arc::clone(b))
            }
            _ => match &self.err_msg {
                Some(e) => Err(anyhow!("{}", e)),
                _ => panic!("should have bundle or error"),
            },
        };
        let end = DateTime::<Utc>::from(SystemTime::now());
        Ok((
            result_bundle?,
            ProcessingResult {
                stage_name: "test stage".to_string(),
                start,
                end,
                sub_results: vec![],
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Ord)]
pub(crate) struct TestProcessingResult {
    pub(crate) stage_name: String,
    pub(crate) sub_results: Vec<TestProcessingResult>,
}

impl PartialOrd for TestProcessingResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.stage_name.partial_cmp(&other.stage_name)
    }
}

impl From<&ProcessingResult> for TestProcessingResult {
    fn from(pr: &ProcessingResult) -> Self {
        if pr.end < pr.start {
            panic!("pr.end < pr.start")
        }
        for sub_r in &pr.sub_results {
            if sub_r.start < pr.start {
                panic!("{} : sub_r.start < pr.start", sub_r.stage_name)
            }
            if sub_r.end > pr.end {
                panic!("{} : sub_r.end > pr.end", sub_r.stage_name)
            }
        }
        let mut sub_results: Vec<TestProcessingResult> = pr.sub_results.iter().map(TestProcessingResult::from).collect();
        sub_results.sort();
        TestProcessingResult {
            stage_name: pr.stage_name.clone(),
            sub_results,
        }
    }
}
