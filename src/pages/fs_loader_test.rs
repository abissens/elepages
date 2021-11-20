#[cfg(test)]
mod tests {
    use crate::pages::fs_loader::FsLoader;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Env, Loader};
    use rustassert::fs::{FileNode, TmpTestFolder};

    #[test]
    fn should_load_from_empty_folder() {
        let test_folder = TmpTestFolder::new().unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        assert_eq!(bundle.pages().len(), 0);
    }

    #[test]
    fn should_load_from_single_file() {
        let test_folder = TmpTestFolder::new_from_node(&FileNode::new_file("file", "this is a test file".as_bytes().to_vec())).unwrap();
        let file_path = test_folder.get_path().join("file");

        let loader = FsLoader::new(file_path.to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        assert_eq!(bundle.pages().len(), 1);
        assert_eq!(
            bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>(),
            &[TestPage {
                path: vec![],
                metadata: None,
                content: "this is a test file".to_string(),
            }]
        );
    }

    #[test]
    fn should_load_from_folder_containing_single_file() {
        let test_folder = TmpTestFolder::new_from_node(&FileNode::new_file("file", "this is a test file".as_bytes().to_vec())).unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        assert_eq!(bundle.pages().len(), 1);
        assert_eq!(
            bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>(),
            &[TestPage {
                path: vec!["file".to_string()],
                metadata: None,
                content: "this is a test file".to_string(),
            }]
        );
    }

    #[test]
    fn should_load_from_folder_containing_multiple_files() {
        let test_folder = TmpTestFolder::new().unwrap();
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
                ],
            })
            .unwrap();

        let loader = FsLoader::new(test_folder.get_path().join("d1").to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let mut actual = bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["f1".to_string()],
                    metadata: None,
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2".to_string()],
                    metadata: None,
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["f3".to_string()],
                    metadata: None,
                    content: "file content 3".to_string(),
                },
            ]
        );
    }

    #[test]
    fn should_load_from_folder_containing_folder_hierarchy() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::File {
                name: "fr1".to_string(),
                content: Vec::new(),
                open_options: None,
            })
            .unwrap();

        test_folder
            .write(&FileNode::File {
                name: "fr2".to_string(),
                content: "file content fr2".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();

        test_folder
            .write(&FileNode::Dir {
                name: "empty_dir".to_string(),
                sub: vec![],
            })
            .unwrap();

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

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let mut actual = bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "d11".to_string(), "f11".to_string()],
                    metadata: None,
                    content: "file content 11".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "file content 3".to_string(),
                },
                TestPage {
                    path: vec!["fr1".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["fr2".to_string()],
                    metadata: None,
                    content: "file content fr2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn should_ignore_hidden_entries() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::File {
                name: "fr1".to_string(),
                content: Vec::new(),
                open_options: None,
            })
            .unwrap();

        test_folder
            .write(&FileNode::File {
                name: ".fr2".to_string(),
                content: "file content fr2".as_bytes().to_vec(),
                open_options: None,
            })
            .unwrap();

        test_folder
            .write(&FileNode::Dir {
                name: "empty_dir".to_string(),
                sub: vec![],
            })
            .unwrap();

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
                        name: ".d11".to_string(),
                        sub: vec![FileNode::File {
                            name: "f11".to_string(),
                            content: "file content 11".as_bytes().to_vec(),
                            open_options: None,
                        }],
                    },
                ],
            })
            .unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load(&Env::test()).unwrap();

        let mut actual = bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "file content 1".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "file content 2".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "file content 3".to_string(),
                },
                TestPage {
                    path: vec!["fr1".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }
}
