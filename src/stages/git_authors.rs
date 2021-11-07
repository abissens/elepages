use crate::pages::{ArcPage, Author, Metadata, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use git2::Repository;
use std::any::Any;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
pub struct GitAuthors {
    pub repo_path: PathBuf,
}

impl GitAuthors {
    fn process_repository(&self, mut blame_pages: HashMap<String, &Arc<dyn Page>>) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let repo = Repository::open(&self.repo_path)?;
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
                    let authors = IntoIter::new([Arc::new(Author {
                        name: commit.author().name().map(|n| n.to_string()).unwrap_or_else(|| "".to_string()),
                        contacts: commit.author().email().map(|e| IntoIter::new([e.to_string()]).collect()).unwrap_or_else(HashSet::default),
                    })])
                    .collect();
                    result.push(origin_page.change_meta(if let Some(m) = origin_page.metadata() {
                        Metadata {
                            title: m.title.clone(),
                            summary: m.summary.clone(),
                            authors,
                            tags: m.tags.clone(),
                            publishing_date: None,
                            last_edit_date: None,
                        }
                    } else {
                        Metadata {
                            title: None,
                            summary: None,
                            authors,
                            tags: HashSet::default(),
                            publishing_date: None,
                            last_edit_date: None,
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

impl Stage for GitAuthors {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut blame_pages = HashMap::default();

        for page in bundle.pages() {
            if page.path().is_empty() {
                continue;
            }
            if let Some(m) = page.metadata() {
                if !m.authors.is_empty() {
                    vec_bundle.p.push(Arc::clone(page));
                    continue;
                }
            }
            blame_pages.insert(page.path().join("/"), page);
        }

        if !blame_pages.is_empty() {
            let mut processed_pages = self.process_repository(blame_pages)?;
            vec_bundle.p.append(&mut processed_pages);
        }

        Ok(Arc::new(vec_bundle))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
