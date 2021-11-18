#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, BundleIndex, Metadata, MetadataIndex, PageBundle, PageIndex, PageRef, VecBundle};
    use std::array::IntoIter;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn generate_index_pages_from_single_page_bundle() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![Arc::new(TestPage {
                path: vec!["dir".to_string(), "f1".to_string()],
                metadata: Some(Metadata {
                    title: Some(Arc::new("f1 title".to_string())),
                    summary: Some(Arc::new("f1 summary".to_string())),
                    authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                        name: "f1 author".to_string(),
                        contacts: Default::default(),
                    })])),
                    tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                    publishing_date: None,
                    last_edit_date: None,
                }),
                content: String::new(),
            })],
        });

        let bundle_index = BundleIndex::from(&vec_bundle);

        assert_eq!(
            bundle_index,
            BundleIndex {
                all_pages: vec![PageIndex {
                    page_ref: PageRef {
                        path: vec!["dir".to_string(), "f1".to_string()]
                    },
                    metadata: Some(MetadataIndex {
                        title: Some("f1 title".to_string()),
                        summary: Some("f1 summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new(["f1 author".to_string()])),
                        tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                        publishing_date: None,
                        last_edit_date: None,
                    })
                }],
                all_authors: HashSet::from_iter(IntoIter::new([Author {
                    name: "f1 author".to_string(),
                    contacts: Default::default()
                }])),
                all_tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                pages_by_author: HashMap::from_iter(IntoIter::new([(
                    "f1 author".to_string(),
                    vec![PageRef {
                        path: vec!["dir".to_string(), "f1".to_string()]
                    }]
                )])),
                pages_by_tag: HashMap::from_iter(IntoIter::new([
                    (
                        "t1".to_string(),
                        vec![PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]
                    ),
                    (
                        "t2".to_string(),
                        vec![PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]
                    ),
                    (
                        "t3".to_string(),
                        vec![PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]
                    ),
                ])),
            }
        );
    }

    #[test]
    fn generate_index_pages_from_multi_page_bundle() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: Some(Arc::new("f1 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "f1 author".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: None,
                        last_edit_date: None,
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2".to_string()],
                    metadata: None,
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f3".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f3 title".to_string())),
                        summary: Some(Arc::new("f3 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([
                            Arc::new(Author {
                                name: "f3 author 1".to_string(),
                                contacts: Default::default(),
                            }),
                            Arc::new(Author {
                                name: "f3 author 2".to_string(),
                                contacts: Default::default(),
                            }),
                        ])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t3".to_string()), Arc::new("t4".to_string())])),
                        publishing_date: None,
                        last_edit_date: None,
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: Some(Arc::new("f4 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "f3 author 1".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::default(),
                        publishing_date: None,
                        last_edit_date: None,
                    }),
                    content: String::new(),
                }),
            ],
        });

        let bundle_index = BundleIndex::from(&vec_bundle);

        assert_eq!(
            bundle_index,
            BundleIndex {
                all_pages: vec![
                    PageIndex {
                        page_ref: PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        },
                        metadata: Some(MetadataIndex {
                            title: Some("f1 title".to_string()),
                            summary: Some("f1 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f1 author".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                        })
                    },
                    PageIndex {
                        page_ref: PageRef { path: vec!["f2".to_string()] },
                        metadata: None
                    },
                    PageIndex {
                        page_ref: PageRef { path: vec!["f3".to_string()] },
                        metadata: Some(MetadataIndex {
                            title: Some("f3 title".to_string()),
                            summary: Some("f3 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f3 author 1".to_string(), "f3 author 2".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t3".to_string(), "t4".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                        })
                    },
                    PageIndex {
                        page_ref: PageRef { path: vec!["f4".to_string()] },
                        metadata: Some(MetadataIndex {
                            title: Some("f4 title".to_string()),
                            summary: Some("f4 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f3 author 1".to_string()])),
                            tags: HashSet::default(),
                            publishing_date: None,
                            last_edit_date: None,
                        })
                    }
                ],

                pages_by_tag: HashMap::from_iter(IntoIter::new([
                    (
                        "t1".to_string(),
                        vec![PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]
                    ),
                    (
                        "t2".to_string(),
                        vec![PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]
                    ),
                    (
                        "t3".to_string(),
                        vec![
                            PageRef {
                                path: vec!["dir".to_string(), "f1".to_string()]
                            },
                            PageRef { path: vec!["f3".to_string()] }
                        ]
                    ),
                    ("t4".to_string(), vec![PageRef { path: vec!["f3".to_string()] }]),
                ])),
                pages_by_author: HashMap::from_iter(IntoIter::new([
                    (
                        "f1 author".to_string(),
                        vec![PageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]
                    ),
                    ("f3 author 1".to_string(), vec![PageRef { path: vec!["f3".to_string()] }, PageRef { path: vec!["f4".to_string()] }]),
                    ("f3 author 2".to_string(), vec![PageRef { path: vec!["f3".to_string()] }]),
                ])),
                all_tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string(), "t4".to_string()])),

                all_authors: HashSet::from_iter(IntoIter::new([
                    Author {
                        name: "f1 author".to_string(),
                        contacts: Default::default()
                    },
                    Author {
                        name: "f3 author 1".to_string(),
                        contacts: Default::default()
                    },
                    Author {
                        name: "f3 author 2".to_string(),
                        contacts: Default::default()
                    }
                ]))
            }
        );
    }
}
