#[cfg(test)]
mod tests {
    use crate::maker::{Env, Maker, StageValue};
    use crate::pages::{ExtSelector, PathSelector};
    use crate::stages::ComposeUnit::{CreateNewSet, ReplaceSubSet};
    use crate::stages::{ComposeStage, GitMetadata, HandlebarsDir, HandlebarsStage, IndexStage, MdStage, SequenceStage, ShadowPages, Stage, UnionStage};
    use indoc::indoc;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn build_default_processor_stages_based_on_config() {
        let git_metadata_stage_config: StageValue = serde_yaml::from_str("git_metadata").unwrap();

        let mut env = Env::new();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let git_metadata_stage = Maker::default().make(None, &git_metadata_stage_config, &env).unwrap();
        assert_eq!(git_metadata_stage.name(), "git metadata stage");

        if let Some(g) = git_metadata_stage.as_any().unwrap().downcast_ref::<GitMetadata>() {
            assert_eq!(&g.repo_path, &PathBuf::from_str("a/b/c").unwrap());
        } else {
            panic!("should downcast to GitMetadata");
        }

        let indexes_stage_config: StageValue = serde_yaml::from_str("indexes").unwrap();
        let indexes_stage = Maker::default().make(None, &indexes_stage_config, &Env::new()).unwrap();
        assert_eq!(indexes_stage.name(), "index stage");
        if let None = indexes_stage.as_any().unwrap().downcast_ref::<IndexStage>() {
            panic!("should downcast to IndexStage");
        }

        let shadow_stage_config: StageValue = serde_yaml::from_str("shadow").unwrap();
        let shadow_stage = Maker::default().make(None, &shadow_stage_config, &Env::new()).unwrap();
        assert_eq!(shadow_stage.name(), "shadow pages stage");
        if let None = shadow_stage.as_any().unwrap().downcast_ref::<ShadowPages>() {
            panic!("should downcast to ShadowPages");
        }

        let md_stage_config: StageValue = serde_yaml::from_str("md").unwrap();
        let md_stage = Maker::default().make(None, &md_stage_config, &Env::new()).unwrap();
        assert_eq!(md_stage.name(), "markdown stage");
        if let None = md_stage.as_any().unwrap().downcast_ref::<MdStage>() {
            panic!("should downcast to MdStage");
        }

        let hb_stage_config: StageValue = serde_yaml::from_str("handlebars").unwrap();
        let hb_stage = Maker::default().make(None, &hb_stage_config, &env).unwrap();
        assert_eq!(hb_stage.name(), "handlebars stage");
        if let Some(hb) = hb_stage.as_any().unwrap().downcast_ref::<HandlebarsStage>() {
            let hbl: &HandlebarsDir = hb.lookup.as_any().unwrap().downcast_ref().unwrap();
            assert_eq!(&hbl.base_path, &PathBuf::from_str("a/b/c").unwrap());
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

        let mut env = Env::new();
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
        let indexes_stage = Maker::default().make(None, &indexes_stage_config, &Env::new()).unwrap();
        assert_eq!(indexes_stage.name(), "index stage renamed");
        if let None = indexes_stage.as_any().unwrap().downcast_ref::<IndexStage>() {
            panic!("should downcast to IndexStage");
        }

        let shadow_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: shadow pages stage renamed
            stage: shadow
        "})
        .unwrap();
        let shadow_stage = Maker::default().make(None, &shadow_stage_config, &Env::new()).unwrap();
        assert_eq!(shadow_stage.name(), "shadow pages stage renamed");
        if let None = shadow_stage.as_any().unwrap().downcast_ref::<ShadowPages>() {
            panic!("should downcast to ShadowPages");
        }

        let md_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: markdown stage renamed
            stage: md
        "})
        .unwrap();
        let md_stage = Maker::default().make(None, &md_stage_config, &Env::new()).unwrap();
        assert_eq!(md_stage.name(), "markdown stage renamed");
        if let None = md_stage.as_any().unwrap().downcast_ref::<MdStage>() {
            panic!("should downcast to MdStage");
        }

        let hb_stage_config: StageValue = serde_yaml::from_str(indoc! {"
            name: handlebars stage renamed
            stage: handlebars
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

        if let Err(e) = Maker::default().make(None, &config, &Env::new()) {
            assert_eq!(e.to_string(), "stage some_stage not found")
        } else {
            panic!("should return Err");
        }
    }

    #[test]
    fn return_err_when_named_selector_not_found() {
        let config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            compose:
              - inner: stage_name
                selector: [some_selector, ~]
        "})
        .unwrap();

        if let Err(e) = Maker::default().make(None, &config, &Env::new()) {
            assert_eq!(e.to_string(), "selector some_selector not found")
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
            - handlebars
        "})
        .unwrap();

        let mut env = Env::new();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(None, &config, &env).unwrap();

        let seq = stage.as_any().unwrap().downcast_ref::<SequenceStage>().expect("SequenceStage");
        seq.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        seq.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        seq.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
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
                stage: handlebars
        "})
        .unwrap();

        let mut env = Env::new();
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
                - handlebars
        "})
        .unwrap();

        let mut env = Env::new();
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
              - handlebars
        "})
        .unwrap();

        let mut env = Env::new();
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
                  selector: [path, ['a', 'b']]
                - inner: handlebars
                  selector: [ext, '.hbs']
        "})
        .unwrap();

        let mut env = Env::new();
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
                  selector: [path, ['a', 'b']]
                - inner: handlebars
                  selector: [ext, '.hbs']
        "})
        .unwrap();

        let mut env = Env::new();
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
