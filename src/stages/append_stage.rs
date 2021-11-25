use crate::pages::{Env, PageBundle, VecBundle};
use crate::stages::{ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;

pub struct AppendStage {
    pub name: String,
    pub inner: Arc<dyn Stage>,
}

impl Stage for AppendStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name), "start appending");
        let mut vec_bundle = VecBundle { p: bundle.pages().to_vec() };

        let (inner_bundle, inner_result) = self.inner.process(bundle, env)?;
        vec_bundle.p.append(&mut inner_bundle.pages().to_vec());

        env.print_vv(&format!("stage {}", self.name), "append ended");
        let end = DateTime::<Utc>::from(SystemTime::now());
        Ok((
            Arc::new(vec_bundle),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results: vec![inner_result],
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}
