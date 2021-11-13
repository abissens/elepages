use crate::pages::PageBundle;
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;

pub struct SequenceStage {
    pub name: String,
    pub stages: Vec<Arc<dyn Stage>>,
}

impl SequenceStage {
    fn sequence_process(bundle: Arc<dyn PageBundle>, stages: &[Arc<dyn Stage>], sub_results: &mut Vec<ProcessingResult>) -> anyhow::Result<Arc<dyn PageBundle>> {
        if stages.is_empty() {
            return Ok(bundle);
        }
        let (result_bundle, p_result) = stages[0].process(&bundle)?;
        sub_results.push(p_result);
        SequenceStage::sequence_process(result_bundle, &stages[1..], sub_results)
    }
}

impl Stage for SequenceStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let mut sub_results = vec![];
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        let result_bundle = SequenceStage::sequence_process(Arc::clone(bundle), &self.stages, &mut sub_results)?;
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        Ok((
            result_bundle,
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results,
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}
