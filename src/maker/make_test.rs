#[cfg(test)]
mod tests {
    use crate::maker::{Maker, StageValue};
    use crate::pages::{DateQuery, Env, ExtSelector, Logical, PathSelector, PublishingDateSelector, TagSelector};
    use crate::stages::ComposeUnit::{CreateNewSet, ReplaceSubSet};
    use crate::stages::{AppendStage, ComposeStage, CopyCut, GitMetadata, HandlebarsDir, HandlebarsStage, IndexStage, MdStage, ReplaceStage, SequenceStage, ShadowPages, Stage, UnionStage};
    use chrono::{DateTime, Utc};
    use indoc::indoc;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn build_default_processor_stages_based_on_config() {
        let git_metadata_stage_config: StageValue = serde_yaml::from_str("git_metadata").unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let git_metadata_stage = Maker::default().make(None, &git_metadata_stage_config, &env).unwrap();
        assert_eq!(git_metadata_stage.name(), "git metadata stage");

        if let Some(g) = git_metadata_stage.as_any().unwrap().downcast_ref::<GitMetadata>() {
            assert_eq!(&g.repo_path, &PathBuf::from_str("a/b/c").unwrap());
        } else {
            panic!("should downcast to GitMetadata");
        }

        let indexes_stage_config: StageValue = serde_yaml::from_str("indexes").unwrap();
        let indexes_stage = Maker::default().make(None, &indexes_stage_config, &Env::test()).unwrap();
        assert_eq!(indexes_stage.name(), "index stage");
        if let None = indexes_stage.as_any().unwrap().downcast_ref::<IndexStage>() {
            panic!("should downcast to IndexStage");
        }

        let shadow_stage_config: StageValue = serde_yaml::from_str("shadow").unwrap();
        let shadow_stage = Maker::default().make(None, &shadow_stage_config, &Env::test()).unwrap();
        assert_eq!(shadow_stage.name(), "shadow pages stage");
        if let None = shadow_stage.as_any().unwrap().downcast_ref::<ShadowPages>() {
            panic!("should downcast to ShadowPages");
        }

        let md_stage_config: StageValue = serde_yaml::from_str("md").unwrap();
        let md_stage = Maker::default().make(None, &md_stage_config, &Env::test()).unwrap();
        assert_eq!(md_stage.name(), "markdown stage");
        if let None = md_stage.as_any().unwrap().downcast_ref::<MdStage>() {
            panic!("should downcast to MdStage");
        }

        let hb_stage_config: StageValue = serde_yaml::from_str("{type: 'handlebars', config: 'd/e' }").unwrap();
        let hb_stage = Maker::default().make(None, &hb_stage_config, &env).unwrap();
        assert_eq!(hb_stage.name(), "handlebars stage");
        if let Some(hb) = hb_stage.as_any().unwrap().downcast_ref::<HandlebarsStage>() {
            let hbl: &HandlebarsDir = hb.lookup.as_any().unwrap().downcast_ref().unwrap();
            assert_eq!(&hbl.base_path, &PathBuf::from_str("d/e").unwrap());
        } else {
            panic!("should downcast to HandlebarsStage");
        }
    }

    #[test]
    fn build_named_default_processor_stages_based_on_config() {
        let git_metadata_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: git metadata renamed
            stage: git_metadata
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let git_metadata_stage = Maker::default().make(None, &git_metadata_stage_config, &env).unwrap();
        assert_eq!(git_metadata_stage.name(), "git metadata renamed");

        if let Some(g) = git_metadata_stage.as_any().unwrap().downcast_ref::<GitMetadata>() {
            assert_eq!(&g.repo_path, &PathBuf::from_str("a/b/c").unwrap());
        } else {
            panic!("should downcast to GitMetadata");
        }

        let indexes_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: index stage renamed
            stage: indexes
        "})
        .unwrap();
        let indexes_stage = Maker::default().make(None, &indexes_stage_config, &Env::test()).unwrap();
        assert_eq!(indexes_stage.name(), "index stage renamed");
        if let None = indexes_stage.as_any().unwrap().downcast_ref::<IndexStage>() {
            panic!("should downcast to IndexStage");
        }

        let shadow_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: shadow pages stage renamed
            stage: shadow
        "})
        .unwrap();
        let shadow_stage = Maker::default().make(None, &shadow_stage_config, &Env::test()).unwrap();
        assert_eq!(shadow_stage.name(), "shadow pages stage renamed");
        if let None = shadow_stage.as_any().unwrap().downcast_ref::<ShadowPages>() {
            panic!("should downcast to ShadowPages");
        }

        let md_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: markdown stage renamed
            stage: md
        "})
        .unwrap();
        let md_stage = Maker::default().make(None, &md_stage_config, &Env::test()).unwrap();
        assert_eq!(md_stage.name(), "markdown stage renamed");
        if let None = md_stage.as_any().unwrap().downcast_ref::<MdStage>() {
            panic!("should downcast to MdStage");
        }

        let hb_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: handlebars stage renamed
            stage: {type: handlebars, config: a/b/c }
        "})
        .unwrap();
        let hb_stage = Maker::default().make(None, &hb_stage_config, &env).unwrap();
        assert_eq!(hb_stage.name(), "handlebars stage renamed");
        if let Some(hb) = hb_stage.as_any().unwrap().downcast_ref::<HandlebarsStage>() {
            let hbl: &HandlebarsDir = hb.lookup.as_any().unwrap().downcast_ref().unwrap();
            assert_eq!(&hbl.base_path, &PathBuf::from_str("a/b/c").unwrap());
        } else {
            panic!("should downcast to HandlebarsStage");
        }
    }

    #[test]
    fn return_err_when_named_stage_not_found() {
        let config: StageValue = serde_yaml::from_str("some_stage").unwrap();

        if let Err(e) = Maker::default().make(None, &config, &Env::test()) {
            assert_eq!(e.to_string(), "stage some_stage not found")
        } else {
            panic!("should return Err");
        }
    }

    #[test]
    fn build_sequence_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            - git_metadata
            - md
            - {type: handlebars, config: a/b/c }
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let seq = stage.as_any().unwrap().downcast_ref::<SequenceStage>().expect("SequenceStage");
        seq.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        seq.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        seq.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
    }

    #[test]
    fn build_copy_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            copy: 'a/**'
            dest: 'copied/dest'
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let copy_cut = stage.as_any().unwrap().downcast_ref::<CopyCut>().expect("CopyCut");
        match copy_cut {
            CopyCut::Copy { selector, dest, name } => {
                assert_eq!(name, "copy stage");
                assert_eq!(dest, &["copied", "dest"]);
                let selector = selector.as_any().unwrap().downcast_ref::<PathSelector>().expect("PathSelector");
                assert_eq!(selector.query, vec!["a".to_string(), "**".to_string()]);
            }
            _ => panic!("CopyCut::Copy"),
        };
    }

    #[test]
    fn build_move_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            move: 'a/**'
            dest: 'moved/dest'
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let copy_cut = stage.as_any().unwrap().downcast_ref::<CopyCut>().expect("CopyCut");
        match copy_cut {
            CopyCut::Move { selector, dest, name } => {
                assert_eq!(name, "move stage");
                assert_eq!(dest, &["moved", "dest"]);
                let selector = selector.as_any().unwrap().downcast_ref::<PathSelector>().expect("PathSelector");
                assert_eq!(selector.query, vec!["a".to_string(), "**".to_string()]);
            }
            _ => panic!("CopyCut::Move"),
        };
    }

    #[test]
    fn build_ignore_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            ignore: 'a/**'
        "})
        .unwrap();

        let stage = Maker::default().make(None, &config, &Env::test()).unwrap();

        let copy_cut = stage.as_any().unwrap().downcast_ref::<CopyCut>().expect("CopyCut");
        match copy_cut {
            CopyCut::Ignore { selector, name } => {
                assert_eq!(name, "ignore stage");
                let selector = selector.as_any().unwrap().downcast_ref::<PathSelector>().expect("PathSelector");
                assert_eq!(selector.query, vec!["a".to_string(), "**".to_string()]);
            }
            _ => panic!("CopyCut::Ignore"),
        };
    }

    #[test]
    fn build_append_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            append: indexes
        "})
        .unwrap();

        let stage = Maker::default().make(None, &config, &Env::test()).unwrap();

        let append_stage = stage.as_any().unwrap().downcast_ref::<AppendStage>().expect("AppendStage");
        assert_eq!(append_stage.name, "append stage");
        let index_stage = append_stage.inner.as_any().unwrap().downcast_ref::<IndexStage>().expect("IndexStage");
        assert_eq!(index_stage.name, "index stage");
    }

    #[test]
    fn build_replace_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            replace: 'a/**'
            by: indexes
        "})
        .unwrap();

        let stage = Maker::default().make(None, &config, &Env::test()).unwrap();

        let replace_stage = stage.as_any().unwrap().downcast_ref::<ReplaceStage>().expect("ReplaceStage");
        assert_eq!(replace_stage.name, "replace stage");
        let index_stage = replace_stage.inner.as_any().unwrap().downcast_ref::<IndexStage>().expect("IndexStage");
        assert_eq!(index_stage.name, "index stage");
    }

    #[test]
    fn build_copy_cut_stages() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            - copy: 'a/**'
              dest: 'copied/dest'
            - move: [{tag: 'draft'}, {publishing: {afterDate: '2021-10-20'}}]
              dest: 'moved/dest'
            - ignore: {publishing: {afterTime: '2021-10-20T22:00:00+00:00'}}
        "})
        .unwrap();

        let env = Env::test();

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let seq = stage.as_any().unwrap().downcast_ref::<SequenceStage>().expect("SequenceStage");
        let cc_1 = seq.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<CopyCut>().expect("CopyCut 0");
        let cc_2 = seq.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<CopyCut>().expect("CopyCut 1");
        let cc_3 = seq.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<CopyCut>().expect("CopyCut 2");

        match cc_1 {
            CopyCut::Copy { dest, selector, name } => {
                assert_eq!(name, "copy stage");
                assert_eq!(dest, &vec!["copied".to_string(), "dest".to_string()]);
                let selector = selector.as_any().unwrap().downcast_ref::<PathSelector>().expect("PathSelector");
                assert_eq!(selector.query, vec!["a".to_string(), "**".to_string()]);
            }
            _ => panic!("CopyCut::Copy"),
        }

        match cc_2 {
            CopyCut::Move { dest, selector, name } => {
                assert_eq!(name, "move stage");
                assert_eq!(dest, &vec!["moved".to_string(), "dest".to_string()]);
                let l = selector.as_any().unwrap().downcast_ref::<Logical>().expect("Logical");
                match l {
                    Logical::And(and) => {
                        let a0 = and[0].as_any().unwrap().downcast_ref::<TagSelector>().expect("TagSelector");
                        assert_eq!(a0.tag, "draft".to_string());

                        let a1 = and[1].as_any().unwrap().downcast_ref::<PublishingDateSelector>().expect("PublishingDateSelector");
                        assert_eq!(a1.query, DateQuery::After(DateTime::<Utc>::from_str("2021-10-20T23:59:59+00:00").unwrap().timestamp()));
                    }
                    _ => panic!("Logical::And"),
                }
            }
            _ => panic!("CopyCut::Move"),
        }

        match cc_3 {
            CopyCut::Ignore { selector, name } => {
                assert_eq!(name, "ignore stage");
                let selector = selector.as_any().unwrap().downcast_ref::<PublishingDateSelector>().expect("PublishingDateSelector");
                assert_eq!(selector.query, DateQuery::After(DateTime::<Utc>::from_str("2021-10-20T22:00:00+00:00").unwrap().timestamp()));
            }
            _ => panic!("CopyCut::Copy"),
        }
    }

    #[test]
    fn build_named_sequence_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            name: my sequence
            stage:
              - git_metadata
              - md
              - name: my handlebars
                stage: {type: handlebars, config: a/b/c }
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();
        assert_eq!(stage.name(), "my sequence");
        let seq = stage.as_any().unwrap().downcast_ref::<SequenceStage>().expect("SequenceStage");
        seq.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        seq.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        let hb_stage = seq.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
        assert_eq!(hb_stage.name(), "my handlebars");
    }

    #[test]
    fn build_union_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            name: my union
            stage:
              union:
                - git_metadata
                - md
                - {type: handlebars, config: a/b/c }
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();
        assert_eq!(stage.name(), "my union");

        let union = stage.as_any().unwrap().downcast_ref::<UnionStage>().expect("UnionStage");
        union.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        union.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        union.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
    }

    #[test]
    fn build_named_union_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            union:
              - git_metadata
              - md
              - {type: handlebars, config: a/b/c }
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let union = stage.as_any().unwrap().downcast_ref::<UnionStage>().expect("UnionStage");
        union.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        union.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        union.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
    }

    #[test]
    fn build_compose_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            compose:
                - md
                - git_metadata
                - inner: git_metadata
                  selector: { path: 'a/b' }
                - inner:
                    type: handlebars
                    config: /d/e
                  selector: { ext: '.hbs' }
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let compose = stage.as_any().unwrap().downcast_ref::<ComposeStage>().expect("ComposeStage");

        if let CreateNewSet(stage) = compose.units.get(0).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        } else {
            panic!("unit should be of variant CreateNewSet")
        }

        if let CreateNewSet(stage) = compose.units.get(1).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        } else {
            panic!("unit should be of variant CreateNewSet")
        }

        if let ReplaceSubSet(selector, stage) = compose.units.get(2).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
            let path = selector.as_any().unwrap().downcast_ref::<PathSelector>().expect("PathSelector");
            assert_eq!(path.query, vec!["a", "b"]);
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }

        if let ReplaceSubSet(selector, stage) = compose.units.get(3).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
            let ext_selector = selector.as_any().unwrap().downcast_ref::<ExtSelector>().expect("ExtSelector");
            assert_eq!(ext_selector.ext, ".hbs");
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }
    }

    #[test]
    fn build_named_compose_stage() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            name: my compose
            stage:
              compose:
                - md
                - git_metadata
                - inner: git_metadata
                  selector: { path: 'a/b' }
                - inner: {type: handlebars, config: a/b/c }
                  selector: { ext: '.hbs' }
        "})
        .unwrap();

        let mut env = Env::test();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();
        assert_eq!(stage.name(), "my compose");

        let compose = stage.as_any().unwrap().downcast_ref::<ComposeStage>().expect("ComposeStage");

        if let CreateNewSet(stage) = compose.units.get(0).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        } else {
            panic!("unit should be of variant CreateNewSet")
        }

        if let CreateNewSet(stage) = compose.units.get(1).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        } else {
            panic!("unit should be of variant CreateNewSet")
        }

        if let ReplaceSubSet(selector, stage) = compose.units.get(2).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
            let path = selector.as_any().unwrap().downcast_ref::<PathSelector>().expect("PathSelector");
            assert_eq!(path.query, vec!["a", "b"]);
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }

        if let ReplaceSubSet(selector, stage) = compose.units.get(3).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
            let ext_selector = selector.as_any().unwrap().downcast_ref::<ExtSelector>().expect("ExtSelector");
            assert_eq!(ext_selector.ext, ".hbs");
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }
    }
}
