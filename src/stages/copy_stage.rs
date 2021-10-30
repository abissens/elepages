use crate::pages::{Page, PageBundle, PageProxy, VecBundle};
use crate::stages::stage::Stage;
use std::any::Any;
use std::sync::Arc;

pub struct CopyStage {
    pub prefix: Vec<String>,
}

impl Stage for CopyStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let p = bundle
            .pages()
            .iter()
            .map(|p| {
                Arc::new(PageProxy {
                    inner: Arc::clone(p),
                    new_path: Some(join_paths(&self.prefix, p.path())),
                    new_metadata: None,
                }) as Arc<dyn Page>
            })
            .collect::<Vec<Arc<dyn Page>>>();
        Ok(Arc::new(VecBundle { p }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn join_paths(a: &[String], b: &[String]) -> Vec<String> {
    let mut result = Vec::from(a);
    result.append(&mut Vec::from(b));
    result
}
