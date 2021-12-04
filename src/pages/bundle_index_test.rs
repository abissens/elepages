#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{AlwaysQuery, AndQuery, Author, AuthorQuery, BundleIndex, BundlePagination, DateIndex, Metadata, MetadataIndex, OrQuery, PageBundle, PageIndex, PageRef, TagQuery, VecBundle};
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
                    title: Some(Arc::new("f1 & xyz title".to_string())),
                    summary: Some(Arc::new("f1 summary".to_string())),
                    authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                        name: "f1 author".to_string(),
                        contacts: Default::default(),
                    })])),
                    tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                    publishing_date: Some(1637582000),
                    last_edit_date: Some(1637581000),
                    data: HashMap::default(),
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
                        title: Some("f1 & xyz title".to_string()),
                        url_title: Some("f1_%26_xyz_title".to_string()),
                        summary: Some("f1 summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new(["f1 author".to_string()])),
                        tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                        publishing_date: Some(DateIndex {
                            timestamp: 1637582000,
                            i_year: 2021,
                            short_year: "21".to_string(),
                            i_month: 11,
                            month: "11".to_string(),
                            short_month: "Nov".to_string(),
                            long_month: "November".to_string(),
                            i_day: 22,
                            day: "22".to_string(),
                            short_day: "Mon".to_string(),
                            long_day: "Monday".to_string(),
                            i_hour: 11,
                            i_minute: 53,
                            i_second: 20
                        }),
                        last_edit_date: Some(DateIndex {
                            timestamp: 1637581000,
                            i_year: 2021,
                            short_year: "21".to_string(),
                            i_month: 11,
                            month: "11".to_string(),
                            short_month: "Nov".to_string(),
                            long_month: "November".to_string(),
                            i_day: 22,
                            day: "22".to_string(),
                            short_day: "Mon".to_string(),
                            long_day: "Monday".to_string(),
                            i_hour: 11,
                            i_minute: 36,
                            i_second: 40
                        }),
                        data: HashMap::default(),
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
                        data: HashMap::from_iter(IntoIter::new([
                            ("a".to_string(), Value::String("a".to_string())),
                            (
                                "b".to_string(),
                                Value::Vec(vec![Value::String("1".to_string()), Value::String("2".to_string()), Value::String("3".to_string())]),
                            ),
                            ("c".to_string(), Value::I32(10)),
                        ])),
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
                        data: HashMap::default(),
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
                        data: HashMap::default(),
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
                            url_title: Some("f1_title".to_string()),
                            summary: Some("f1 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f1 author".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::from_iter(IntoIter::new([
                                ("a".to_string(), Value::String("a".to_string())),
                                (
                                    "b".to_string(),
                                    Value::Vec(vec![Value::String("1".to_string()), Value::String("2".to_string()), Value::String("3".to_string())])
                                ),
                                ("c".to_string(), Value::I32(10)),
                            ])),
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
                            url_title: Some("f3_title".to_string()),
                            summary: Some("f3 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f3 author 1".to_string(), "f3 author 2".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t3".to_string(), "t4".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        })
                    },
                    PageIndex {
                        page_ref: PageRef { path: vec!["f4".to_string()] },
                        metadata: Some(MetadataIndex {
                            title: Some("f4 title".to_string()),
                            url_title: Some("f4_title".to_string()),
                            summary: Some("f4 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f3 author 1".to_string()])),
                            tags: HashSet::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
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

    #[test]
    fn sort_generated_index_pages() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(1637583000),
                        last_edit_date: None,
                        data: Default::default(),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(1637582000),
                        last_edit_date: None,
                        data: Default::default(),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f3".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(1637584000),
                        last_edit_date: None,
                        data: Default::default(),
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
                        page_ref: PageRef { path: vec!["f3".to_string()] },
                        metadata: Some(MetadataIndex {
                            title: None,
                            url_title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: Some(DateIndex {
                                timestamp: 1637584000,
                                i_year: 2021,
                                short_year: "21".to_string(),
                                i_month: 11,
                                month: "11".to_string(),
                                short_month: "Nov".to_string(),
                                long_month: "November".to_string(),
                                i_day: 22,
                                day: "22".to_string(),
                                short_day: "Mon".to_string(),
                                long_day: "Monday".to_string(),
                                i_hour: 12,
                                i_minute: 26,
                                i_second: 40
                            }),
                            last_edit_date: None,
                            data: Default::default()
                        })
                    },
                    PageIndex {
                        page_ref: PageRef { path: vec!["f1".to_string()] },
                        metadata: Some(MetadataIndex {
                            title: None,
                            url_title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: Some(DateIndex {
                                timestamp: 1637583000,
                                i_year: 2021,
                                short_year: "21".to_string(),
                                i_month: 11,
                                month: "11".to_string(),
                                short_month: "Nov".to_string(),
                                long_month: "November".to_string(),
                                i_day: 22,
                                day: "22".to_string(),
                                short_day: "Mon".to_string(),
                                long_day: "Monday".to_string(),
                                i_hour: 12,
                                i_minute: 10,
                                i_second: 00
                            }),
                            last_edit_date: None,
                            data: Default::default()
                        })
                    },
                    PageIndex {
                        page_ref: PageRef { path: vec!["f2".to_string()] },
                        metadata: Some(MetadataIndex {
                            title: None,
                            url_title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: Some(DateIndex {
                                timestamp: 1637582000,
                                i_year: 2021,
                                short_year: "21".to_string(),
                                i_month: 11,
                                month: "11".to_string(),
                                short_month: "Nov".to_string(),
                                long_month: "November".to_string(),
                                i_day: 22,
                                day: "22".to_string(),
                                short_day: "Mon".to_string(),
                                long_day: "Monday".to_string(),
                                i_hour: 11,
                                i_minute: 53,
                                i_second: 20
                            }),
                            last_edit_date: None,
                            data: Default::default()
                        })
                    },
                ],
                pages_by_tag: HashMap::default(),
                pages_by_author: HashMap::default(),
                all_tags: HashSet::default(),
                all_authors: HashSet::default()
            }
        );
    }

    #[test]
    fn query_bundle_index() {
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
                        data: HashMap::default(),
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
                        data: HashMap::default(),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: Some(Arc::new("f4 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "f3 author".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: String::new(),
                }),
            ],
        });

        let bundle_index = BundleIndex::from(&vec_bundle);

        let result = bundle_index.query(&AlwaysQuery, &BundlePagination { skip: None, limit: None });
        assert_eq!(result, bundle_index.all_pages.iter().collect::<Vec<&PageIndex>>());

        let result = bundle_index.query(
            &OrQuery(vec![Box::new(AuthorQuery("f3 author".to_string())), Box::new(AuthorQuery("f1 author".to_string()))]),
            &BundlePagination { skip: None, limit: None },
        );
        assert_eq!(result, vec![bundle_index.all_pages.get(0).unwrap(), bundle_index.all_pages.get(3).unwrap()]);

        let result = bundle_index.query(&TagQuery("t1".to_string()), &BundlePagination { skip: None, limit: None });
        assert_eq!(result, vec![bundle_index.all_pages.get(0).unwrap()]);

        let result = bundle_index.query(
            &AndQuery(vec![Box::new(TagQuery("t3".to_string())), Box::new(TagQuery("t2".to_string()))]),
            &BundlePagination { skip: None, limit: None },
        );
        assert_eq!(result, vec![bundle_index.all_pages.get(0).unwrap()]);
    }

    #[test]
    fn paginate_bundle_index_query() {
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
                        data: HashMap::default(),
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
                        data: HashMap::default(),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: Some(Arc::new("f4 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "f3 author".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: String::new(),
                }),
            ],
        });

        let bundle_index = BundleIndex::from(&vec_bundle);

        let result = bundle_index.query(&AlwaysQuery, &BundlePagination { skip: None, limit: None });
        assert_eq!(result, bundle_index.all_pages.iter().collect::<Vec<&PageIndex>>());

        let result = bundle_index.query(&AlwaysQuery, &BundlePagination { skip: None, limit: Some(2) });
        assert_eq!(result, vec![bundle_index.all_pages.get(0).unwrap(), bundle_index.all_pages.get(1).unwrap()]);

        let result = bundle_index.query(&AlwaysQuery, &BundlePagination { skip: Some(1), limit: Some(1) });
        assert_eq!(result, vec![bundle_index.all_pages.get(1).unwrap()]);

        let result = bundle_index.query(&AlwaysQuery, &BundlePagination { skip: Some(2), limit: None });
        assert_eq!(result, vec![bundle_index.all_pages.get(2).unwrap(), bundle_index.all_pages.get(3).unwrap()]);
    }
}
