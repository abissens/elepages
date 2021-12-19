#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Env, Metadata, PageBundle, VecBundle};
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::{PageGeneratorBagImpl, PathGenerator, Stage};
    use std::array::IntoIter;
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn generate_page_paths() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("a/b/c".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(1637671914),
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("page/{{short_year}}/{{short_month}}/{{day}}".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f3".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("F3 Title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{url_title}}".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f4".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: Default::default(),
                    }),
                    content: "test content".to_string(),
                }),
            ],
        });

        let path_generator = PathGenerator::new("path generator".to_string());

        let result_bundle = path_generator.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "path generator".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("a/b/c".to_string())),])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f4".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: Default::default(),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["f3_title".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("F3 Title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{url_title}}".to_string())),])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["page".to_string(), "21".to_string(), "Nov".to_string(), "23".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(1637671914),
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("page/{{short_year}}/{{short_month}}/{{day}}".to_string()),),])),
                    }),
                    content: "test content".to_string(),
                },
            ]
        );
    }

    #[test]
    fn generate_page_paths_using_path_and_rev_path_elements() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path.0}}/other".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("other/{{rev_path.0}}".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
            ],
        });

        let path_generator = PathGenerator::new("path generator".to_string());

        let result_bundle = path_generator.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "path generator".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "other".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path.0}}/other".to_string()))])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["other".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("other/{{rev_path.0}}".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }
            ]
        );
    }

    #[test]
    fn generate_page_paths_using_path_join_helper() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path_join path}}/index.html".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path_join path 0 -2}}/index.html".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path_join path 0 -2}}/{{file_name}}/index.html".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }),
            ],
        });

        let path_generator = PathGenerator::new("path generator".to_string());

        let result_bundle = path_generator.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "path generator".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string(), "index.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path_join path 0 -2}}/{{file_name}}/index.html".to_string()))])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string(), "index.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path_join path}}/index.html".to_string()))])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "index.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("path".to_string(), Value::String("{{path_join path 0 -2}}/index.html".to_string()))])),
                    }),
                    content: "test content".to_string(),
                }
            ]
        );
    }

    #[test]
    fn generate_page_paths_based_on_regexp_pattern() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([(
                            "path".to_string(),
                            Value::Vec(vec![Value::String("^d1/d2".to_string()), Value::String("{{path_join path}}/index.html".to_string())]),
                        )])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([(
                            "path".to_string(),
                            Value::Vec(vec![
                                Value::Vec(vec![Value::String("^d1/d2".to_string()), Value::String("{{path_join path 0 -2}}/index_1.html".to_string())]),
                                Value::Vec(vec![Value::String(r"f\d{1}$".to_string()), Value::String("{{path_join path 0 -2}}/index_2.html".to_string())]),
                                Value::Vec(vec![Value::String("^d4".to_string()), Value::String("{{path_join path 0 -2}}/index_3.html".to_string())]),
                            ]),
                        )])),
                    }),
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([(
                            "path".to_string(),
                            Value::Vec(vec![Value::String("^d1/d2".to_string()), Value::String("{{path_join path 0 -2}}/{{file_name}}/index.html".to_string())]),
                        )])),
                    }),
                    content: "test content".to_string(),
                }),
            ],
        });

        let path_generator = PathGenerator::new("path generator".to_string());

        let result_bundle = path_generator.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "path generator".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string(), "index.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([(
                            "path".to_string(),
                            Value::Vec(vec![Value::String("^d1/d2".to_string()), Value::String("{{path_join path 0 -2}}/{{file_name}}/index.html".to_string())])
                        )])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([(
                            "path".to_string(),
                            Value::Vec(vec![Value::String("^d1/d2".to_string()), Value::String("{{path_join path}}/index.html".to_string())])
                        )])),
                    }),
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "index_2.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([(
                            "path".to_string(),
                            Value::Vec(vec![
                                Value::Vec(vec![Value::String("^d1/d2".to_string()), Value::String("{{path_join path 0 -2}}/index_1.html".to_string())]),
                                Value::Vec(vec![Value::String(r"f\d{1}$".to_string()), Value::String("{{path_join path 0 -2}}/index_2.html".to_string())]),
                                Value::Vec(vec![Value::String("^d4".to_string()), Value::String("{{path_join path 0 -2}}/index_3.html".to_string())]),
                            ]),
                        )])),
                    }),
                    content: "test content".to_string(),
                }
            ]
        );
    }

    #[test]
    fn generate_page_returns_err_when_malformed_regexp() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![Arc::new(TestPage {
                path: vec!["d1".to_string(), "f1".to_string()],
                metadata: Some(Metadata {
                    title: None,
                    summary: None,
                    authors: Default::default(),
                    tags: Default::default(),
                    publishing_date: None,
                    last_edit_date: None,
                    data: HashMap::from_iter(IntoIter::new([(
                        "path".to_string(),
                        Value::Vec(vec![Value::String("^[a".to_string()), Value::String("{{path_join path}}/index.html".to_string())]),
                    )])),
                }),
                content: "test content".to_string(),
            })],
        });

        let path_generator = PathGenerator::new("path generator".to_string());

        let result_bundle = path_generator.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new());
        if let Err(err) = result_bundle {
            if let Some(regex::Error::Syntax(s)) = err.downcast_ref::<regex::Error>() {
                assert_eq!(s, "regex parse error:\n    ^[a\n     ^\nerror: unclosed character class");
            }
        } else {
            panic!("should rise regexp parsing error")
        }
    }
}
