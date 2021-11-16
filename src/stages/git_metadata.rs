use crate::pages::{ArcPage, Author, Metadata, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use git2::{ErrorCode, Repository};
use std::any::Any;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

pub struct GitMetadata {
    pub name: String,
    pub repo_path: PathBuf,
}

impl GitMetadata {
    fn process_repository(&self, repo: Repository, mut blame_pages: HashMap<String, &Arc<dyn Page>>) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let mut rev_walk = repo.revwalk()?;
        rev_walk.set_sorting(git2::Sort::TIME)?;
        rev_walk.push_head()?;

        let mut result = vec![];
        let mut commit_id = rev_walk.next();

        while !blame_pages.is_empty() && commit_id.is_some() {
            let commit = repo.find_commit(commit_id.unwrap()?)?;
            let parent_tree = if commit.parents().len() == 1 {
                let parent = commit.parent(0)?;
                Some(parent.tree()?)
            } else {
                None
            };
            let current_tree = commit.tree()?;

            let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&current_tree), None).unwrap();
            let commit_files: HashSet<String> = diff.deltas().filter_map(|d| d.new_file().path().map(|p| p.to_string_lossy().to_string())).collect();
            for commit_file in &commit_files {
                if let Some(origin_page) = blame_pages.remove(commit_file) {
                    let origin_metadata = origin_page.metadata();
                    let authors = match origin_metadata.map(|m| m.authors.clone()) {
                        Some(origin_authors) if !origin_authors.is_empty() => origin_authors,
                        _ => IntoIter::new([Arc::new(Author {
                            name: commit.author().name().map(|n| n.to_string()).unwrap_or_else(|| "".to_string()),
                            contacts: commit.author().email().map(|e| IntoIter::new([e.to_string()]).collect()).unwrap_or_else(HashSet::default),
                        })])
                        .collect(),
                    };
                    let last_edit_date = match origin_metadata {
                        Some(metadata) => match metadata.last_edit_date {
                            Some(l) => Some(l),
                            None => Some(commit.time().seconds()),
                        },
                        None => Some(commit.time().seconds()),
                    };
                    result.push(origin_page.change_meta(if let Some(m) = origin_page.metadata() {
                        Metadata {
                            title: m.title.clone(),
                            summary: m.summary.clone(),
                            authors,
                            tags: m.tags.clone(),
                            publishing_date: m.publishing_date,
                            last_edit_date,
                        }
                    } else {
                        Metadata {
                            title: None,
                            summary: None,
                            authors,
                            tags: HashSet::default(),
                            publishing_date: None,
                            last_edit_date,
                        }
                    }))
                }
            }
            commit_id = rev_walk.next();
        }
        if !blame_pages.is_empty() {
            for (_, remaining_page) in blame_pages {
                result.push(Arc::clone(remaining_page))
            }
        }
        Ok(result)
    }
}

impl Stage for GitMetadata {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        let repo_result = Repository::open(&self.repo_path);
        if let Err(e) = repo_result {
            if e.code() == ErrorCode::NotFound {
                // Ignore not found repository
                return Ok((
                    Arc::clone(bundle),
                    ProcessingResult {
                        stage_name: self.name.clone(),
                        start,
                        end: DateTime::<Utc>::from(SystemTime::now()).timestamp(),
                        sub_results: vec![],
                    },
                ));
            }
            return Err(e.into());
        }
        let repo = repo_result.unwrap();
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut blame_pages = HashMap::default();

        for page in bundle.pages() {
            if page.path().is_empty() {
                continue;
            }
            if let Some(m) = page.metadata() {
                if !m.authors.is_empty() && m.last_edit_date.is_some() {
                    vec_bundle.p.push(Arc::clone(page));
                    continue;
                }
            }
            blame_pages.insert(page.path().join("/"), page);
        }

        if !blame_pages.is_empty() {
            let mut processed_pages = self.process_repository(repo, blame_pages)?;
            vec_bundle.p.append(&mut processed_pages);
        }
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();
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
