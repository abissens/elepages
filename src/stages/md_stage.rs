use crate::pages::{BundleIndex, Env, Metadata, Page, PageBundle, PageIndex, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use pulldown_cmark::{html, Parser};
use std::any::Any;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::SystemTime;

pub struct MdStage {
    pub name: String,
}

impl Stage for MdStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "md processing");
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
        let end = DateTime::<Utc>::from(SystemTime::now());
        Ok((
            Arc::new(vec_bundle),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results: vec![],
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
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

    fn open(&self, output_page: &PageIndex, output_index: &BundleIndex, env: &Env) -> anyhow::Result<Box<dyn Read>> {
        let mut markdown_input: String = String::new();
        self.source.open(output_page, output_index, env)?.read_to_string(&mut markdown_input)?;

        let parser = Parser::new(&markdown_input);
        let mut html_output: String = String::with_capacity(markdown_input.len() * 3 / 2);
        html::push_html(&mut html_output, parser);

        Ok(Box::new(Cursor::new(html_output)))
    }
}
