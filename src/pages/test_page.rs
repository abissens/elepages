use crate::pages::{BundleIndex, Env, Metadata, Page};
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
        p.open(
            &BundleIndex {
                all_authors: Default::default(),
                all_tags: Default::default(),
                all_pages: vec![],
                pages_by_author: Default::default(),
                pages_by_tag: Default::default(),
            },
            &Env::test(),
        )
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();
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

    fn open(&self, _: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(self.content.clone())))
    }
}
