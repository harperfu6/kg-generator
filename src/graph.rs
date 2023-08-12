use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use regex::RegexSet;
use snafu::prelude::*;

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
    node: Node,
    count: usize,
}

impl Graph {
    pub fn new(graph_name: &str, triples: Vec<Triple>) -> Self {
        Graph {
            graph_name: graph_name.to_string(),
            triples,
        }
    }

    /// remove duplicate triple
    pub fn remove_duplicates(self) -> Self {
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

    /// get unique nodes (including subject and object)
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

    /// group by node count
    pub fn group_by_node_count(&self) -> Vec<NodeCount> {
        let nodes = Self::get_unique_node(&self);

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

    /// filter by target nodes
    pub fn filter_by_target_nodes(
        &self,
        subject_target_nodes: Vec<Node>,
        object_target_nodes: Vec<Node>,
    ) -> Self {
        let mut triples: Vec<Triple> = Vec::new();
        for triple in &self.triples {
            if subject_target_nodes.contains(&triple.subject)
                && object_target_nodes.contains(&triple.object)
            {
                triples.push(triple.clone());
            }
        }

        Graph {
            graph_name: self.graph_name.clone(),
            triples,
        }
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

/// すべてのグラフに含まれるノードごとの出現数
pub fn reduce_node_counts(node_counts_vec: Vec<Vec<NodeCount>>) -> Vec<NodeCount> {
    let mut node_count_map: HashMap<String, usize> = HashMap::new();
    for node_counts in node_counts_vec {
        for node_count in node_counts {
            let count = node_count_map.entry(node_count.node).or_insert(0);
            *count += node_count.count;
        }
    }

    // sort by count
    let mut node_count: Vec<NodeCount> = node_count_map
        .into_iter()
        .map(|(node, count)| NodeCount { node, count })
        .collect();

    node_count.sort_by(|a, b| b.count.cmp(&a.count));
    node_count
}

/// 抽出対象とするノードを取得する
///
/// * `node_counts` - すべてのグラフに含まれるノードごとの出現数
/// * `node_count_thres` - 抽出対象とするノードの最低出現数
/// * `include_node_name_pattern` - 抽出対象とするノード名のパターン
/// * `remove_node_name_pattern` - 抽出対象から除外するノード名のパターン
pub fn get_target_nodes(
    node_counts: Vec<NodeCount>,
    node_count_thres: &usize,
    include_node_name_pattern: RegexSet,
    remove_node_name_pattern: RegexSet,
) -> Vec<Node> {
    let node_over_count_thres: Vec<NodeCount> = node_counts
        .into_iter()
        .filter(|node_count| node_count.count >= *node_count_thres)
        .collect();

    let include_node_name_pattern: Vec<NodeCount> = node_over_count_thres
        .into_iter()
        .filter(|node_count| include_node_name_pattern.is_match(&node_count.node))
        .collect();

    let node_remove_pattern: Vec<NodeCount> = include_node_name_pattern
        .into_iter()
        .filter(|node_count| !remove_node_name_pattern.is_match(&node_count.node))
        .collect();

    node_remove_pattern
        .into_iter()
        .map(|node_count| node_count.node)
        .collect()
}

#[derive(Debug, Snafu)]
pub enum GraphWriteError {
    #[snafu(display("IO Error: {}", source))]
    IoError { source: std::io::Error },
}
