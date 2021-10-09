use crate::pages::{Page, PageBundle, PageProxy, VecBundle};
use crate::stages::stage::Stage;
use std::rc::Rc;

pub struct CopyStage {
    pub prefix: Vec<String>,
}

impl Stage for CopyStage {
    fn process(&self, bundle: &dyn PageBundle) -> Box<dyn PageBundle> {
        let p = bundle
            .pages()
            .iter()
            .map(|p| {
                Rc::new(PageProxy {
                    inner: Rc::clone(&p),
                    new_path: Some(join_paths(&self.prefix, p.path())),
                    new_metadata: None,
                }) as Rc<dyn Page>
            })
            .collect::<Vec<Rc<dyn Page>>>();
        Box::new(VecBundle { p })
    }
}

fn join_paths(a: &[String], b: &[String]) -> Vec<String> {
    let mut result = Vec::from(a);
    result.append(&mut Vec::from(b));
    result
}
