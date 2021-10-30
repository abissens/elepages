use crate::pages::{Metadata, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use pulldown_cmark::{html, Parser};
use std::any::Any;
use std::io::{Cursor, Read};
use std::sync::Arc;

pub struct MdStage;

impl Stage for MdStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let vec_bundle = VecBundle {
            p: bundle
                .pages()
                .iter()
                .map(|p| {
                    let mut rel_path = p.path().to_vec();
                    let f_index = rel_path.len();
                    if f_index > 0 {
                        if let Some(ext_index) = rel_path[f_index - 1].rfind('.') {
                            rel_path[f_index - 1] = format!("{}.html", &rel_path[f_index - 1][0..ext_index]);
                        }
                    }
                    Arc::new(MdPage { source: Arc::clone(p), rel_path }) as Arc<dyn Page>
                })
                .collect(),
        };
        Ok(Arc::new(vec_bundle))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub(crate) struct MdPage {
    pub(crate) source: Arc<dyn Page>,
    rel_path: Vec<String>,
}

impl Page for MdPage {
    fn path(&self) -> &[String] {
        &self.rel_path
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.source.metadata()
    }

    fn open(&self) -> anyhow::Result<Box<dyn Read>> {
        let mut markdown_input: String = String::new();
        self.source.open()?.read_to_string(&mut markdown_input)?;

        let parser = Parser::new(&markdown_input);
        let mut html_output: String = String::with_capacity(markdown_input.len() * 3 / 2);
        html::push_html(&mut html_output, parser);

        Ok(Box::new(Cursor::new(html_output)))
    }
}
