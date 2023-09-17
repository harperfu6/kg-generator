use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use regex::RegexSet;
use snafu::prelude::*;

#[derive(Debug)]
pub struct Graph {
    pub graph_name: String,
    pub triples: Vec<Triple>,
}

pub type Node = String;
pub type Link = String;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Triple {
    pub subject: Node,
    pub predicate: Link,
    pub object: Node,
}

#[derive(Debug)]
pub struct NodeCount {
    pub node: Node,
    pub count: usize,
}

impl Graph {
    pub fn new(graph_name: &str, triples: Vec<Triple>) -> Self {
        let graph = Graph {
            graph_name: graph_name.to_string(),
            triples,
        };
        graph.remove_duplicates()
    }

    fn remove_duplicates(self) -> Self {
        let mut triples_set: HashSet<Triple> = HashSet::new();
        for triple in self.triples {
            triples_set.insert(triple);
        }

        let mut triple_vec: Vec<Triple> = Vec::new();
        for triple in triples_set {
            triple_vec.push(triple);
        }

        Graph {
            graph_name: self.graph_name,
            triples: triple_vec,
        }
    }

    /// ノードのユニークリストを取得する
    pub fn get_unique_node(&self) -> Vec<Node> {
        let mut nodes: HashSet<Node> = HashSet::new();
        for triple in &self.triples {
            nodes.insert(triple.subject.clone());
            nodes.insert(triple.object.clone());
        }

        let mut node_vec: Vec<Node> = Vec::new();
        for node in nodes {
            node_vec.push(node);
        }

        node_vec
    }

    pub fn get_node(&self) -> Vec<Node> {
        let subject_node_vec: Vec<Node> = self
            .triples
            .iter()
            .map(|triple| triple.subject.clone())
            .collect();
        let object_node_vec: Vec<Node> = self
            .triples
            .iter()
            .map(|triple| triple.object.clone())
            .collect();
        subject_node_vec
            .into_iter()
            .chain(object_node_vec.into_iter())
            .collect()
    }

    /// ノードの出現回数を算出する
    pub fn group_by_node_count(&self) -> Vec<NodeCount> {
        // let nodes = Self::get_unique_node(&self);
        let nodes = self.get_node();

        let mut node_count_map: HashMap<String, usize> = HashMap::new();
        for node in nodes {
            let count = node_count_map.entry(node).or_insert(0);
            *count += 1;
        }

        // sort by count
        let mut node_count: Vec<NodeCount> = node_count_map
            .into_iter()
            .map(|(node, count)| NodeCount { node, count })
            .collect();

        node_count.sort_by(|a, b| b.count.cmp(&a.count));
        node_count
    }

    /// 指定ノードによるフィルタリング
    pub fn filter_by_target_nodes(&self, target_nodes: &Vec<Node>) -> Self {
        let mut triples: Vec<Triple> = Vec::new();
        for triple in &self.triples {
            if target_nodes.contains(&triple.subject) && target_nodes.contains(&triple.object) {
                triples.push(triple.clone());
            }
        }

        Graph {
            graph_name: self.graph_name.clone(),
            triples,
        }
    }

    /// ノード名のパターンによるフィルタリング
    ///
    /// * `include_node_name_pattern` - 抽出対象とするノード名のパターン
    /// * `remove_node_name_pattern` - 抽出対象から除外するノード名のパターン
    pub fn filter_by_pattern_nodes(
        &self,
        include_node_name_pattern: &RegexSet,
        remove_node_name_pattern: &RegexSet,
    ) -> Self {
        let nodes: Vec<Node> = self.get_unique_node();

        let include_nodes: Vec<Node> = nodes
            .into_iter()
            .filter(|node| include_node_name_pattern.is_match(&node))
            .collect();

        let filtered_nodes: Vec<Node> = include_nodes
            .into_iter()
            .filter(|node| !remove_node_name_pattern.is_match(&node))
            .collect();

        self.filter_by_target_nodes(&filtered_nodes)
    }

    pub fn save_as_n3(&self, file_name: &str) -> Result<(), GraphWriteError> {
        // create directory if not exists
        let path = Path::new(file_name);
        let dir = path.parent().unwrap();
        if !path.parent().unwrap().exists() {
            create_dir_all(dir).context(IoSnafu)?;
        }

        let mut file = File::create(file_name).context(IoSnafu)?;
        for triple in self.triples.iter() {
            let n3 = format!(
                "<{}> <{}> <{}> .\n",
                triple.subject, triple.predicate, triple.object
            );
            file.write_all(n3.as_bytes()).context(IoSnafu)?;
        }

        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum GraphWriteError {
    #[snafu(display("IO Error: {}", source))]
    IoError { source: std::io::Error },
}
