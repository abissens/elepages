use crate::config::Value;
use crate::pages::{BundleIndex, BundlePagination, BundleQuery, Metadata, Page, PageIndex};
use crate::stages::hbs_asset::{HbsAsset, HbsAssetSelection};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::array::IntoIter;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TplAsset {
    Tpl {
        asset_path: Vec<String>,
        tpl_name: String,
        metadata: Option<TplAssetMetadata>,
    },
    Static {
        base_path: PathBuf,
        file_path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct TplAssetMetadata {
    #[serde(alias = "query")]
    pub(crate) base_query: Option<BundleQuery>,
    #[serde(alias = "groupBy")]
    pub(crate) group_by: Option<TplAssetGroupBy>,
    #[serde(alias = "limit")]
    pub(crate) limit: Option<usize>,
    #[serde(alias = "path")]
    pub(crate) path_pattern: Option<String>,
    #[serde(alias = "firstPagePath")]
    pub(crate) first_page_path_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum TplAssetGroupBy {
    #[serde(alias = "tag")]
    Tag,
    #[serde(alias = "author")]
    Author,
}

impl TplAssetMetadata {
    fn make_path(&self, asset_path: &[String], selection: &HbsAssetSelection) -> anyhow::Result<Vec<String>> {
        if selection.index == 0 {
            if let Some(pattern) = &self.first_page_path_pattern {
                return Ok(Handlebars::new().render_template(pattern, selection)?.split('/').map(|s| s.to_string()).collect());
            }
        }
        if let Some(pattern) = &self.path_pattern {
            return Ok(Handlebars::new().render_template(pattern, selection)?.split('/').map(|s| s.to_string()).collect());
        }

        Ok(asset_path.to_vec())
    }

    pub(crate) fn yield_pages(&self, registry: &handlebars::Handlebars<'static>, asset_path: &[String], tpl_name: &str, output_bundle: &BundleIndex) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let base_query = self.base_query.clone().unwrap_or(BundleQuery::Always);
        let mut queries = vec![];

        if let Some(group_by) = &self.group_by {
            match group_by {
                TplAssetGroupBy::Tag => queries.append(
                    &mut output_bundle
                        .all_tags
                        .iter()
                        .map(|t| {
                            (
                                BundleQuery::And {
                                    and: vec![base_query.clone(), BundleQuery::Tag { tag: t.to_string() }],
                                },
                                Some(t.to_string()),
                                None,
                            )
                        })
                        .collect(),
                ),
                TplAssetGroupBy::Author => queries.append(
                    &mut output_bundle
                        .all_authors
                        .iter()
                        .map(|a| {
                            (
                                BundleQuery::And {
                                    and: vec![base_query.clone(), BundleQuery::Author { author: a.name.to_string() }],
                                },
                                None,
                                Some(a.name.to_string()),
                            )
                        })
                        .collect(),
                ),
            }
        }

        if queries.is_empty() {
            queries.push((base_query, None, None));
        }
        let no_paginate = BundlePagination { skip: None, limit: None };
        if let Some(limit) = self.limit {
            let mut result = vec![];
            for (q, selection_tag, selection_author) in &queries {
                let pages_size = output_bundle.count(q, &no_paginate);
                let nb_pages: usize = (pages_size as f32 / limit as f32).ceil() as usize;
                for p in 0..nb_pages {
                    let pages = output_bundle.query(
                        q,
                        &BundlePagination {
                            skip: Some(p * limit),
                            limit: Some(limit),
                        },
                    );
                    let selection = HbsAssetSelection {
                        pages: pages
                            .iter()
                            .map(|e| PageIndex {
                                page_ref: e.page_ref.clone(),
                                page_uri: e.page_uri.clone(),
                                metadata: e.metadata.clone(),
                            })
                            .collect::<Vec<PageIndex>>(),
                        index: p,
                        limit,
                        last: nb_pages - 1,
                        size: Some(pages_size),
                        tag: selection_tag.clone(),
                        author: selection_author.clone(),
                    };

                    result.push(Arc::new(HbsAsset {
                        registry: registry.clone(),
                        tpl_name: tpl_name.to_string(),
                        path: self.make_path(asset_path, &selection)?,
                        metadata: Some(Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                        }),
                        selection: Some(selection),
                    }) as Arc<dyn Page>);
                }
            }

            return Ok(result);
        }

        let result = queries
            .iter()
            .map(|(q, selection_tag, selection_author)| {
                let pages: Vec<PageIndex> = output_bundle
                    .query(q, &no_paginate)
                    .iter()
                    .map(|e| PageIndex {
                        page_ref: e.page_ref.clone(),
                        page_uri: e.page_uri.clone(),
                        metadata: e.metadata.clone(),
                    })
                    .collect();
                let limit = pages.len();
                let selection = HbsAssetSelection {
                    pages,
                    index: 0,
                    limit,
                    last: 0,
                    size: None,
                    tag: selection_tag.clone(),
                    author: selection_author.clone(),
                };

                Ok(Arc::new(HbsAsset {
                    registry: registry.clone(),
                    tpl_name: tpl_name.to_string(),
                    path: self.make_path(asset_path, &selection)?,
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                    }),
                    selection: Some(selection),
                }) as Arc<dyn Page>)
            })
            .collect::<anyhow::Result<Vec<Arc<dyn Page>>>>()?;

        Ok(result)
    }
}
