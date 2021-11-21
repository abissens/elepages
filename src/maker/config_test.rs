#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::maker::config::{ComposeUnitConfig, StageValue};
    use crate::maker::{DateQueryConfig, SelectorConfig};
    use indoc::indoc;
    use std::array::IntoIter;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[test]
    fn parse_stage_value_configs() {
        let single_named: StageValue = serde_yaml::from_str(indoc! {"
            ---
            stage_type
        "})
        .unwrap();

        assert_eq!(single_named, StageValue::ProcessorWithoutConfigStage("stage_type".to_string()));

        let named_with_config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            type: stage_type
            config: [1,2,3]
        "})
        .unwrap();

        assert_eq!(
            named_with_config,
            StageValue::ProcessorStage {
                processor_type: "stage_type".to_string(),
                config: Value::Vec(vec![Value::I32(1), Value::I32(2), Value::I32(3)])
            }
        );

        let copy_stage: StageValue = serde_yaml::from_str(indoc! {"
            ---
            copy: 'a/**'
            dest: 'copied/dest'
        "})
        .unwrap();

        assert_eq!(
            copy_stage,
            StageValue::Copy {
                copy_selector: SelectorConfig::PathShortCut("a/**".to_string()),
                dest: "copied/dest".to_string(),
            }
        );

        let move_stage: StageValue = serde_yaml::from_str(indoc! {"
            ---
            move: { tag: 'a', path: 'a/**' }
            dest: 'moved/dest'
        "})
        .unwrap();

        assert_eq!(
            move_stage,
            StageValue::Move {
                move_selector: SelectorConfig::Base {
                    path: Some("a/**".to_string()),
                    tag: Some("a".to_string()),
                    tags: None,
                    ext: None,
                    author: None,
                    publishing: None
                },
                dest: "moved/dest".to_string(),
            }
        );

        let ignore_stage: StageValue = serde_yaml::from_str(indoc! {"
            ---
            ignore: { publishing: { afterDate: 'now' } }
        "})
        .unwrap();

        assert_eq!(
            ignore_stage,
            StageValue::Ignore {
                ignore_selector: SelectorConfig::Base {
                    publishing: Some(DateQueryConfig::AfterDate { after_date: "now".to_string() }),
                    path: None,
                    tag: None,
                    tags: None,
                    ext: None,
                    author: None,
                },
            }
        );

        let append_stage: StageValue = serde_yaml::from_str(indoc! {"
            ---
            append: stage_type
        "})
        .unwrap();

        assert_eq!(
            append_stage,
            StageValue::Append {
                append: Box::new(StageValue::ProcessorWithoutConfigStage("stage_type".to_string()))
            }
        );

        let replace_stage: StageValue = serde_yaml::from_str(indoc! {"
            ---
            replace: { tag: 'a', path: 'a/**' }
            by: stage_type
        "})
        .unwrap();

        assert_eq!(
            replace_stage,
            StageValue::Replace {
                replace: SelectorConfig::Base {
                    path: Some("a/**".to_string()),
                    tag: Some("a".to_string()),
                    tags: None,
                    ext: None,
                    author: None,
                    publishing: None
                },
                by: Box::new(StageValue::ProcessorWithoutConfigStage("stage_type".to_string()))
            }
        );

        let sequence: StageValue = serde_yaml::from_str(indoc! {"
            ---
            - type: stage_type_1
              config:
                a: some text
            - type: stage_type_2
              config: ~
            - type: stage_type_3
            - stage_type_4
            - copy: 'a/**'
              dest: 'copied/dest'
        "})
        .unwrap();

        assert_eq!(
            sequence,
            StageValue::Sequence(vec![
                StageValue::ProcessorStage {
                    processor_type: "stage_type_1".to_string(),
                    config: Value::Map(HashMap::from_iter(IntoIter::new([("a".to_string(), Value::String("some text".to_string()))]))),
                },
                StageValue::ProcessorStage {
                    processor_type: "stage_type_2".to_string(),
                    config: Value::None,
                },
                StageValue::ProcessorStage {
                    processor_type: "stage_type_3".to_string(),
                    config: Value::None,
                },
                StageValue::ProcessorWithoutConfigStage("stage_type_4".to_string()),
                StageValue::Copy {
                    copy_selector: SelectorConfig::PathShortCut("a/**".to_string()),
                    dest: "copied/dest".to_string(),
                }
            ])
        );

        let union: StageValue = serde_yaml::from_str(indoc! {"
            ---
            union:
                - type: stage_type_1
                  config:
                    a: some text
                - type: stage_type_2
                  config: ~
                - type: stage_type_3
                - stage_type_4
        "})
        .unwrap();

        assert_eq!(
            union,
            StageValue::Union {
                union: vec![
                    StageValue::ProcessorStage {
                        processor_type: "stage_type_1".to_string(),
                        config: Value::Map(HashMap::from_iter(IntoIter::new([("a".to_string(), Value::String("some text".to_string()))]))),
                    },
                    StageValue::ProcessorStage {
                        processor_type: "stage_type_2".to_string(),
                        config: Value::None,
                    },
                    StageValue::ProcessorStage {
                        processor_type: "stage_type_3".to_string(),
                        config: Value::None,
                    },
                    StageValue::ProcessorWithoutConfigStage("stage_type_4".to_string()),
                ]
            }
        );

        let compose: StageValue = serde_yaml::from_str(indoc! {"
            ---
            compose:
                - type: stage_type_1
                  config:
                    a: some text
                - type: stage_type_2
                  config: ~
                - type: stage_type_3
                - stage_type_4
                - inner: stage_type_5
                  selector: { ext: '.md' }
        "})
        .unwrap();

        assert_eq!(
            compose,
            StageValue::Composition {
                compose: vec![
                    ComposeUnitConfig::Create(StageValue::ProcessorStage {
                        processor_type: "stage_type_1".to_string(),
                        config: Value::Map(HashMap::from_iter(IntoIter::new([("a".to_string(), Value::String("some text".to_string()))]))),
                    }),
                    ComposeUnitConfig::Create(StageValue::ProcessorStage {
                        processor_type: "stage_type_2".to_string(),
                        config: Value::None,
                    }),
                    ComposeUnitConfig::Create(StageValue::ProcessorStage {
                        processor_type: "stage_type_3".to_string(),
                        config: Value::None,
                    }),
                    ComposeUnitConfig::Create(StageValue::ProcessorWithoutConfigStage("stage_type_4".to_string())),
                    ComposeUnitConfig::Replace {
                        inner: StageValue::ProcessorWithoutConfigStage("stage_type_5".to_string()),
                        selector: SelectorConfig::Base {
                            ext: Some(".md".to_string()),
                            path: None,
                            tag: None,
                            tags: None,
                            author: None,
                            publishing: None
                        }
                    }
                ]
            }
        );
    }
}
