#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, Env, Metadata, PageBundle, VecBundle};
    use crate::stages::shadow_pages::ShadowPages;
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::PageGeneratorBagImpl;
    use indoc::indoc;
    use std::array::IntoIter;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn shadow_pages_stage_should_load_page_metadata() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a".to_string()],
                    metadata: None,
                    content: "'a' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a.json".to_string()],
                    metadata: None,
                    content: r#"{"title": "a title"}"#.to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default("shadow stage".to_string());

        let result_bundle = shadow_stage.process(&vec_bundle, &mut Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "shadow stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["a".to_string()],
                metadata: Some(Metadata {
                    title: Some(Arc::new("a title".to_string())),
                    summary: None,
                    authors: Default::default(),
                    tags: Default::default(),
                    publishing_date: None,
                    last_edit_date: None,
                    data: HashMap::default(),
                }),
                content: "'a' content".to_string()
            },]
        );
    }

    #[test]
    fn shadow_pages_stage_should_load_multiple_page_metadata() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a".to_string()],
                    metadata: None,
                    content: "'a' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a.json".to_string()],
                    metadata: None,
                    content: r#"{"title": "a title"}"#.to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["b".to_string()],
                    metadata: None,
                    content: "'b' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["b.json".to_string()],
                    metadata: None,
                    content: r#"{
                        "title": "b title",
                        "summary": "b summary",
                        "authors": [{"name": "a1"}, {"name": "a2", "contacts": ["c1", "c2"]}],
                        "tags": ["t1", "t2", "t3"],
                        "publishingDate": "2021-10-20T16:00:00-08:00",
                        "lastEditDate": "2021-10-20T17:00:00-08:00",
                        "data": {"a": 10}
                    }"#
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["c".to_string()],
                    metadata: None,
                    content: "'c' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["c.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        title: c title
                        summary: c summary
                        authors:
                          - name: a1
                          - name: a2
                            contacts: [c1, c2]
                        tags: [t1, t2, t3]
                        publishingDate: 2021-10-20T16:00:00
                        lastEditDate: 2021-10-20
                        data:
                          a: 10
                    "}
                    .to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default("shadow stage".to_string());

        let result_bundle = shadow_stage.process(&vec_bundle, &mut Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "shadow stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("a title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'a' content".to_string()
                },
                TestPage {
                    path: vec!["b".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("b title".to_string())),
                        summary: Some(Arc::new("b summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([
                            Arc::new(Author {
                                name: "a1".to_string(),
                                contacts: HashSet::default(),
                            }),
                            Arc::new(Author {
                                name: "a2".to_string(),
                                contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                            })
                        ])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: Some(1634778000),
                        data: HashMap::from_iter(IntoIter::new([("a".to_string(), Value::I32(10))])),
                    }),
                    content: "'b' content".to_string()
                },
                TestPage {
                    path: vec!["c".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("c title".to_string())),
                        summary: Some(Arc::new("c summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([
                            Arc::new(Author {
                                name: "a1".to_string(),
                                contacts: HashSet::default(),
                            }),
                            Arc::new(Author {
                                name: "a2".to_string(),
                                contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                            })
                        ])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: Some(1634745600),
                        last_edit_date: Some(1634688000),
                        data: HashMap::from_iter(IntoIter::new([("a".to_string(), Value::I32(10))])),
                    }),
                    content: "'c' content".to_string()
                },
            ]
        );
    }

    #[test]
    fn shadow_pages_stage_should_load_nothing_when_no_metadata() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: None,
                    content: "'c' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: None,
                    content: "'d' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: None,
                    content: "'e' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default("shadow stage".to_string());

        let result_bundle = shadow_stage.process(&vec_bundle, &mut Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "shadow stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: None,
                    content: "'c' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: None,
                    content: "'d' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: None,
                    content: "'e' content".to_string()
                },
                TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string()
                },
                TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string()
                },
            ]
        );
    }

    #[test]
    fn shadow_pages_stage_should_load_root_hierarchy_metadata() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        authors:
                          - name: a1
                            contacts: [c1, c2]
                          - name: a2
                            contacts: [c3, c4]
                        tags: [t1, t2]
                        publishingDate: 2021-10-20T16:00:00-08:00
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: None,
                    content: "'c' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: None,
                    content: "'d' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: None,
                    content: "'e' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default("shadow stage".to_string());

        let result_bundle = shadow_stage.process(&vec_bundle, &mut Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "shadow stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'c' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'d' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'e' content".to_string()
                },
                TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string()
                },
                TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string()
                },
            ]
        );
    }

    #[test]
    fn shadow_pages_stage_should_load_multi_level_hierarchy_metadata() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        authors:
                          - name: a1
                            contacts: [c1, c2]
                          - name: a2
                            contacts: [c3, c4]
                        tags: [t1, t2]
                        publishingDate: 2021-10-20T16:00:00-08:00
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: None,
                    content: "'c' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        title: c title
                        summary: c summary
                        authors:
                          - name: a2
                        tags: [t2, t3]
                        publishingDate: 2021-10-20T17:00:00-08:00
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: None,
                    content: "'d' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: None,
                    content: "'e' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default("shadow stage".to_string());

        let result_bundle = shadow_stage.process(&vec_bundle, &mut Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "shadow stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("c title".to_string())),
                        summary: Some(Arc::new("c summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "a2".to_string(),
                            contacts: vec!["c3", "c4"].iter().map(|x| x.to_string()).collect(),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: Some(1634778000),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'c' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'d' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "'e' content".to_string()
                },
                TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string()
                },
                TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string()
                },
            ]
        );
    }

    #[test]
    fn shadow_pages_stage_should_load_root_metadata() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["pages.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        page:
                          tags: [root_tag]
                          data:
                            some_root_key: some_value
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        authors:
                          - name: a1
                            contacts: [c1, c2]
                          - name: a2
                            contacts: [c3, c4]
                        tags: [t1, t2]
                        publishingDate: 2021-10-20T16:00:00-08:00
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: None,
                    content: "'c' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt.yaml".to_string()],
                    metadata: None,
                    content: indoc! {"
                        title: c title
                        summary: c summary
                        authors:
                          - name: a2
                        tags: [t2, t3]
                        publishingDate: 2021-10-20T17:00:00-08:00
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: None,
                    content: "'d' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: None,
                    content: "'e' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: None,
                    content: "'f' content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: None,
                    content: "'i' content".to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default("shadow stage".to_string());

        let result_bundle = shadow_stage.process(&vec_bundle, &mut Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "shadow stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("c title".to_string())),
                        summary: Some(Arc::new("c summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "a2".to_string(),
                            contacts: vec!["c3", "c4"].iter().map(|x| x.to_string()).collect(),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([
                            Arc::new("root_tag".to_string()),
                            Arc::new("t1".to_string()),
                            Arc::new("t2".to_string()),
                            Arc::new("t3".to_string())
                        ])),
                        publishing_date: Some(1634778000),
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("some_root_key".to_string(), Value::String("some_value".to_string()))])),
                    }),
                    content: "'c' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("root_tag".to_string()), Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("some_root_key".to_string(), Value::String("some_value".to_string()))])),
                    }),
                    content: "'d' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("root_tag".to_string()), Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(1634774400),
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("some_root_key".to_string(), Value::String("some_value".to_string()))])),
                    }),
                    content: "'e' content".to_string()
                },
                TestPage {
                    path: vec!["f.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("root_tag".to_string())])),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("some_root_key".to_string(), Value::String("some_value".to_string()))])),
                    }),
                    content: "'f' content".to_string()
                },
                TestPage {
                    path: vec!["g".to_string(), "h".to_string(), "i.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("root_tag".to_string())])),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("some_root_key".to_string(), Value::String("some_value".to_string()))])),
                    }),
                    content: "'i' content".to_string()
                },
            ]
        );
    }
}
