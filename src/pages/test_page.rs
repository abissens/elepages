use crate::pages::{Metadata, Page};
use crate::pages_error::PagesError;
use std::cmp::Ordering;
use std::io::{Cursor, Read};
use std::sync::Arc;

#[derive(PartialEq, Debug)]
pub(crate) struct TestPage {
    pub(crate) path: Vec<String>,
    pub(crate) metadata: Option<Metadata>,
    pub(crate) content: String,
}

impl PartialOrd for TestPage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl From<&Arc<dyn Page>> for TestPage {
    fn from(p: &Arc<dyn Page>) -> Self {
        let mut content: String = "".to_string();
        p.open().unwrap().read_to_string(&mut content).unwrap();
        TestPage {
            path: p.path().to_vec(),
            metadata: p.metadata().cloned(),
            content,
        }
    }
}

impl Page for TestPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    fn open(&self) -> Result<Box<dyn Read>, PagesError> {
        Ok(Box::new(Cursor::new(self.content.clone())))
    }
}
