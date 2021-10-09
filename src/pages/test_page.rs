use crate::pages::{Metadata, Page};
use std::error::Error;
use std::io::{Cursor, Read};
use std::rc::Rc;

#[derive(PartialOrd, PartialEq, Debug)]
pub(crate) struct TestPage {
    pub(crate) path: Vec<String>,
    pub(crate) metadata: Option<Metadata>,
    pub(crate) content: String,
}

impl From<&Rc<dyn Page>> for TestPage {
    fn from(p: &Rc<dyn Page>) -> Self {
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

    fn open(&self) -> Result<Box<dyn Read>, Box<dyn Error>> {
        Ok(Box::new(Cursor::new(self.content.clone())))
    }
}
