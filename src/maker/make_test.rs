#[cfg(test)]
mod tests {
    use crate::maker::config::{StageValue, ValueConfig};
    use crate::maker::Maker;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, FsLoader, Loader, Metadata};
    use git2::{IndexAddOption, Repository};
    use indoc::indoc;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::array::IntoIter;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn build_git_authors_stage_based_on_named_config() {
        let mut test_folder = TmpTestFolder::new().unwrap();
        test_folder.preserve();
        let repo = Repository::init(test_folder.get_path()).unwrap();
        repo.config().unwrap().set_str("user.name", "user_1").unwrap();
        repo.config().unwrap().set_str("user.email", "user_1@pages.io").unwrap();

        test_folder
            .write(&FileNode::File {
                name: "file_1".to_string(),
                content: "file content 1".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();
        commit(&repo, "Initial commit");

        let git_authors_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            git_authors
        "})
        .unwrap();

        let env = &HashMap::from_iter(IntoIter::new([("root_path".to_string(), ValueConfig::String(test_folder.get_path().to_string_lossy().to_string()))]));
        let git_authors_stage = Maker::default().make(&git_authors_stage_config, env).unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load().unwrap();
        let result_bundle = git_authors_stage.process(&Arc::new(bundle)).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["file_1".to_string()],
                metadata: Some(Metadata {
                    title: None,
                    summary: None,
                    authors: HashSet::from_iter(IntoIter::new([Author {
                        name: "user_1".to_string(),
                        contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                    }])),
                    tags: Default::default()
                }),
                content: "file content 1".to_string()
            }]
        );
    }

    fn commit(repo: &Repository, message: &str) {
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None).unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        if let Ok(h) = repo.head() {
            if let Some(t) = h.target() {
                let parent = repo.find_commit(t).unwrap();
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent]).unwrap();
            }
            return;
        }
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[]).unwrap();
    }
}
