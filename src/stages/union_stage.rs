use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use std::rc::Rc;

pub struct UnionStage<'a> {
    pub stages: &'a [&'a dyn Stage],
}

impl Stage for UnionStage<'_> {
    fn process(&self, bundle: &dyn PageBundle) -> Box<dyn PageBundle> {
        let mut vec_bundle = VecBundle { p: vec![] };
        for stage in self.stages {
            let mut stage_pages = stage.process(bundle).pages().iter().map(|p| Rc::clone(p)).collect::<Vec<Rc<dyn Page>>>();
            vec_bundle.p.append(&mut stage_pages);
        }
        Box::new(vec_bundle)
    }
}
