mod graph;
mod sparql;
mod utils;
mod word_search;

use graph::{Graph, Node, NodeCount};
use regex::RegexSet;
use snafu::prelude::*;

use utils::{concat_graphs, reduce_node_counts};
use word_search::get_graphs_from_file;

use crate::utils::get_node_over_count;

/// 検索後のリストからグラフを取得しKnowledgeGraphとして保存する
///
/// * `file_path` - 検索語のリストが記載されたファイルのパス
async fn get_kgs(file_path: &str) -> Result<(), Error> {
    // let include_node_name_pattern_list: Vec<&str> = [r"http://ja.dbpedia.org/resource/+"].to_vec();
    let include_node_name_pattern_list: Vec<&str> =
        [r"http://ja.dbpedia.org/resource/Category+"].to_vec();
    let exclude_node_name_pattern_list: Vec<&str> = [
        // 日付ノードは除外
        r"http://ja.dbpedia.org/resource/(\\d{1})月(\\d{1})日",
        r"http://ja.dbpedia.org/resource/(\\d{1})月(\\d{2})日",
        r"http://ja.dbpedia.org/resource/(\\d{2})月(\\d{1})日",
        r"http://ja.dbpedia.org/resource/(\\d{2})月(\\d{2})日",
        r"http://ja.dbpedia.org/resource/(\\d{4})年",
        // テンプレートノードは除外
        r"http://ja.dbpedia.org/resource/Template:+",
        // 地名関連のノードは除外
        r"http://ja.dbpedia.org/resource/[^\\s]+[都道府県市区町村郡(地方)]",
    ]
    .to_vec();

    let graphs = get_graphs_from_file(
        file_path,
        &include_node_name_pattern_list,
        &exclude_node_name_pattern_list,
    )
    .await
    .context(WordSearchSnafu)?;

    for graph in &graphs {
        println!("graph_name: {}", graph.graph_name);

        let file_name = format!("{}/{}/{}/{}.n3", "output", "rdf", "kg", graph.graph_name);
        graph.save_as_n3(&file_name).context(GraphWriteSnafu)?;
    }

    let kg = concat_graphs(&graphs);
    // filter by multi edge node
    let node_count_thres = 2; // 2つ以上のエッジを持つノードのみを対象とする
    let node_counts = kg.group_by_node_count();
    dbg!(&node_counts[0..10]);

    let multi_edge_node = node_counts
        .into_iter()
        .filter(|nc| nc.count >= node_count_thres)
        .map(|nc| nc.node)
        .collect::<Vec<Node>>();
    let kg_filtered = kg.filter_by_target_nodes(&multi_edge_node);

    let file_name = format!(
        "{}/{}/{}/{}.n3",
        "output", "rdf", "kg", kg_filtered.graph_name
    );
    kg_filtered
        .save_as_n3(&file_name)
        .context(GraphWriteSnafu)?;

    Ok(())
}

/// 検索語のリストからグラフを取得しそのまま保存する
///
/// * `file_path` - 検索語のリストが記載されたファイルのパス
// async fn get_graphs(file_path: &str) -> Result<(), Error> {
//     let graphs = get_graphs_from_file(file_path)
//         .await
//         .context(WordSearchSnafu)?;

//     for graph in graphs {
//         println!("graph_name: {}", graph.graph_name);

//         let file_name = format!("{}/{}/{}/{}.n3", "output", "rdf", "raw", graph.graph_name);
//         graph.save_as_n3(&file_name).context(GraphWriteSnafu)?;
//     }

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    let search_word_file_path = "input/search_words.csv";
    get_kgs(search_word_file_path).await?;

    Ok(())
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("word searh error: {}", source))]
    WordSearch { source: word_search::Error },
    #[snafu(display("sparql error: {}", source))]
    Sparql { source: sparql::Error },
    #[snafu(display("regex error: {}", source))]
    Regex { source: regex::Error },
    #[snafu(display("io error: {}", source))]
    GraphWrite { source: graph::GraphWriteError },
}
