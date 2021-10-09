#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{FsLoader, Loader};
    use crate::stages::copy_stage::CopyStage;
    use crate::stages::stage::Stage;
    use crate::stages::union_stage::UnionStage;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::borrow::Borrow;

    #[test]
    fn copy_stage_should_copy_all_bundle_paths_to_another_root_path() {
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

        let copy_stage_1 = CopyStage {
            prefix: vec!["root".to_string(), "sub_root".to_string()],
        };

        let copy_stage_2 = CopyStage {
            prefix: vec!["second_root".to_string()],
        };

        let union_stage = UnionStage {
            stages: &[&copy_stage_1, &copy_stage_2],
        };

        let result_bundle = union_stage.process(bundle.borrow());

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["root".to_string(), "sub_root".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["root".to_string(), "sub_root".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["second_root".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["second_root".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }
}
