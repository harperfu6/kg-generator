mod graph;
mod sparql;
mod utils;
mod word_search;

use graph::{Graph, NodeCount};
use regex::RegexSet;
use snafu::prelude::*;

use utils::reduce_node_counts;
use word_search::get_graphs_from_file;

use crate::utils::get_node_over_count;

/// 検索後のリストからグラフを取得しKnowledgeGraphとして保存する
///
/// * `file_path` - 検索語のリストが記載されたファイルのパス
async fn get_kgs(file_path: &str) -> Result<(), Error> {
    let include_node_name_pattern_list: Vec<&str> = [r"http://ja.dbpedia.org/resource/+"].to_vec();
    let remove_node_name_pattern_list: Vec<&str> = [
        // 日付ノードは除外
        r"http://ja.dbpedia.org/resource/(\d{1})月(\d{1})日",
        r"http://ja.dbpedia.org/resource/(\d{1})月(\d{2})日",
        // r"http://ja.dbpedia.org/resource/(\d{2})月(\d{1})日",
        // r"http://ja.dbpedia.org/resource/(\d{2})月(\d{2})日",
        // r"http://ja.dbpedia.org/resource/(\d{4})年",
        // // テンプレートノードは除外
        // r"http://ja.dbpedia.org/resource/Template:+",
        // // 地名関連のノードは除外
        // r"http://ja.dbpedia.org/resource/[^\s]+[都道府県市区町村郡(地方)]",
    ]
    .to_vec();

    let include_node_name_pattern =
        // resourceノードのみを抽出
        RegexSet::new(&[r"http://ja.dbpedia.org/resource/+"]).context(RegexSnafu)?;
    let remove_node_name_pattern = RegexSet::new(&[
        // 日付ノードは除外
        r"http://ja.dbpedia.org/resource/(\d{1})月(\d{1})日",
        r"http://ja.dbpedia.org/resource/(\d{1})月(\d{2})日",
        r"http://ja.dbpedia.org/resource/(\d{2})月(\d{1})日",
        r"http://ja.dbpedia.org/resource/(\d{2})月(\d{2})日",
        r"http://ja.dbpedia.org/resource/(\d{4})年",
        // テンプレートノードは除外
        r"http://ja.dbpedia.org/resource/Template:+",
        // 地名関連のノードは除外
        r"http://ja.dbpedia.org/resource/[^\s]+[都道府県市区町村郡(地方)]",
    ])
    .context(RegexSnafu)?;

    let graphs = get_graphs_from_file(
        file_path,
        &include_node_name_pattern_list,
        &remove_node_name_pattern_list,
    )
    .await
    .context(WordSearchSnafu)?;
    // dbg!(&graphs);

    let all_node_counts = reduce_node_counts(&graphs);
    // 知識グラフとして構築するためには各グラフ間でエッジが張られる必要がある
    // グラフ数と同じにすると、各グラフについてすべてのノードがエッジを持つことになり厳しい条件なので
    // 要検討
    // let node_count_thres = graphs.len();
    let node_count_thres = 1; // 2つ以上のエッジを持つノードのみを対象とする
    let node_over_thres = get_node_over_count(&all_node_counts, node_count_thres);

    let kgs = graphs
        .into_iter()
        .map(|g| g.filter_by_pattern_nodes(&include_node_name_pattern, &remove_node_name_pattern))
        .map(|g| g.filter_by_target_nodes(&node_over_thres))
        .collect::<Vec<Graph>>();

    for kg in kgs {
        println!("graph_name: {}", kg.graph_name);

        let file_name = format!("{}/{}/{}/{}.n3", "output", "rdf", "kg", kg.graph_name);
        kg.save_as_n3(&file_name).context(GraphWriteSnafu)?;
    }

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
