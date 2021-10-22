use crate::pages::PageBundle;
use crate::stages::stage::Stage;
use anyhow::anyhow;
use std::any::Any;
use std::sync::{Arc, Mutex};

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
    fn process(&self, _: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        return match &self.bundle {
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
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
