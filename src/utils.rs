use std::collections::HashMap;

use crate::graph::{Graph, Node, NodeCount, Triple};

/// すべてのグラフに含まれるノードごとの出現数
///
/// * `graphs` - グラフのリスト
pub fn reduce_node_counts(graphs: &Vec<Graph>) -> Vec<NodeCount> {
    let mut node_count_map: HashMap<String, usize> = HashMap::new();
    let node_counts_vec: Vec<Vec<NodeCount>> = graphs
        .into_iter()
        .map(|graph| graph.group_by_node_count())
        .collect();

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

/// 指定の出現回数以上のノードを抽出する
///
/// * `node_counts` - すべてのグラフに含まれるノードごとの出現数
/// * `node_count_thres` - 抽出対象とするノードの最低出現数
pub fn get_node_over_count(node_counts: &Vec<NodeCount>, node_count_thres: usize) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    for node_count in node_counts {
        if node_count.count >= node_count_thres {
            nodes.push(node_count.node.clone());
        }
    }

    nodes
}

/// 複数のグラフを結合する
///
/// * `graphs` - グラフのリスト
pub fn concat_graphs(graphs: &Vec<Graph>) -> Graph {
    let triples = graphs
        .into_iter()
        .map(|graph| graph.triples.clone())
        .flatten()
        .collect::<Vec<Triple>>();

    Graph::new("all", triples)
}
