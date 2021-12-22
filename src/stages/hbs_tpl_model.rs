use crate::config::Value;
use crate::pages::{BundleIndex, Env, FsPage, Metadata, Page};
use crate::stages::hbs_asset::HbsAsset;
use crate::stages::hbs_tpl_asset::TplAsset;
use crate::stages::{HbsPage, PageGenerator};
use std::array::IntoIter;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct TplModel {
    pub(crate) registry: handlebars::Handlebars<'static>,
    pub(crate) pages_tpl_names: HashSet<String>,
    pub(crate) assets: Vec<TplAsset>,
}

impl PageGenerator for TplModel {
    fn yield_pages(&self, output_bundle: &BundleIndex, _: &Env) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let mut result = vec![];
        for asset in &self.assets {
            match asset {
                TplAsset::Tpl { tpl_name, asset_path, metadata } => {
                    if let Some(tpl_meta) = metadata {
                        let mut pages = tpl_meta.yield_pages(&self.registry, asset_path, tpl_name, output_bundle)?;
                        result.append(&mut pages);
                    } else {
                        result.push(Arc::new(HbsAsset {
                            registry: self.registry.clone(),
                            tpl_name: tpl_name.clone(),
                            path: asset_path.clone(),
                            metadata: Some(Metadata {
                                title: None,
                                summary: None,
                                authors: Default::default(),
                                tags: Default::default(),
                                publishing_date: None,
                                last_edit_date: None,
                                data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                            }),
                            selection: None,
                        }) as Arc<dyn Page>);
                    }
                }
                TplAsset::Static { base_path, file_path } => {
                    result.push(Arc::new(FsPage::new_with_metadata(
                        base_path,
                        file_path.to_path_buf(),
                        Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                        },
                    )?) as Arc<dyn Page>);
                }
            }
        }
        Ok(result)
    }
}

impl TplModel {
    pub(crate) fn fetch(&self, page: &Arc<dyn Page>) -> Option<Arc<dyn Page>> {
        self.fetch_page_tpl_name(page).map(|tpl_name| {
            Arc::new(HbsPage {
                registry: self.registry.clone(),
                source: Arc::clone(page),
                tpl_name,
            }) as Arc<dyn Page>
        })
    }

    fn fetch_page_tpl_name(&self, page: &Arc<dyn Page>) -> Option<String> {
        let page_path = page.path();
        let l = page_path.len();
        let mut c = path_join(page_path, l - 1);
        let mut c1 = c.clone();
        c1.push_str("page.");
        c1.push_str(&page_path[l - 1]);
        if self.pages_tpl_names.contains(&c1) {
            return Some(c1);
        }

        c.push_str("page");

        if self.pages_tpl_names.contains(&c) {
            return Some(c);
        }

        for i in (0..l - 1).rev() {
            let mut c = path_join(page_path, i);
            c.push_str("page");

            if self.pages_tpl_names.contains(&c) {
                return Some(c);
            }
        }

        None
    }
}

fn path_join(v: &[String], i: usize) -> String {
    let mut c: String = v[0..i].join("/");
    if !c.is_empty() {
        c.push('/');
    }
    c
}
