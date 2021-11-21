#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, Env, FsLoader, Loader, Metadata};
    use crate::stages::git_metadata::GitMetadata;
    use crate::stages::sequence_stage::SequenceStage;
    use crate::stages::shadow_pages::ShadowPages;
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestProcessingResult;
    use git2::{IndexAddOption, Repository};
    use indoc::indoc;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::array::IntoIter;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn ignore_not_initialized_repositories() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::File {
                name: "file_1".to_string(),
                content: "file content 1".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();

        let git_metadata_stage = GitMetadata {
            name: "git meta stage".to_string(),
            repo_path: test_folder.get_path().to_path_buf(),
        };

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let result_bundle = git_metadata_stage.process(&Arc::new(bundle), &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "git meta stage".to_string(),
                sub_results: vec![]
            }
        );

        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();

        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["file_1".to_string()],
                metadata: None,
                content: "file content 1".to_string()
            }]
        );
    }

    #[test]
    fn load_author_from_git_metadata() {
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
        let commit_time = commit(&repo, "Initial commit");

        let git_metadata_stage = GitMetadata {
            name: "git meta stage".to_string(),
            repo_path: test_folder.get_path().to_path_buf(),
        };

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let result_bundle = git_metadata_stage.process(&Arc::new(bundle), &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "git meta stage".to_string(),
                sub_results: vec![]
            }
        );

        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();

        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["file_1".to_string()],
                metadata: Some(Metadata {
                    title: None,
                    summary: None,
                    authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                        name: "user_1".to_string(),
                        contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                    })])),
                    tags: Default::default(),
                    publishing_date: None,
                    last_edit_date: commit_time,
                    data: HashMap::default(),
                }),
                content: "file content 1".to_string()
            }]
        );
    }

    #[test]
    fn load_multiple_authors_from_git_metadata() {
        let mut test_folder = TmpTestFolder::new().unwrap();
        test_folder.preserve();
        let repo = Repository::init(test_folder.get_path()).unwrap();
        repo.config().unwrap().set_str("user.name", "user_1").unwrap();
        repo.config().unwrap().set_str("user.email", "user_1@pages.io").unwrap();

        test_folder
            .write(&FileNode::File {
                name: "file_1".to_string(),
                content: "file content 1\n".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();
        commit(&repo, "Initial commit");

        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "file content 1".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: "file content 2".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f3".to_string(),
                        content: "file content 3".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "d11".to_string(),
                        sub: vec![FileNode::File {
                            name: "f11".to_string(),
                            content: "file content 11".as_bytes().to_vec(),
                            open_options: None,
                        }],
                    },
                ],
            })
            .unwrap();

        repo.config().unwrap().set_str("user.name", "user_2").unwrap();
        repo.config().unwrap().set_str("user.email", "user_2@pages.io").unwrap();

        let commit_time_2 = commit(&repo, "Second commit");

        test_folder
            .write(&FileNode::File {
                name: "file_1".to_string(),
                content: indoc! {"
                        file content 1
                        file content 2
                        file content 3
                    "}
                .as_bytes()
                .to_vec(),
                open_options: None,
            })
            .unwrap();

        repo.config().unwrap().set_str("user.name", "user_3").unwrap();
        repo.config().unwrap().set_str("user.email", "user_3@pages.io").unwrap();
        let commit_time_3 = commit(&repo, "Third commit");

        let git_metadata_stage = GitMetadata {
            name: "git meta stage".to_string(),
            repo_path: test_folder.get_path().to_path_buf(),
        };

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let result_bundle = git_metadata_stage.process(&Arc::new(bundle), &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "git meta stage".to_string(),
                sub_results: vec![]
            }
        );

        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();

        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "d11".to_string(), "f11".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 11".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 3".to_string(),
                },
                TestPage {
                    path: vec!["file_1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_3".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_3@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_3,
                        data: HashMap::default(),
                    }),
                    content: indoc! {"
                        file content 1
                        file content 2
                        file content 3
                    "}
                    .to_string()
                }
            ]
        );
    }

    #[test]
    fn ignore_uncommitted_files() {
        let mut test_folder = TmpTestFolder::new().unwrap();
        test_folder.preserve();
        let repo = Repository::init(test_folder.get_path()).unwrap();
        repo.config().unwrap().set_str("user.name", "user_1").unwrap();
        repo.config().unwrap().set_str("user.email", "user_1@pages.io").unwrap();

        test_folder
            .write(&FileNode::File {
                name: "file_1".to_string(),
                content: "file content 1\n".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();
        let commit_time_1 = commit(&repo, "Initial commit");

        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "file content 1".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: "file content 2".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f3".to_string(),
                        content: "file content 3".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "d11".to_string(),
                        sub: vec![FileNode::File {
                            name: "f11".to_string(),
                            content: "file content 11".as_bytes().to_vec(),
                            open_options: None,
                        }],
                    },
                ],
            })
            .unwrap();

        repo.config().unwrap().set_str("user.name", "user_2").unwrap();
        repo.config().unwrap().set_str("user.email", "user_2@pages.io").unwrap();

        let commit_time_2 = commit(&repo, "Second commit");

        test_folder
            .write(&FileNode::File {
                name: "file_1".to_string(),
                content: indoc! {"
                        file content 1
                        file content 2
                        file content 3
                    "}
                .as_bytes()
                .to_vec(),
                open_options: None,
            })
            .unwrap();

        test_folder
            .write(&FileNode::File {
                name: "file_xyz".to_string(),
                content: "some content xyz".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();

        let git_metadata_stage = GitMetadata {
            name: "git meta stage".to_string(),
            repo_path: test_folder.get_path().to_path_buf(),
        };

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let result_bundle = git_metadata_stage.process(&Arc::new(bundle), &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "git meta stage".to_string(),
                sub_results: vec![]
            }
        );

        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "d11".to_string(), "f11".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 11".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_2".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_2@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_2,
                        data: HashMap::default(),
                    }),
                    content: "file content 3".to_string(),
                },
                TestPage {
                    path: vec!["file_1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_1".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time_1,
                        data: HashMap::default(),
                    }),
                    content: indoc! {"
                        file content 1
                        file content 2
                        file content 3
                    "}
                    .to_string()
                },
                TestPage {
                    path: vec!["file_xyz".to_string()],
                    metadata: None,
                    content: "some content xyz".to_string()
                }
            ]
        );
    }

    #[test]
    fn ignore_loading_authors_when_file_has_already_ones() {
        let mut test_folder = TmpTestFolder::new().unwrap();
        test_folder.preserve();
        let repo = Repository::init(test_folder.get_path()).unwrap();
        repo.config().unwrap().set_str("user.name", "user_1").unwrap();
        repo.config().unwrap().set_str("user.email", "user_1@pages.io").unwrap();

        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "file content 1".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f1.yaml".to_string(),
                        content: indoc! {"
                            authors:
                              - name: a1
                            tags: [t1, t2]
                        "}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: "file content 2".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f2.yaml".to_string(),
                        content: indoc! {"
                            title: f2 title
                            summary: f2 summary
                            tags: [t1, t2, t3]
                        "}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f3".to_string(),
                        content: "file content 3".as_bytes().to_vec(),
                        open_options: None,
                    },
                ],
            })
            .unwrap();
        let commit_time = commit(&repo, "Initial commit");

        let sequence_stage = SequenceStage {
            name: "sequence stage".to_string(),
            stages: vec![
                Arc::new(ShadowPages::default("shadow stage".to_string())),
                Arc::new(GitMetadata {
                    name: "git meta stage".to_string(),
                    repo_path: test_folder.get_path().to_path_buf(),
                }),
            ],
        };

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let result_bundle = sequence_stage.process(&Arc::new(bundle), &Env::test()).unwrap();

        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "sequence stage".to_string(),
                sub_results: vec![
                    TestProcessingResult {
                        stage_name: "git meta stage".to_string(),
                        sub_results: vec![]
                    },
                    TestProcessingResult {
                        stage_name: "shadow stage".to_string(),
                        sub_results: vec![]
                    },
                ]
            }
        );

        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "a1".to_string(),
                            contacts: HashSet::default()
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: None,
                        last_edit_date: commit_time,
                        data: HashMap::default(),
                    }),
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f2 title".to_string())),
                        summary: Some(Arc::new("f2 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_1".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: None,
                        last_edit_date: commit_time,
                        data: HashMap::default(),
                    }),
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_1".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time,
                        data: HashMap::default(),
                    }),
                    content: "file content 3".to_string(),
                }
            ]
        );
    }

    #[test]
    fn ignore_loading_last_edit_date_when_file_has_already_ones() {
        let mut test_folder = TmpTestFolder::new().unwrap();
        test_folder.preserve();
        let repo = Repository::init(test_folder.get_path()).unwrap();
        repo.config().unwrap().set_str("user.name", "user_1").unwrap();
        repo.config().unwrap().set_str("user.email", "user_1@pages.io").unwrap();

        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "file content 1".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f1.yaml".to_string(),
                        content: indoc! {"
                            lastEditDate: 2021-10-20T17:00:00-08:00
                            tags: [t1, t2]
                            data:
                              some_key: some_value
                        "}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: "file content 2".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f2.yaml".to_string(),
                        content: indoc! {"
                            title: f2 title
                            summary: f2 summary
                            tags: [t1, t2, t3]
                        "}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "f3".to_string(),
                        content: "file content 3".as_bytes().to_vec(),
                        open_options: None,
                    },
                ],
            })
            .unwrap();
        let commit_time = commit(&repo, "Initial commit");

        let sequence_stage = SequenceStage {
            name: "sequence stage".to_string(),
            stages: vec![
                Arc::new(ShadowPages::default("shadow stage".to_string())),
                Arc::new(GitMetadata {
                    name: "git meta stage".to_string(),
                    repo_path: test_folder.get_path().to_path_buf(),
                }),
            ],
        };

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let result_bundle = sequence_stage.process(&Arc::new(bundle), &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "sequence stage".to_string(),
                sub_results: vec![
                    TestProcessingResult {
                        stage_name: "git meta stage".to_string(),
                        sub_results: vec![]
                    },
                    TestProcessingResult {
                        stage_name: "shadow stage".to_string(),
                        sub_results: vec![]
                    },
                ]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_1".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: None,
                        last_edit_date: Some(1634778000),
                        data: HashMap::from_iter(IntoIter::new([("some_key".to_string(), Value::String("some_value".to_string()))])),
                    }),
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f2 title".to_string())),
                        summary: Some(Arc::new("f2 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_1".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: None,
                        last_edit_date: commit_time,
                        data: HashMap::default(),
                    }),
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "user_1".to_string(),
                            contacts: HashSet::from_iter(IntoIter::new(["user_1@pages.io".to_string()])),
                        })])),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: commit_time,
                        data: HashMap::default(),
                    }),
                    content: "file content 3".to_string(),
                }
            ]
        );
    }

    fn commit(repo: &Repository, message: &str) -> Option<i64> {
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
                let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent]).unwrap();
                return Some(repo.find_commit(oid).unwrap().time().seconds());
            }
            panic!("cannot find target")
        }
        let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[]).unwrap();
        return Some(repo.find_commit(oid).unwrap().time().seconds());
    }
}
