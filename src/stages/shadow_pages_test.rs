#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, Metadata, PageBundle, VecBundle};
    use crate::stages::shadow_pages::ShadowPages;
    use crate::stages::stage::Stage;
    use indoc::indoc;
    use std::array::IntoIter;
    use std::collections::HashSet;
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
        let shadow_stage = ShadowPages::default();

        let result_bundle = shadow_stage.process(&vec_bundle);

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["a".to_string()],
                metadata: Some(Metadata {
                    title: Some("a title".to_string()),
                    summary: None,
                    authors: Default::default(),
                    tags: Default::default()
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
                        "tags": ["t1", "t2", "t3"]
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
                    "}
                    .to_string(),
                }),
            ],
        });
        let shadow_stage = ShadowPages::default();

        let result_bundle = shadow_stage.process(&vec_bundle);

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string()],
                    metadata: Some(Metadata {
                        title: Some("a title".to_string()),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default()
                    }),
                    content: "'a' content".to_string()
                },
                TestPage {
                    path: vec!["b".to_string()],
                    metadata: Some(Metadata {
                        title: Some("b title".to_string()),
                        summary: Some("b summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([
                            Author {
                                name: "a1".to_string(),
                                contacts: HashSet::default(),
                            },
                            Author {
                                name: "a2".to_string(),
                                contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                            }
                        ])),
                        tags: vec!["t1", "t2", "t3"].iter().map(|x| x.to_string()).collect()
                    }),
                    content: "'b' content".to_string()
                },
                TestPage {
                    path: vec!["c".to_string()],
                    metadata: Some(Metadata {
                        title: Some("c title".to_string()),
                        summary: Some("c summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([
                            Author {
                                name: "a1".to_string(),
                                contacts: HashSet::default(),
                            },
                            Author {
                                name: "a2".to_string(),
                                contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                            }
                        ])),
                        tags: vec!["t1", "t2", "t3"].iter().map(|x| x.to_string()).collect()
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
        let shadow_stage = ShadowPages::default();

        let result_bundle = shadow_stage.process(&vec_bundle);

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
        let shadow_stage = ShadowPages::default();

        let result_bundle = shadow_stage.process(&vec_bundle);

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                        tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
                    }),
                    content: "'c' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
                    }),
                    content: "'d' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
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
        let shadow_stage = ShadowPages::default();

        let result_bundle = shadow_stage.process(&vec_bundle);

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string(), "b".to_string(), "c.txt".to_string()],
                    metadata: Some(Metadata {
                        title: Some("c title".to_string()),
                        summary: Some("c summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([Author {
                            name: "a2".to_string(),
                            contacts: vec!["c3", "c4"].iter().map(|x| x.to_string()).collect(),
                        }])),
                        tags: vec!["t1", "t2", "t3"].iter().map(|x| x.to_string()).collect(),
                    }),
                    content: "'c' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "d.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
                    }),
                    content: "'d' content".to_string()
                },
                TestPage {
                    path: vec!["a".to_string(), "e.txt".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
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
}
