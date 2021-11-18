use std::any::Any;
use std::collections::HashMap;

pub struct Env {
    pub(crate) values: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Env {
    pub fn new() -> Self {
        Self { values: Default::default() }
    }

    pub fn get(&self, key: &str) -> Option<&(dyn Any + Send + Sync)> {
        self.values.get(key).map(|b| b.as_ref())
    }

    pub fn get_downcast<T: 'static>(&self, key: &str) -> anyhow::Result<Option<&T>> {
        match self.values.get(key) {
            None => Ok(None),
            Some(a) => Ok(a.downcast_ref::<T>()),
        }
    }

    pub fn insert(&mut self, key: String, value: Box<dyn Any + Send + Sync>) -> Option<Box<dyn Any + Send + Sync>> {
        self.values.insert(key, value)
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
