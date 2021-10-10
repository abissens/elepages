#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{FsLoader, Loader};
    use crate::stages::compose_stage::ComposeUnit::{CreateNewSet, ReplaceSubSet};
    use crate::stages::compose_stage::{ComposeStage, PrefixSelector};
    use crate::stages::copy_stage::CopyStage;
    use crate::stages::stage::Stage;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::borrow::Borrow;
    use std::sync::Arc;

    #[test]
    fn compose_stage_should_create_new_page_set() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "test content".to_string().into_bytes(),
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: vec![],
                    },
                ],
            })
            .unwrap();

        let bundle = FsLoader::new(test_folder.get_path().to_path_buf()).load().unwrap();

        let compose_stage = ComposeStage {
            units: vec![CreateNewSet(Arc::new(CopyStage { prefix: vec!["copied".to_string()] }))],
        };

        let result_bundle = compose_stage.process(bundle.borrow());

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn compose_stage_should_replace_sub_page_set() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "test content".to_string().into_bytes(),
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: vec![],
                    },
                    FileNode::Dir {
                        name: "d2".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "f3".to_string(),
                                content: "".to_string().into_bytes(),
                            },
                            FileNode::File {
                                name: "f4".to_string(),
                                content: vec![],
                            },
                        ],
                    },
                ],
            })
            .unwrap();

        let bundle = FsLoader::new(test_folder.get_path().to_path_buf()).load().unwrap();

        let compose_stage = ComposeStage {
            units: vec![ReplaceSubSet(
                Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                Arc::new(CopyStage { prefix: vec!["copied".to_string()] }),
            )],
        };

        let result_bundle = compose_stage.process(bundle.borrow());

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn compose_stage_should_create_and_replace_sub_page_set() {
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "f1".to_string(),
                        content: "test content".to_string().into_bytes(),
                    },
                    FileNode::File {
                        name: "f2".to_string(),
                        content: vec![],
                    },
                    FileNode::Dir {
                        name: "d2".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "f3".to_string(),
                                content: "".to_string().into_bytes(),
                            },
                            FileNode::File {
                                name: "f4".to_string(),
                                content: vec![],
                            },
                        ],
                    },
                ],
            })
            .unwrap();

        let bundle = FsLoader::new(test_folder.get_path().to_path_buf()).load().unwrap();

        let compose_stage = ComposeStage {
            units: vec![
                CreateNewSet(Arc::new(CopyStage {
                    prefix: vec!["backup".to_string(), "copied".to_string()],
                })),
                ReplaceSubSet(
                    Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                    Arc::new(CopyStage { prefix: vec!["copied".to_string()] }),
                ),
            ],
        };

        let result_bundle = compose_stage.process(bundle.borrow());

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }
}
