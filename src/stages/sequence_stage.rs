use crate::pages::PageBundle;
use crate::stages::stage::Stage;
use std::any::Any;
use std::sync::Arc;

pub struct SequenceStage {
    pub stages: Vec<Arc<dyn Stage>>,
}

impl SequenceStage {
    fn sequence_process(bundle: Arc<dyn PageBundle>, stages: &[Arc<dyn Stage>]) -> anyhow::Result<Arc<dyn PageBundle>> {
        if stages.is_empty() {
            return Ok(bundle);
        }
        let result_bundle = stages[0].process(&bundle)?;
        SequenceStage::sequence_process(result_bundle, &stages[1..])
    }
}

impl Stage for SequenceStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        SequenceStage::sequence_process(Arc::clone(bundle), &self.stages)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
