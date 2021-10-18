use crate::pages::Metadata;
use crate::pages_error::PagesError;
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub(crate) enum MetadataTree {
    Root { sub: HashMap<String, MetadataTree> },
    Node { metadata: Option<Metadata>, sub: HashMap<String, MetadataTree> },
}

#[derive(PartialEq, Debug)]
pub(crate) struct MetadataNode<'a> {
    pub(crate) path: String,
    pub(crate) metadata: Option<&'a Metadata>,
}

impl MetadataTree {
    pub(crate) fn get_metadata_from_path<'a>(&'a self, path: &[String], result: &mut Vec<MetadataNode<'a>>) {
        self._get_metadata_from_path(path, result, "".to_string());
    }
    pub(crate) fn _get_metadata_from_path<'a>(&'a self, path: &[String], result: &mut Vec<MetadataNode<'a>>, current_name: String) {
        match self {
            MetadataTree::Root { sub } => {
                if path.is_empty() {
                    return;
                }
                if let Some(sub_tree) = sub.get(&path[0]) {
                    sub_tree._get_metadata_from_path(&path[1..], result, path[0].to_string());
                }
            }
            MetadataTree::Node { metadata, sub } => {
                match metadata {
                    Some(metadata) => result.push(MetadataNode {
                        path: current_name,
                        metadata: Some(&metadata),
                    }),
                    None => result.push(MetadataNode { path: current_name, metadata: None }),
                }
                if path.is_empty() {
                    return;
                }
                if let Some(sub_tree) = sub.get(&path[0]) {
                    sub_tree._get_metadata_from_path(&path[1..], result, path[0].to_string());
                }
            }
        }
    }

    pub(crate) fn push(&mut self, path: &[String], metadata: Metadata) -> anyhow::Result<()> {
        return match self {
            MetadataTree::Root { sub } => {
                if path.is_empty() {
                    return Err(PagesError::MetadataTree("path cannot be empty on root node".to_string()).into());
                }
                if !sub.contains_key(&path[0]) {
                    let mut node = MetadataTree::Node { metadata: None, sub: HashMap::new() };
                    node.push(&path[1..], metadata)?;
                    sub.insert(path[0].clone(), node);
                    return Ok(());
                }

                sub.get_mut(&path[0]).unwrap().push(&path[1..], metadata)?;
                Ok(())
            }
            MetadataTree::Node { metadata: node_metadata, sub } => {
                if path.is_empty() {
                    *node_metadata = Some(metadata);
                    return Ok(());
                }
                if !sub.contains_key(&path[0]) {
                    let mut node = MetadataTree::Node { metadata: None, sub: HashMap::new() };
                    node.push(&path[1..], metadata)?;
                    sub.insert(path[0].clone(), node);
                    return Ok(());
                }

                sub.get_mut(&path[0]).unwrap().push(&path[1..], metadata)?;
                Ok(())
            }
        };
    }
}
