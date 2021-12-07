use crate::pages::Env;
use crate::pages_error::PagesError;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Direction, Oid, Remote, RemoteHead, Repository};
use std::fmt::{Display, Formatter};
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Debug)]
pub struct GitRemote {
    pub remote: String,
    pub oid: Oid,
    pub local_dir: PathBuf,
}

pub enum GitReference {
    Commit(String),
    Branch(String),
    Tag(String),
}

impl Display for GitReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GitReference::Commit(v) => v.fmt(f),
            GitReference::Branch(v) => v.fmt(f),
            GitReference::Tag(v) => v.fmt(f),
        }
    }
}

impl GitRemote {
    fn find_or_make_local_dir(home_dir: &Path, remote: &str) -> anyhow::Result<(PathBuf, bool)> {
        let mut result_dir = home_dir.join("elepages");
        let mut created = false;
        if !result_dir.exists() {
            create_dir(&result_dir)?;
            created = true;
        }

        let parsed_remote = Url::parse(remote)?;

        if let Some(domain) = parsed_remote.domain() {
            result_dir = result_dir.join(domain);
            if !result_dir.exists() {
                create_dir(&result_dir)?;
                created = true;
            }
        }
        if let Some(segments) = parsed_remote.path_segments() {
            for segment in segments {
                result_dir = result_dir.join(segment);
                if !result_dir.exists() {
                    create_dir(&result_dir)?;
                    created = true;
                }
            }
        }
        Ok((result_dir, created))
    }

    fn fetch_oid_from_remote_ref(remote: &str, reference: &str) -> anyhow::Result<Oid> {
        let mut remote = Remote::create_detached(remote)?;
        remote.connect(Direction::Fetch)?;
        let remote_ls = remote.list()?;
        let ref_item: &RemoteHead = remote_ls
            .iter()
            .find(|e| e.name() == reference)
            .ok_or_else(|| PagesError::ElementNotFound(format!("ref name {} not found", reference)))?;
        Ok(ref_item.oid())
    }

    fn fetch_remote_ref_oid(remote: &str, reference: &GitReference) -> anyhow::Result<Oid> {
        match reference {
            GitReference::Commit(commit) => Ok(Oid::from_str(commit)?),
            GitReference::Branch(branch) => GitRemote::fetch_oid_from_remote_ref(remote, &format!("refs/heads/{}", branch)),
            GitReference::Tag(tag) => GitRemote::fetch_oid_from_remote_ref(remote, &format!("refs/tags/{}", tag)),
        }
    }

    fn fetch_local_ref_oid(repo: &Repository, reference: &GitReference) -> anyhow::Result<Oid> {
        match reference {
            GitReference::Commit(commit) => Ok(Oid::from_str(commit)?),
            GitReference::Branch(branch) => {
                let tag_ref = repo.find_reference(&format!("refs/remotes/origin/{}", branch))?;
                Ok(tag_ref.peel_to_commit()?.id())
            }
            GitReference::Tag(tag) => {
                let tag_ref = repo.find_reference(&format!("refs/tags/{}", tag))?;
                Ok(tag_ref.peel_to_commit()?.id())
            }
        }
    }

    pub fn new(home_dir: &Path, remote: &str, reference: &GitReference, env: &Env) -> anyhow::Result<Self> {
        env.print_v("GitRemote", &format!("loading {} {}", remote, reference));
        if !home_dir.exists() {
            return Err(PagesError::ElementNotFound(format!("home dir {} not found", home_dir.to_string_lossy())).into());
        }

        let (local_dir, local_dir_created) = GitRemote::find_or_make_local_dir(home_dir, remote)?;
        env.print_vv("GitRemote", &format!("local dir : {}", local_dir.to_string_lossy()));
        if local_dir_created {
            let repo = RepoBuilder::new().clone(remote, &local_dir)?;
            let oid = GitRemote::fetch_local_ref_oid(&repo, reference)?;
            repo.set_head_detached(oid)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
            return Ok(Self {
                remote: remote.to_string(),
                oid,
                local_dir,
            });
        }

        let oid = GitRemote::fetch_remote_ref_oid(remote, reference)?;
        env.print_vv("GitRemote", &format!("fetched oid : {}", &oid));
        let repo = Repository::open(&local_dir)?;
        let obj = match repo.revparse_single(&oid.to_string()) {
            Ok(v) => v,
            Err(_) => {
                env.print_vv("GitRemote", "oid not found locally");
                repo.remote_set_url("origin", remote)?;
                let mut origin = repo.find_remote("origin")?;
                origin.connect(Direction::Fetch)?;
                origin.fetch(&[] as &[&str], None, None)?;
                repo.revparse_single(&oid.to_string())?
            }
        };
        repo.set_head_detached(obj.id())?;
        repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
        env.print_vv("GitRemote", "template fetched");
        Ok(Self {
            remote: remote.to_string(),
            oid,
            local_dir,
        })
    }
}
