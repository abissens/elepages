#[cfg(test)]
mod tests {
    use crate::maker::config::{ComposeUnitConfig, StageValue, ValueConfig};
    use indoc::indoc;
    use std::array::IntoIter;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[test]
    fn parse_stage_value_configs() {
        let single_named: StageValue = serde_yaml::from_str(indoc! {"
            ---
            stage_name
        "})
        .unwrap();

        assert_eq!(single_named, StageValue::NamedWithoutConfig("stage_name".to_string()));

        let named_with_config: StageValue = serde_yaml::from_str(indoc! {"
            ---
            stage: stage_name
            config: [1,2,3]
        "})
        .unwrap();

        assert_eq!(
            named_with_config,
            StageValue::Named {
                name: "stage_name".to_string(),
                config: ValueConfig::Vec(vec![ValueConfig::I32(1), ValueConfig::I32(2), ValueConfig::I32(3)])
            }
        );

        let sequence: StageValue = serde_yaml::from_str(indoc! {"
            ---
            - stage: stage_name_1
              config:
                a: some text
            - stage: stage_name_2
              config: ~
            - stage: stage_name_3
            - stage_name_4
        "})
        .unwrap();

        assert_eq!(
            sequence,
            StageValue::Sequence(vec![
                StageValue::Named {
                    name: "stage_name_1".to_string(),
                    config: ValueConfig::Map(HashMap::from_iter(IntoIter::new([("a".to_string(), ValueConfig::String("some text".to_string()))]))),
                },
                StageValue::Named {
                    name: "stage_name_2".to_string(),
                    config: ValueConfig::None,
                },
                StageValue::Named {
                    name: "stage_name_3".to_string(),
                    config: ValueConfig::None,
                },
                StageValue::NamedWithoutConfig("stage_name_4".to_string()),
            ])
        );

        let union: StageValue = serde_yaml::from_str(indoc! {"
            ---
            union:
                - stage: stage_name_1
                  config:
                    a: some text
                - stage: stage_name_2
                  config: ~
                - stage: stage_name_3
                - stage_name_4
        "})
        .unwrap();

        assert_eq!(
            union,
            StageValue::Union {
                union: vec![
                    StageValue::Named {
                        name: "stage_name_1".to_string(),
                        config: ValueConfig::Map(HashMap::from_iter(IntoIter::new([("a".to_string(), ValueConfig::String("some text".to_string()))]))),
                    },
                    StageValue::Named {
                        name: "stage_name_2".to_string(),
                        config: ValueConfig::None,
                    },
                    StageValue::Named {
                        name: "stage_name_3".to_string(),
                        config: ValueConfig::None,
                    },
                    StageValue::NamedWithoutConfig("stage_name_4".to_string()),
                ]
            }
        );

        let compose: StageValue = serde_yaml::from_str(indoc! {"
            ---
            compose:
                - stage: stage_name_1
                  config:
                    a: some text
                - stage: stage_name_2
                  config: ~
                - stage: stage_name_3
                - stage_name_4
                - inner: stage_name_5
                  selector: [regexp, '.*?.md$']
        "})
        .unwrap();

        assert_eq!(
            compose,
            StageValue::Composition {
                compose: vec![
                    ComposeUnitConfig::Create(StageValue::Named {
                        name: "stage_name_1".to_string(),
                        config: ValueConfig::Map(HashMap::from_iter(IntoIter::new([("a".to_string(), ValueConfig::String("some text".to_string()))]))),
                    }),
                    ComposeUnitConfig::Create(StageValue::Named {
                        name: "stage_name_2".to_string(),
                        config: ValueConfig::None,
                    }),
                    ComposeUnitConfig::Create(StageValue::Named {
                        name: "stage_name_3".to_string(),
                        config: ValueConfig::None,
                    }),
                    ComposeUnitConfig::Create(StageValue::NamedWithoutConfig("stage_name_4".to_string())),
                    ComposeUnitConfig::Replace {
                        inner: StageValue::NamedWithoutConfig("stage_name_5".to_string()),
                        selector: ("regexp".to_string(), ValueConfig::String(".*?.md$".to_string()))
                    }
                ]
            }
        );
    }
}
