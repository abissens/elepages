use crate::pages::{Author, Metadata, PageBundle, PageProxy, VecBundle};
use crate::stages::stage::Stage;
use git2::{BlameOptions, Repository};
use std::array::IntoIter;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

pub struct GitAuthors {
    pub repo_path: PathBuf,
}

impl Stage for GitAuthors {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut blame_pages = vec![];

        for page in bundle.pages() {
            if let Some(m) = page.metadata() {
                if !m.authors.is_empty() {
                    vec_bundle.p.push(Arc::clone(page));
                    continue;
                }
            }
            blame_pages.push(page);
        }

        if !blame_pages.is_empty() {
            let repo = Repository::open(&self.repo_path)?;
            let mut opts = BlameOptions::new();
            opts.track_copies_same_commit_moves(true).track_copies_same_commit_copies(true).first_parent(true);

            let commit_id = "HEAD".to_string();

            for page in blame_pages {
                let path = PathBuf::from(page.path().join("/"));

                let spec = format!("{}:{}", commit_id, path.display());
                let blame = repo.blame_file(&path, Some(&mut opts))?;
                let object = repo.revparse_single(&spec[..])?;
                let blob = repo.find_blob(object.id())?;
                let reader = BufReader::new(blob.content());
                let mut page_authors = HashSet::new();
                for (i, line) in reader.lines().enumerate() {
                    if let (Ok(line), Some(hunk)) = (line, blame.get_line(i + 1)) {
                        let sig = hunk.final_signature();
                        let name = String::from_utf8_lossy(sig.name_bytes());
                        let email = String::from_utf8_lossy(sig.email_bytes());
                        let size = line.len();
                        let stats = AuthorStats {
                            name: name.to_string(),
                            email: email.to_string(),
                            size,
                        };
                        if !page_authors.contains(&stats) {
                            page_authors.insert(stats);
                        } else {
                            page_authors.replace(AuthorStats {
                                name: name.to_string(),
                                email: email.to_string(),
                                size: stats.size + size,
                            });
                        }
                    }
                }

                vec_bundle.p.push(Arc::new(PageProxy {
                    new_path: None,
                    new_metadata: page
                        .metadata()
                        .map(|m| Metadata {
                            title: m.title.clone(),
                            summary: m.summary.clone(),
                            authors: page_authors
                                .iter()
                                .map(|pa| Author {
                                    name: pa.name.to_string(),
                                    contacts: IntoIter::new([pa.email.to_string()]).collect(),
                                })
                                .collect(),
                            tags: m.tags.clone(),
                        })
                        .or_else(|| {
                            Some(Metadata {
                                title: None,
                                summary: None,
                                authors: page_authors
                                    .iter()
                                    .map(|pa| Author {
                                        name: pa.name.to_string(),
                                        contacts: IntoIter::new([pa.email.to_string()]).collect(),
                                    })
                                    .collect(),
                                tags: Default::default(),
                            })
                        }),
                    inner: Arc::clone(page),
                }))
            }
        }

        Ok(Arc::new(vec_bundle))
    }
}

struct AuthorStats {
    name: String,
    email: String,
    size: usize,
}
impl Eq for AuthorStats {}
impl PartialEq for AuthorStats {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Hash for AuthorStats {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
