#[cfg(test)]
mod tests {
    use crate::pages::fs_loader::FsLoader;
    use crate::pages::{Loader, Metadata, Page};
    use rustassert::assert;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::io::Read;

    #[test]
    fn should_load_from_empty_folder() {
        let test_folder = TmpTestFolder::new().unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load().unwrap();

        assert_eq!(bundle.pages().len(), 0);
    }

    #[test]
    fn should_load_from_single_file() {
        let test_folder = TmpTestFolder::new_from_node(&FileNode::new_file("file", "this is a test file".as_bytes().to_vec())).unwrap();
        let file_path = test_folder.get_path().join("file");

        let loader = FsLoader::new(file_path.to_path_buf());
        let bundle = loader.load().unwrap();

        assert_eq!(bundle.pages().len(), 1);
        assert::that(bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>()).eq_each(&[TestPage {
            path: vec![],
            metadata: None,
            content: "this is a test file".to_string(),
        }]);
    }

    #[test]
    fn should_load_from_folder_containing_single_file() {
        let test_folder = TmpTestFolder::new_from_node(&FileNode::new_file("file", "this is a test file".as_bytes().to_vec())).unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load().unwrap();

        assert_eq!(bundle.pages().len(), 1);
        assert::that(bundle.pages().iter().collect::<Vec<_>>()).map(|p| TestPage::from(*p)).eq_each(&[TestPage {
            path: vec!["file".to_string()],
            metadata: None,
            content: "this is a test file".to_string(),
        }]);
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
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: "file content 2".as_bytes().to_vec(),
                    },
                    FileNode::File {
                        name: "f3".to_string(),
                        content: "file content 3".as_bytes().to_vec(),
                    },
                ],
            })
            .unwrap();

        let loader = FsLoader::new(test_folder.get_path().join("d1").to_path_buf());
        let bundle = loader.load().unwrap();

        let mut actual = bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert::that(actual).eq_each(&[
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
        ]);
    }

    #[test]
    fn should_load_from_folder_containing_folder_hierarchy() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::File {
                name: "fr1".to_string(),
                content: Vec::new(),
            })
            .unwrap();

        test_folder
            .write(&FileNode::File {
                name: "fr2".to_string(),
                content: "file content fr2".as_bytes().to_vec(),
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
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: "file content 2".as_bytes().to_vec(),
                    },
                    FileNode::File {
                        name: "f3".to_string(),
                        content: "file content 3".as_bytes().to_vec(),
                    },
                    FileNode::Dir {
                        name: "d11".to_string(),
                        sub: vec![FileNode::File {
                            name: "f11".to_string(),
                            content: "file content 11".as_bytes().to_vec(),
                        }],
                    },
                ],
            })
            .unwrap();

        let loader = FsLoader::new(test_folder.get_path().to_path_buf());
        let bundle = loader.load().unwrap();

        let mut actual = bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert::that(actual).eq_each(&[
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
        ]);
    }

    #[derive(PartialOrd, PartialEq)]
    struct TestPage {
        path: Vec<String>,
        metadata: Option<Metadata>,
        content: String,
    }

    impl From<&Box<dyn Page>> for TestPage {
        fn from(p: &Box<dyn Page>) -> Self {
            let mut content: String = "".to_string();
            p.open().unwrap().read_to_string(&mut content).unwrap();
            TestPage {
                path: p.path().to_vec(),
                metadata: p.metadata().map(|m| m.clone()),
                content,
            }
        }
    }
}
