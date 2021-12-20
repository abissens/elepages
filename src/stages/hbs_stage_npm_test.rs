#[cfg(test)]
mod tests {
    use crate::commands::NpmRunner;
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{BundleIndex, Env, Metadata, Page, PageBundle, VecBundle};
    use crate::pages_error::PagesError;
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::{HbsStage, PageGeneratorBagImpl, Stage};
    use indoc::indoc;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::array::IntoIter;
    use std::collections::HashMap;
    use std::fmt::{Debug, Formatter};
    use std::iter::FromIterator;
    use std::path::Path;
    use std::process::Command;
    use std::sync::Arc;

    #[test]
    fn should_ignore_package_json_when_no_build_script() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![Arc::new(TestPage {
                path: vec!["f1.html".to_string()],
                metadata: None,
                content: "content 1".to_string(),
            })],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.hbs".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "package.json".to_string(),
                        content: indoc! {r#"
                        {
                          "name": "sample",
                          "version": "1.0.0",
                          "main": "index.js",
                          "scripts": {
                            "test": "echo \"Error: no test specified\" && exit 1"
                          }
                        }
                    "#}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                ],
            })
            .unwrap();
        let hb_stage = HbsStage::new("hb stage".to_string(), test_folder.get_path().join("templates")).unwrap();
        let page_generator_bag = PageGeneratorBagImpl::new();
        let result_bundle = hb_stage.process(&bundle, &Env::test(), &page_generator_bag).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );

        let bundle_index = BundleIndex::from(&result_bundle.0);
        let generated: Vec<Arc<dyn Page>> = page_generator_bag.all().unwrap().iter().flat_map(|g| g.yield_pages(&bundle_index, &Env::test()).unwrap()).collect();

        let mut actual_generated = generated.iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual_generated.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual_generated,
            &[TestPage {
                path: vec!["package.json".to_string()],
                metadata: default_metadata(),
                content: indoc! {r#"
                        {
                          "name": "sample",
                          "version": "1.0.0",
                          "main": "index.js",
                          "scripts": {
                            "test": "echo \"Error: no test specified\" && exit 1"
                          }
                        }
                    "#}
                .to_string(),
            },]
        );

        let mut actual_pages = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual_pages.sort_by_key(|f| f.path.join("/"));

        assert_eq!(
            actual_pages,
            &[TestPage {
                path: vec!["f1.html".to_string()],
                metadata: None,
                content: "TPL root :  \n content 1".to_string(),
            },]
        );
    }

    #[test]
    fn launches_build_script_and_use_built_folder_as_template_root() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![Arc::new(TestPage {
                path: vec!["f1.html".to_string()],
                metadata: None,
                content: "content 1".to_string(),
            })],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.src".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "package.json".to_string(),
                        content: indoc! {r#"
                        {
                          "name": "sample",
                          "version": "1.0.0",
                          "main": "index.js",
                          "scripts": {
                            "build": "rm -rf output && mkdir output && cp ./page.src ./output/page.hbs"
                          },
                          "buildOutputDir": "output"
                        }
                    "#}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                ],
            })
            .unwrap();

        let template_folder = test_folder.get_path().join("templates");
        let fake_runner = FakeNpmRunner::new(Arc::new(move |script: &str| {
            assert_eq!(script, "build");
            Command::new("mkdir").arg("output").current_dir(&template_folder).output()?;
            Command::new("cp").arg("./page.src").arg("./output/page.hbs").current_dir(&template_folder).output()?;
            Ok(())
        }));

        let hb_stage = HbsStage::new_with_npm_runner("hb stage".to_string(), test_folder.get_path().join("templates"), fake_runner);

        let result_bundle = hb_stage.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));

        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["f1.html".to_string()],
                metadata: None,
                content: "TPL root :  \n content 1".to_string(),
            },]
        );
    }

    #[test]
    fn launches_build_script_fail_when_output_folder_not_found() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle { p: vec![] });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.src".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "package.json".to_string(),
                        content: indoc! {r#"
                        {
                          "name": "sample",
                          "version": "1.0.0",
                          "main": "index.js",
                          "scripts": {
                            "build": "rm -rf output && mkdir output && cp ./page.src ./output/page.hbs"
                          },
                          "buildOutputDir": "output"
                        }
                    "#}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                ],
            })
            .unwrap();

        let hb_stage = HbsStage::new_with_npm_runner("hb stage".to_string(), test_folder.get_path().join("templates"), FakeNpmRunner::noop());

        let result_bundle = hb_stage.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new());
        if let Err(err) = result_bundle {
            if let PagesError::Exec(page_error) = err.downcast_ref::<PagesError>().unwrap() {
                assert_eq!(
                    page_error,
                    format!("build folder {} not found", test_folder.get_path().join("templates").join("output").to_string_lossy()).as_str()
                )
            } else {
                panic!("should raise an error");
            }
        } else {
            panic!("should raise an error");
        }
    }

    #[test]
    fn launches_build_script_fail_when_output_folder_is_not_a_directory() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle { p: vec![] });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.src".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "package.json".to_string(),
                        content: indoc! {r#"
                        {
                          "name": "sample",
                          "version": "1.0.0",
                          "main": "index.js",
                          "scripts": {
                            "build": "rm -rf output && mkdir output && cp ./page.src ./output/page.hbs"
                          },
                          "buildOutputDir": "output"
                        }
                    "#}
                        .as_bytes()
                        .to_vec(),
                        open_options: None,
                    },
                ],
            })
            .unwrap();
        let template_folder = test_folder.get_path().join("templates");
        let fake_runner = FakeNpmRunner::new(Arc::new(move |script: &str| {
            assert_eq!(script, "build");
            Command::new("touch").arg("output").current_dir(&template_folder).output()?;
            Ok(())
        }));

        let hb_stage = HbsStage::new_with_npm_runner("hb stage".to_string(), test_folder.get_path().join("templates"), fake_runner);

        let result_bundle = hb_stage.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new());
        if let Err(err) = result_bundle {
            if let PagesError::Exec(page_error) = err.downcast_ref::<PagesError>().unwrap() {
                assert_eq!(
                    page_error,
                    format!("build result {} is not a dir", test_folder.get_path().join("templates").join("output").to_string_lossy()).as_str()
                );
            } else {
                panic!("should raise an error");
            }
        } else {
            panic!("should raise an error");
        }
    }

    fn default_metadata() -> Option<Metadata> {
        Some(Metadata {
            title: None,
            summary: None,
            authors: Default::default(),
            tags: Default::default(),
            publishing_date: None,
            last_edit_date: None,
            data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))])),
        })
    }

    struct FakeNpmRunner {
        run_cmd: Arc<dyn Fn(&str) -> anyhow::Result<()> + Send + Sync>,
    }

    impl FakeNpmRunner {
        fn noop() -> Box<dyn NpmRunner> {
            Box::new(FakeNpmRunner { run_cmd: Arc::new(|_: &str| Ok(())) })
        }

        fn new(run_cmd: Arc<dyn Fn(&str) -> anyhow::Result<()> + Send + Sync>) -> Box<dyn NpmRunner> {
            Box::new(FakeNpmRunner { run_cmd })
        }
    }

    impl Debug for FakeNpmRunner {
        fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
            panic!("not implemented");
        }
    }

    impl NpmRunner for FakeNpmRunner {
        fn install(&self, _: &Path, _: &Env) -> anyhow::Result<()> {
            Ok(())
        }

        fn run(&self, _: &Path, script: &str, _: &Env) -> anyhow::Result<()> {
            (self.run_cmd)(script)
        }
    }
}
