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
    pub link_nodes: Vec<LinkAndNode>,
}

pub type Node = String;
pub type Link = String;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct LinkAndNode {
    pub node: Node,
    pub link: Link,
}

#[derive(Debug)]
pub struct NodeCount {
    node: Node,
    count: usize,
}

impl Graph {
    pub fn new(graph_name: &str, link_nodes: Vec<LinkAndNode>) -> Self {
        Graph {
            graph_name: graph_name.to_string(),
            link_nodes,
        }
    }

    /// remove duplicate link and node
    pub fn remove_duplicates(self) -> Self {
        let mut link_nodes_set: HashSet<LinkAndNode> = HashSet::new();
        for link_node in self.link_nodes {
            link_nodes_set.insert(link_node);
        }

        let mut link_nodes_vec: Vec<LinkAndNode> = Vec::new();
        for link_node in link_nodes_set {
            link_nodes_vec.push(link_node);
        }

        Graph {
            graph_name: self.graph_name,
            link_nodes: link_nodes_vec,
        }
    }

    /// remove duplicate node
    pub fn get_unique_node(&self) -> Vec<Node> {
        let nodes: Vec<Node> = self
            .link_nodes
            .iter()
            .map(|link_node| link_node.node.clone())
            .collect();

        let mut node_set: HashSet<Node> = HashSet::new();
        for node in nodes {
            node_set.insert(node);
        }

        let mut node_vec: Vec<Node> = Vec::new();
        for node in node_set {
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

    pub fn filter_by_target_nodes(&self, target_nodes: &Vec<Node>) -> Self {
        self.link_nodes.iter().fold(
            Graph::new(&self.graph_name, Vec::new()),
            |mut graph, link_node| {
                if target_nodes.contains(&link_node.node) {
                    graph.link_nodes.push(link_node.clone());
                }
                graph
            },
        )
    }

    pub fn save_as_n3(&self, file_name: &str) -> Result<(), GraphWriteError> {
        // create directory if not exists
        let path = Path::new(file_name);
        let dir = path.parent().unwrap();
        if !path.parent().unwrap().exists() {
            create_dir_all(dir).context(IoSnafu)?;
        }

        let mut file = File::create(file_name).context(IoSnafu)?;
        for link_node in self.link_nodes.iter() {
            let n3 = format!(
                "<{}> <{}> <{}> .\n",
                self.graph_name, link_node.link, link_node.node
            );
            file.write_all(n3.as_bytes()).context(IoSnafu)?;
        }

        Ok(())
    }
}

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

/// get target nodes by node count and remove node name pattern
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
