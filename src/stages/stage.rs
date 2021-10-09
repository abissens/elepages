use crate::pages::PageBundle;

pub struct Pipeline {
    pub stages: Vec<Box<dyn Stage>>,
}

pub trait Stage {
    fn process(&self, bundle: &dyn PageBundle) -> Box<dyn PageBundle>;
}
