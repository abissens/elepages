use crate::pages::{ArcPage, BundleIndex, Env, Metadata, Page, PageBundle, PageIndex, VecBundle};
use crate::stages::metadata_tree::MetadataTree;
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::any::Any;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::option::Option::Some;
use std::sync::Arc;
use std::time::SystemTime;

pub trait ShadowLoader: Send + Sync {
    fn load(&self, page: Arc<dyn Page>, shadow_page_index: &PageIndex, shadow_output_index: &BundleIndex, env: &Env) -> anyhow::Result<Metadata>;
}

pub struct ShadowPages {
    pub name: String,
    pub loaders: HashMap<String, Arc<dyn ShadowLoader>>,
}

impl Stage for ShadowPages {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "shadow metadata page processing started");
        let shadow_output_index = BundleIndex::from(bundle);

        let mut vec_bundle = VecBundle { p: vec![] };

        let mut metadata_candidates = vec![];
        let mut all_paths = HashSet::new();

        // select metadata candidates
        for page in bundle.pages() {
            let path = page.path();
            if path.is_empty() {
                continue;
            }
            for (ext, loader) in &self.loaders {
                if path[path.len() - 1].ends_with(ext) {
                    metadata_candidates.push(MetadataCandidate {
                        path: page
                            .path()
                            .iter()
                            .enumerate()
                            .map(|(pos, p)| {
                                if pos == path.len() - 1 {
                                    return p.strip_suffix(ext).unwrap().to_string();
                                }
                                p.to_string()
                            })
                            .collect(),
                        page,
                        loader,
                    })
                }
            }
            for i in 0..path.len() {
                all_paths.insert(&path[..i + 1]);
            }
        }

        // retain only shadow pages
        let root_metadata_path = vec!["pages".to_string()];
        metadata_candidates.retain(|c| c.path == root_metadata_path || all_paths.contains(c.path.as_slice()));

        let loaded_metadata_vec: Vec<LoadedMetadata> = metadata_candidates
            .par_iter()
            .map(|metadata_candidate: &MetadataCandidate| {
                let c_page = Arc::clone(metadata_candidate.page);
                let c_page_index = PageIndex::from(&c_page);
                let path = metadata_candidate.path.clone();
                metadata_candidate
                    .loader
                    .load(c_page, &c_page_index, &shadow_output_index, env)
                    .map(|metadata| LoadedMetadata { path, metadata })
            })
            .collect::<anyhow::Result<Vec<LoadedMetadata>>>()?;

        // feed metadata tree
        let mut metadata_tree = MetadataTree::Root { sub: HashMap::new() };

        for loaded_metadata in loaded_metadata_vec {
            metadata_tree.push(&loaded_metadata.path, loaded_metadata.metadata)?
        }

        let mut root_metadata_vec = vec![];
        metadata_tree.get_metadata_from_path(&root_metadata_path, &mut root_metadata_vec);
        let root_metadata = if root_metadata_vec.len() == 1 { root_metadata_vec[0].metadata.cloned() } else { None };

        let metadata_pages_set = metadata_candidates.iter().map(|c| c.page.path().to_vec()).collect::<HashSet<Vec<String>>>();

        // push non metadata pages to result bundle
        for page in bundle.pages() {
            if !metadata_pages_set.contains(page.path()) {
                // get path metadata
                let mut metadata_vec = vec![];
                metadata_tree.get_metadata_from_path(page.path(), &mut metadata_vec);
                if metadata_vec.is_empty() && root_metadata.is_none() {
                    vec_bundle.p.push(Arc::clone(page));
                    continue;
                } else if metadata_vec.is_empty() && page.metadata().is_none() {
                    // default metadata
                    let current_metadata = Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    };
                    vec_bundle.p.push(page.change_meta(current_metadata.merge(root_metadata.as_ref().unwrap())?));
                    continue;
                } else if metadata_vec.is_empty() {
                    vec_bundle.p.push(page.change_meta(page.metadata().unwrap().merge(root_metadata.as_ref().unwrap())?));
                    continue;
                }
                let mut current_metadata: Metadata;
                if let Some(page_metadata) = page.metadata() {
                    // page has already metadata
                    current_metadata = page_metadata.clone();
                } else if metadata_vec.len() == page.path().len() {
                    // dedicated metadata for current page
                    current_metadata = metadata_vec
                        .pop()
                        .unwrap()
                        .metadata
                        .unwrap_or(&Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        })
                        .clone();
                } else {
                    // default metadata
                    current_metadata = Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }
                }

                while let Some(metadata_node) = metadata_vec.pop() {
                    if let Some(m) = metadata_node.metadata {
                        current_metadata = current_metadata.merge(m)?;
                    }
                }
                if let Some(rm) = &root_metadata {
                    current_metadata = current_metadata.merge(rm)?;
                }
                vec_bundle.p.push(page.change_meta(current_metadata));
            }
        }
        env.print_vv(&format!("stage {}", self.name()), "shadow metadata page processing ended");
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

impl ShadowPages {
    pub fn new(name: String, loaders: HashMap<String, Arc<dyn ShadowLoader>>) -> Self {
        ShadowPages { name, loaders }
    }

    pub fn default(name: String) -> Self {
        ShadowPages {
            name,
            loaders: IntoIter::new([
                (".json".to_string(), Arc::new(JsonShadowLoader()) as Arc<dyn ShadowLoader>),
                (".yaml".to_string(), Arc::new(YamlShadowLoader()) as Arc<dyn ShadowLoader>),
            ])
            .collect(),
        }
    }
}

struct JsonShadowLoader();
struct YamlShadowLoader();

impl ShadowLoader for JsonShadowLoader {
    fn load(&self, page: Arc<dyn Page>, shadow_page_index: &PageIndex, shadow_output_index: &BundleIndex, env: &Env) -> anyhow::Result<Metadata> {
        env.print_vvv("json shadow loader", &format!("loading from page {}", page.path().join("/")));
        Ok(serde_json::from_reader(page.open(shadow_page_index, shadow_output_index, env)?)?)
    }
}

impl ShadowLoader for YamlShadowLoader {
    fn load(&self, page: Arc<dyn Page>, shadow_page_index: &PageIndex, shadow_output_index: &BundleIndex, env: &Env) -> anyhow::Result<Metadata> {
        env.print_vvv("yaml shadow loader", &format!("loading from page {}", page.path().join("/")));
        Ok(serde_yaml::from_reader(page.open(shadow_page_index, shadow_output_index, env)?)?)
    }
}

struct MetadataCandidate<'a> {
    path: Vec<String>,
    page: &'a Arc<dyn Page>,
    loader: &'a Arc<dyn ShadowLoader>,
}

struct LoadedMetadata {
    path: Vec<String>,
    metadata: Metadata,
}
