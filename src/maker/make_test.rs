#[cfg(test)]
mod tests {
    use crate::maker::{Env, Maker, StageValue};
    use crate::stages::ComposeUnit::{CreateNewSet, ReplaceSubSet};
    use crate::stages::{ComposeStage, ExtSelector, GitMetadata, HandlebarsDir, HandlebarsStage, IndexStage, MdStage, PrefixSelector, RegexSelector, SequenceStage, ShadowPages, UnionStage};
    use indoc::indoc;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn build_default_named_stages_based_on_named_config() {
        let git_metadata_stage_config: StageValue = serde_yaml::from_str("git_metadata").unwrap();

        let mut env = Env::new();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let git_metadata_stage = Maker::default().make(&git_metadata_stage_config, &env).unwrap();

        if let Some(g) = git_metadata_stage.as_any().unwrap().downcast_ref::<GitMetadata>() {
            assert_eq!(&g.repo_path, &PathBuf::from_str("a/b/c").unwrap());
        } else {
            panic!("should downcast to GitMetadata");
        }

        let indexes_stage_config: StageValue = serde_yaml::from_str("indexes").unwrap();
        let indexes_stage = Maker::default().make(&indexes_stage_config, &Env::new()).unwrap();
        if let None = indexes_stage.as_any().unwrap().downcast_ref::<IndexStage>() {
            panic!("should downcast to IndexStage");
        }

        let shadow_stage_config: StageValue = serde_yaml::from_str("shadow").unwrap();
        let shadow_stage = Maker::default().make(&shadow_stage_config, &Env::new()).unwrap();
        if let None = shadow_stage.as_any().unwrap().downcast_ref::<ShadowPages>() {
            panic!("should downcast to ShadowPages");
        }

        let md_stage_config: StageValue = serde_yaml::from_str("md").unwrap();
        let md_stage = Maker::default().make(&md_stage_config, &Env::new()).unwrap();
        if let None = md_stage.as_any().unwrap().downcast_ref::<MdStage>() {
            panic!("should downcast to MdStage");
        }

        let hb_stage_config: StageValue = serde_yaml::from_str("handlebars").unwrap();
        let hb_stage = Maker::default().make(&hb_stage_config, &env).unwrap();
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

        if let Err(e) = Maker::default().make(&config, &Env::new()) {
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

        if let Err(e) = Maker::default().make(&config, &Env::new()) {
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

        let stage = Maker::default().make(&config, &env).unwrap();

        let seq = stage.as_any().unwrap().downcast_ref::<SequenceStage>().expect("SequenceStage");
        seq.stages.get(0).unwrap().as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
        seq.stages.get(1).unwrap().as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
        seq.stages.get(2).unwrap().as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
    }

    #[test]
    fn build_union_stage() {
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

        let stage = Maker::default().make(&config, &env).unwrap();

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
                - inner: md
                  selector: [regex, '.*?.md$']
                - inner: git_metadata
                  selector: [prefix, 'a/b']
                - inner: handlebars
                  selector: [ext, '.hbs']
        "})
        .unwrap();

        let mut env = Env::new();
        env.insert("root_path".to_string(), Box::new(PathBuf::from_str("a/b/c").unwrap()));

        let stage = Maker::default().make(&config, &env).unwrap();

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
            stage.as_any().unwrap().downcast_ref::<MdStage>().expect("MdStage");
            let re = selector.as_any().unwrap().downcast_ref::<RegexSelector>().expect("RegexSelector");
            assert_eq!(re.0.to_string(), ".*?.md$");
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }

        if let ReplaceSubSet(selector, stage) = compose.units.get(3).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<GitMetadata>().expect("GitMetadata");
            let pre = selector.as_any().unwrap().downcast_ref::<PrefixSelector>().expect("PrefixSelector");
            assert_eq!(pre.0, vec!["a", "b"]);
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }

        if let ReplaceSubSet(selector, stage) = compose.units.get(4).unwrap().as_ref() {
            stage.as_any().unwrap().downcast_ref::<HandlebarsStage>().expect("HandlebarsStage");
            let ext = selector.as_any().unwrap().downcast_ref::<ExtSelector>().expect("ExtSelector");
            assert_eq!(ext.0, ".hbs");
        } else {
            panic!("unit should be of variant ReplaceSubSet")
        }
    }
}
