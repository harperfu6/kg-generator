use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
};

use regex::RegexSet;
use snafu::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct LinkAndNode {
    pub link: String,
    pub node: String,
}

#[derive(Debug)]
pub struct NodeCount {
    node: String,
    count: usize,
}

/// remove duplicate link and node
pub fn remove_duplicates(link_nodes: Vec<LinkAndNode>) -> Vec<LinkAndNode> {
    let mut link_nodes_set: HashSet<LinkAndNode> = HashSet::new();
    for link_node in link_nodes {
        link_nodes_set.insert(link_node);
    }

    let mut link_nodes_vec: Vec<LinkAndNode> = Vec::new();
    for link_node in link_nodes_set {
        link_nodes_vec.push(link_node);
    }

    link_nodes_vec
}

/// remove duplicate node
pub fn remove_duplicate_node(nodes: Vec<String>) -> Vec<String> {
    let mut node_set: HashSet<String> = HashSet::new();
    for node in nodes {
        node_set.insert(node);
    }

    let mut node_vec: Vec<String> = Vec::new();
    for node in node_set {
        node_vec.push(node);
    }

    node_vec
}

/// group by node count
pub fn group_by_node_count(nodes: Vec<String>) -> Vec<NodeCount> {
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

/// get target nodes by node count and remove node name pattern
pub fn get_target_nodes(
    node_counts: Vec<NodeCount>,
    node_count_thres: &usize,
    include_node_name_pattern: RegexSet,
    remove_node_name_pattern: RegexSet,
) -> Vec<String> {
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

pub fn filter_by_target_nodes(
    link_nodes: HashMap<String, Vec<LinkAndNode>>,
    target_nodes: Vec<String>,
) -> HashMap<String, Vec<LinkAndNode>> {
    link_nodes
        .into_iter()
        .fold(HashMap::new(), |mut acc, (search_word, link_nodes)| {
            let target_link_nodes: Vec<LinkAndNode> = link_nodes
                .into_iter()
                .filter(|link_node| target_nodes.contains(&link_node.node))
                .collect();

            if !target_link_nodes.is_empty() {
                acc.insert(search_word, target_link_nodes);
            }
            acc
        })
}

pub fn save_as_n3(
    link_nodes: HashMap<String, Vec<LinkAndNode>>,
    file_name: &str,
) -> Result<(), GraphWriteError> {
    let mut file = File::create(file_name).context(IoSnafu)?;
    for (search_word, link_nodes) in link_nodes {
        for link_node in link_nodes {
            let n3 = format!(
                "<{}> <{}> <{}> .\n",
                search_word, link_node.link, link_node.node
            );
            file.write(n3.as_bytes()).context(IoSnafu)?;
        }
    }

    Ok(())
}

#[derive(Debug, Snafu)]
pub enum GraphWriteError {
    #[snafu(display("IO Error: {}", source))]
    IoError { source: std::io::Error },
}
